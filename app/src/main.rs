use anyhow::Result;
use raydium_amm_poc::rpc::get_account;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use raydium_amm_poc::amm_math::{
    calc_swap_token_amount_base_in, calc_swap_token_amount_base_out, load_state,
    simulate_calc_swap_token_amount, swap_with_slippage,
};
use raydium_amm_poc::openbook::get_keys_for_market;
use raydium_amm_poc::raydium_amm::maths::SwapDirection;
use raydium_amm_poc::raydium_amm::AmmInfo;
use raydium_amm_poc::utils::load_amm_keys;

#[derive(Debug)]
pub enum SimulationMode {
    MainNetMode,
    DevNetMode,
}

fn simulate_transaction(
    simulation_mode: SimulationMode,
    wallet_pubkey: &Pubkey,
    amm_pool_key: &Pubkey, 
    slippage_bps: u64
) -> Result<()> {
    let client: RpcClient;
    let amm_program_key: Pubkey;
    
    match simulation_mode {
        SimulationMode::MainNetMode => {
            client = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());
            amm_program_key = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
        }
        SimulationMode::DevNetMode => {
            client = RpcClient::new("https://api.devnet.solana.com/".to_string());
            amm_program_key = Pubkey::from_str("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8")?;
        }
    }

    let amm_info: AmmInfo = get_account::<AmmInfo>(&client, &amm_pool_key)?.unwrap();
    let amm_keys = load_amm_keys(&amm_program_key, &amm_pool_key, &amm_info).unwrap();
    let market_keys = get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)?;
    let user_source = spl_associated_token_account::get_associated_token_address(&wallet_pubkey,&amm_info.coin_vault_mint,);
    let user_destination = spl_associated_token_account::get_associated_token_address(&wallet_pubkey,&amm_info.pc_vault_mint,);
    let state_coin_pc = load_state(&client, &amm_program_key, &amm_pool_key).unwrap();

    let amount_threshold_for_base_out = swap_with_slippage(
        state_coin_pc.pool_pc_vault_amount,
        state_coin_pc.pool_coin_vault_amount,
        state_coin_pc.swap_fee_numerator,
        state_coin_pc.swap_fee_denominator,
        SwapDirection::Coin2PC,
        100000000000000,
        true,
        slippage_bps,
    )?;

    let amount_threshold_for_base_in = swap_with_slippage(
        state_coin_pc.pool_pc_vault_amount,
        state_coin_pc.pool_coin_vault_amount,
        state_coin_pc.swap_fee_numerator,
        state_coin_pc.swap_fee_denominator,
        SwapDirection::Coin2PC,
        10000000,
        false,
        slippage_bps,
    )?;
    
    let _simulate_base_in = simulate_calc_swap_token_amount(
        &client,
        &amm_program_key,
        &amm_keys,
        &market_keys,
        &wallet_pubkey,
        &user_source,
        &user_destination,
        100000000000000,
        amount_threshold_for_base_out,
        true,
    )
    .unwrap();

    let _simulate_base_out = simulate_calc_swap_token_amount(
        &client,
        &amm_program_key,
        &amm_keys,
        &market_keys,
        &wallet_pubkey,
        &user_source,
        &user_destination,
        10000000,
        amount_threshold_for_base_in,
        false,
    )
    .unwrap();

    Ok(())
}

fn main() -> Result<()> {
    let slippage_bps = 50u64; // 0.5%
    //DevNet
    let client: RpcClient = RpcClient::new("https://api.devnet.solana.com/".to_string());
    let amm_program_key: Pubkey = Pubkey::from_str("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8")?;
    let coin_pc_pool: Pubkey = Pubkey::from_str("AZzhMReRb1ZqVRoFh6Yk2YaVH8eVYJXQP3SmrDNdKB5V")?;
    let wallet_pubkey: Pubkey = Pubkey::from_str("2S6WuiBfjGVbCX4ism2ReASkS7raDgpSPgZgeqEdf4bu")?;

    //Mainnet
    // let client: RpcClient = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());
    // let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    // let coin_pc_pool: Pubkey = Pubkey::from_str("FTLA9G7cj1MGxa715JZo3dL9SiNrw2suLCd2fRL5P9g7")?;
    // let wallet_pubkey: Pubkey = Pubkey::from_str("Your Wallet Pubkey")?;
    
    // load everything
    let state_coin_pc = load_state(&client, &amm_program_key, &coin_pc_pool).unwrap();
    
    // off-chain swap
    let amount_out = calc_swap_token_amount_base_in(&state_coin_pc, SwapDirection::Coin2PC, 100000000000000).unwrap();
    let amount_in = calc_swap_token_amount_base_out(&state_coin_pc, SwapDirection::Coin2PC, 10000000).unwrap();
    println!("===================Math Trade===================");
    println!("100000 BaseToken In -> {} QuoteToken Out",(amount_out as f64) / (10_f64.powf(state_coin_pc.pool_pc_decimals as f64)));
    println!("100 QuoteToken Out <- {} BaseToken Out", (amount_in as f64) / (10_f64.powf(state_coin_pc.pool_coin_decimals as f64)));
    println!("===================Simulation Swap===================");
    let _= simulate_transaction(SimulationMode::DevNetMode, &wallet_pubkey, &coin_pc_pool, slippage_bps);
    Ok(())
}