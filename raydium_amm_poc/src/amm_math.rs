use anyhow::Result;
use arrayref::array_ref;
use safe_transmute::{to_bytes::transmute_to_bytes, transmute_one_pedantic};
use solana_client::rpc_client::RpcClient;
use solana_program::{
    account_info::{AccountInfo, IntoAccountInfo},
    program_pack::Pack,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, message::Message, pubkey::Pubkey, transaction::Transaction,
};
use spl_token::state::Account;

use raydium_amm::{
    math::{CheckedCeilDiv, U128},
    processor,
    state::{AmmStatus, TargetOrders},
    log::decode_ray_log,
};

use crate::raydium_amm::maths::{Calculator, SwapDirection};
use crate::raydium_amm::processor::Processor;
use crate::raydium_amm::state::AmmInfo;

use crate::rpc::{get_account, get_multiple_accounts, simulate_transaction};
use crate::utils::load_amm_keys;
use crate::{
    instruction::swap,
    openbook::{get_keys_for_market, MarketPubkeys},
    utils::AmmKeys,
};

pub const TEN_THOUSAND: u64 = 10000;

#[derive(Clone, Copy, Debug)]
pub struct CalculateResult {
    pub pool_pc_vault_amount: u64,
    pub pool_pc_decimals: u64,
    pub pool_coin_vault_amount: u64,
    pub pool_coin_decimals: u64,
    pub pool_lp_amount: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
}

// pool_vault_amount = vault_amount + open_orders.native_total + partial filled without consumed - amm.need_take
pub fn calc_pool_valut_amounts(
    client: &RpcClient,
    amm_program_key: &Pubkey,
    amm_pool_key: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &MarketPubkeys,
    amm: &AmmInfo,
) -> Result<CalculateResult> {
    let load_pubkeys: Vec<Pubkey> = vec![
        *amm_pool_key,
        amm_keys.amm_target,
        amm_keys.amm_pc_vault,
        amm_keys.amm_coin_vault,
        amm_keys.amm_open_order,
        amm_keys.market,
        *market_keys.event_q,
    ];
    let rsps = get_multiple_accounts(client, &load_pubkeys)?;
    let accounts = array_ref![rsps, 0, 7];
    let [_, amm_target_account, amm_pc_vault_account, amm_coin_vault_account, amm_open_orders_account, market_account, market_event_q_account] =
        accounts;
    let _amm_target: TargetOrders = transmute_one_pedantic::<TargetOrders>(transmute_to_bytes(
        &amm_target_account.as_ref().unwrap().clone().data,
    ))
    .map_err(|e| e.without_src())?;
    let amm_pc_vault =
        Account::unpack(&amm_pc_vault_account.as_ref().unwrap().clone().data).unwrap();
    let amm_coin_vault =
        Account::unpack(&amm_coin_vault_account.as_ref().unwrap().clone().data).unwrap();
    let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) = if AmmStatus::from_u64(amm.status)
        .orderbook_permission()
    {
        // println!("AMM + OpenBook");
        let amm_open_orders_account = &mut amm_open_orders_account.as_ref().unwrap().clone();
        let market_account = &mut market_account.as_ref().unwrap().clone();
        let market_event_q_account = &mut market_event_q_account.as_ref().unwrap().clone();

        let amm_open_orders_info = (&amm.open_orders, amm_open_orders_account).into_account_info();
        let market_account_info = (&amm.market, market_account).into_account_info();
        let market_event_queue_info: AccountInfo =
            (&(*market_keys.event_q), market_event_q_account).into_account_info();

        let amm_authority =
            Pubkey::find_program_address(&[processor::AUTHORITY_AMM], &amm_program_key).0;
        let lamports = &mut 0;
        let data = &mut [0u8];
        let owner = Pubkey::default();
        let amm_authority_info = AccountInfo::new(
            &amm_authority,
            false,
            false,
            lamports,
            data,
            &owner,
            false,
            0,
        );
        let (market_state, open_orders) = Processor::load_serum_market_order(
            &market_account_info,
            &amm_open_orders_info,
            &amm_authority_info,
            &amm,
            false,
        )?;
        let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
            Calculator::calc_total_without_take_pnl(
                amm_pc_vault.amount,
                amm_coin_vault.amount,
                &open_orders,
                &amm,
                &market_state,
                &market_event_queue_info,
                &amm_open_orders_info,
            )?;
        (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount)
    } else {
        // println!("only AMM");
        let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
            Calculator::calc_total_without_take_pnl_no_orderbook(
                amm_pc_vault.amount,
                amm_coin_vault.amount,
                &amm,
            )?;
        (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount)
    };
    Ok(CalculateResult {
        pool_pc_vault_amount: amm_pool_pc_vault_amount,
        pool_pc_decimals: amm.pc_decimals,
        pool_coin_vault_amount: amm_pool_coin_vault_amount,
        pool_coin_decimals: amm.coin_decimals,
        pool_lp_amount: amm.lp_amount,
        swap_fee_numerator: amm.fees.swap_fee_numerator,
        swap_fee_denominator: amm.fees.swap_fee_denominator,
    })
}

pub fn calc_swap_token_amount_base_in(
    state: &CalculateResult,
    swap_direction: SwapDirection,
    amount_specified: u64,
) -> Result<u64> {
    let swap_fee = U128::from(amount_specified)
        .checked_mul(state.swap_fee_numerator.into())
        .unwrap()
        .checked_ceil_div(state.swap_fee_denominator.into())
        .unwrap()
        .0;
    let swap_in_after_deduct_fee = U128::from(amount_specified).checked_sub(swap_fee).unwrap();
    let swap_amount_out = Calculator::swap_token_amount_base_in(
        swap_in_after_deduct_fee,
        state.pool_pc_vault_amount.into(),
        state.pool_coin_vault_amount.into(),
        swap_direction,
    )
    .as_u64();
    Ok(swap_amount_out)
}

pub fn simulate_calc_swap_token_amount(
    client: &RpcClient,
    amm_program: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &MarketPubkeys,
    user_owner: &Pubkey,
    user_source: &Pubkey,
    user_destination: &Pubkey,
    amount_specified: u64,
    other_amount_threshold: u64,
    swap_base_in: bool,
) -> Result<()> {
    let simulate_swap_instruction = swap(
        amm_program,
        amm_keys,
        market_keys,
        user_owner,
        user_source,
        user_destination,
        amount_specified,
        other_amount_threshold,
        swap_base_in,
    )?;
    let mut message = Message::new(&[simulate_swap_instruction], Some(&user_owner));
    message.recent_blockhash = client.get_latest_blockhash()?;
    let txn = Transaction::new_unsigned(message);
    let response_from_simulation = simulate_transaction(&client, &txn, false, CommitmentConfig::confirmed())?;
    let logs = response_from_simulation.value.logs.unwrap();
    if let Some(ray_log_entry) = logs.iter().find(|log| log.contains("ray_log:")) {
        // Extract the ray_log value
        if let Some(start) = ray_log_entry.find("ray_log:") {
            let ray_log_value = &ray_log_entry[start + "ray_log: ".len()..];
            let _decode_ray_log = decode_ray_log(ray_log_value);
        }
    } else {
        println!("Transaction Simulation Failed");
    }

    Ok(())
}

pub fn calc_swap_token_amount_base_out(
    state: &CalculateResult,
    swap_direction: SwapDirection,
    amount_specified: u64,
) -> Result<u64> {
    let swap_in_before_add_fee = Calculator::swap_token_amount_base_out(
        amount_specified.into(),
        state.pool_pc_vault_amount.into(),
        state.pool_coin_vault_amount.into(),
        swap_direction,
    );
    let swap_in_after_add_fee = swap_in_before_add_fee
        .checked_mul(state.swap_fee_denominator.into())
        .unwrap()
        .checked_ceil_div(
            (state
                .swap_fee_denominator
                .checked_sub(state.swap_fee_numerator)
                .unwrap())
            .into(),
        )
        .unwrap()
        .0
        .as_u64();

    Ok(swap_in_after_add_fee)
}

pub fn load_state(
    client: &RpcClient,
    amm_program_key: &Pubkey,
    amm_pool_key: &Pubkey,
) -> Result<CalculateResult> {
    let amm_info: AmmInfo = get_account::<AmmInfo>(&client, &amm_pool_key)?.unwrap();
    let amm_keys: AmmKeys = load_amm_keys(&amm_program_key, &amm_pool_key, &amm_info).unwrap();
    let market_keys: MarketPubkeys =
        get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)?;
    let calculate_result: CalculateResult = calc_pool_valut_amounts(
        &client,
        &amm_program_key,
        &amm_pool_key,
        &amm_keys,
        &market_keys,
        &amm_info,
    )?;

    Ok(calculate_result)
}

pub fn max_amount_with_slippage(input_amount: u64, slippage_bps: u64) -> u64 {
    input_amount
        .checked_mul(slippage_bps.checked_add(TEN_THOUSAND).unwrap())
        .unwrap()
        .checked_div(TEN_THOUSAND)
        .unwrap()
}

pub fn min_amount_with_slippage(input_amount: u64, slippage_bps: u64) -> u64 {
    input_amount
        .checked_mul(TEN_THOUSAND.checked_sub(slippage_bps).unwrap())
        .unwrap()
        .checked_div(TEN_THOUSAND)
        .unwrap()
}

pub fn swap_exact_amount(
    pc_vault_amount: u64,
    coin_vault_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    swap_direction: SwapDirection,
    amount_specified: u64,
    swap_base_in: bool,
) -> Result<u64> {
    let other_amount_threshold = if swap_base_in {
        let swap_fee = U128::from(amount_specified)
            .checked_mul(swap_fee_numerator.into())
            .unwrap()
            .checked_ceil_div(swap_fee_denominator.into())
            .unwrap()
            .0;
        let swap_in_after_deduct_fee = U128::from(amount_specified).checked_sub(swap_fee).unwrap();
        let swap_amount_out = Calculator::swap_token_amount_base_in(
            swap_in_after_deduct_fee,
            pc_vault_amount.into(),
            coin_vault_amount.into(),
            swap_direction,
        )
        .as_u64();
        swap_amount_out
    } else {
        let swap_in_before_add_fee = Calculator::swap_token_amount_base_out(
            amount_specified.into(),
            pc_vault_amount.into(),
            coin_vault_amount.into(),
            swap_direction,
        );
        let swap_in_after_add_fee = swap_in_before_add_fee
            .checked_mul(swap_fee_denominator.into())
            .unwrap()
            .checked_ceil_div(
                (swap_fee_denominator
                    .checked_sub(swap_fee_numerator)
                    .unwrap())
                .into(),
            )
            .unwrap()
            .0
            .as_u64();

        swap_in_after_add_fee
    };

    Ok(other_amount_threshold)
}

pub fn swap_with_slippage(
    pc_vault_amount: u64,
    coin_vault_amount: u64,
    swap_fee_numerator: u64,
    swap_fee_denominator: u64,
    swap_direction: SwapDirection,
    amount_specified: u64,
    swap_base_in: bool,
    slippage_bps: u64,
) -> Result<u64> {
    let other_amount_threshold = swap_exact_amount(
        pc_vault_amount,
        coin_vault_amount,
        swap_fee_numerator,
        swap_fee_denominator,
        swap_direction,
        amount_specified,
        swap_base_in,
    )?;
    let other_amount_threshold = if swap_base_in {
        // min out
        min_amount_with_slippage(other_amount_threshold, slippage_bps)
    } else {
        // max in
        max_amount_with_slippage(other_amount_threshold, slippage_bps)
    };
    Ok(other_amount_threshold)
}

pub fn calc_coin_in_pc(
    state: &CalculateResult
) -> Result<f64> {
    // pc_amount * pc_price = coin_amount * coin_price
    // coin_price = pc_price * (pc_amount / coin_amount)
    
    Ok((state.pool_pc_vault_amount as f64) / 10_f64.powf(state.pool_pc_decimals as f64) / (state.pool_coin_vault_amount as f64) * 10_f64.powf(state.pool_coin_decimals as f64))
}

pub fn calc_pool_liquidity(
    state: &CalculateResult
) -> Result<f64> {
    // liquidity = 2 * sol_amount * sol_price
    Ok(2.0 * (state.pool_pc_vault_amount as f64) / 10_f64.powf(state.pool_pc_decimals as f64))
}