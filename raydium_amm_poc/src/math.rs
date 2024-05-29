use anyhow::{Ok, Result};
use arrayref::array_ref;
use raydium_amm::error::AmmError;
use raydium_amm::math::Calculator;
use raydium_amm::processor;
use raydium_amm::state::{AmmStatus, TargetOrders};
use safe_transmute::{to_bytes::transmute_to_bytes, transmute_one_pedantic};
use serum_dex::state::{MarketState, OpenOrders};
use solana_client::rpc_client::RpcClient;
use solana_program::account_info::{AccountInfo, IntoAccountInfo};
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Account;

use crate::processor::load_serum_market_order;
use crate::state::{get_account, get_keys_for_market, get_multiple_accounts, load_amm_keys};
use crate::types::{AmmInfo, AmmKeys, CalculateResult, MarketPubkeys, PoolTokenPairResult};

#[derive(Clone, Debug, PartialEq)]
pub struct PoolCalculator {}

impl PoolCalculator {
    pub fn calc_total_without_take_pnl<'a>(
        pc_amount: u64,
        coin_amount: u64,
        open_orders: &'a OpenOrders,
        amm: &'a AmmInfo,
        market_state: &'a Box<MarketState>,
        event_q_account: &'a AccountInfo,
        amm_open_account: &'a AccountInfo,
    ) -> Result<(u64, u64)> {
        let (pc_total_in_serum, coin_total_in_serum) = Calculator::calc_exact_vault_in_serum(
            open_orders,
            market_state,
            event_q_account,
            amm_open_account,
        )?;

        let total_pc_without_take_pnl = pc_amount
            .checked_add(pc_total_in_serum)
            .ok_or(AmmError::CheckedAddOverflow)?
            .checked_sub(amm.state_data.need_take_pnl_pc)
            .ok_or(AmmError::CheckedSubOverflow)?;
        let total_coin_without_take_pnl = coin_amount
            .checked_add(coin_total_in_serum)
            .ok_or(AmmError::CheckedAddOverflow)?
            .checked_sub(amm.state_data.need_take_pnl_coin)
            .ok_or(AmmError::CheckedSubOverflow)?;
        Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
    }

    pub fn calc_total_without_take_pnl_no_orderbook<'a>(
        pc_amount: u64,
        coin_amount: u64,
        amm: &'a AmmInfo,
    ) -> Result<(u64, u64)> {
        let total_pc_without_take_pnl = pc_amount
            .checked_sub(amm.state_data.need_take_pnl_pc)
            .ok_or(AmmError::CheckedSubOverflow)?;
        let total_coin_without_take_pnl = coin_amount
            .checked_sub(amm.state_data.need_take_pnl_coin)
            .ok_or(AmmError::CheckedSubOverflow)?;
        Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
    }

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
        let (amm_pool_key_pc_vault_amount, amm_pool_key_coin_vault_amount) =
            if AmmStatus::from_u64(amm.status).orderbook_permission() {
                let amm_open_orders_account =
                    &mut amm_open_orders_account.as_ref().unwrap().clone();
                let market_account = &mut market_account.as_ref().unwrap().clone();
                let market_event_q_account = &mut market_event_q_account.as_ref().unwrap().clone();

                let amm_open_orders_info =
                    (&amm.open_orders, amm_open_orders_account).into_account_info();
                let market_account_info = (&amm.market, market_account).into_account_info();
                let market_event_queue_info =
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
                let (market_state, open_orders) = load_serum_market_order(
                    &market_account_info,
                    &amm_open_orders_info,
                    &amm_authority_info,
                    &amm,
                    false,
                )?;
                let (amm_pool_key_pc_vault_amount, amm_pool_key_coin_vault_amount) =
                    Self::calc_total_without_take_pnl(
                        amm_pc_vault.amount,
                        amm_coin_vault.amount,
                        &open_orders,
                        &amm,
                        &market_state,
                        &market_event_queue_info,
                        &amm_open_orders_info,
                    )?;
                (amm_pool_key_pc_vault_amount, amm_pool_key_coin_vault_amount)
            } else {
                let (amm_pool_key_pc_vault_amount, amm_pool_key_coin_vault_amount) =
                    Self::calc_total_without_take_pnl_no_orderbook(
                        amm_pc_vault.amount,
                        amm_coin_vault.amount,
                        &amm,
                    )?;
                (amm_pool_key_pc_vault_amount, amm_pool_key_coin_vault_amount)
            };
        Ok(CalculateResult {
            pool_pc_vault_amount: amm_pool_key_pc_vault_amount,
            pool_coin_vault_amount: amm_pool_key_coin_vault_amount,
            pool_lp_amount: amm.lp_amount,
            swap_fee_numerator: amm.fees.swap_fee_numerator,
            swap_fee_denominator: amm.fees.swap_fee_denominator,
        })
    }

    pub fn calc_pc_and_liquidity_in_sol(
        client: &RpcClient,
        amm_program_key: &Pubkey,
        amm_pool_key: &Pubkey,
    ) -> Result<PoolTokenPairResult> {
        let amm_info: AmmInfo = get_account::<AmmInfo>(&client, &amm_pool_key)?.unwrap();
        let amm_keys: AmmKeys = load_amm_keys(&amm_program_key, &amm_pool_key, &amm_info).unwrap();
        let market_keys: MarketPubkeys =
            get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)?;
        let calculate_result: CalculateResult = Self::calc_pool_valut_amounts(
            &client,
            &amm_program_key,
            &amm_pool_key,
            &amm_keys,
            &market_keys,
            &amm_info,
        )?;
        let quote_token_decimal: f64 = amm_info.pc_decimals as f64;
        let base_token_decimal: f64 = amm_info.coin_decimals as f64;
        let quote_token_amount: f64 = calculate_result.pool_pc_vault_amount as f64;
        let base_token_amount: f64 = calculate_result.pool_coin_vault_amount as f64;
        let base_toke_price: f64 = (quote_token_amount / (10_f64.powf(quote_token_decimal)))
            / (base_token_amount / (10_f64.powf(base_token_decimal)));
        let liqudity_as_quote_token: f64 =
            quote_token_amount / (10_f64.powf(quote_token_decimal)) * 2_f64;

        Ok(PoolTokenPairResult {
            base_toke_price: base_toke_price,
            liquidity: liqudity_as_quote_token,
        })
    }
}
