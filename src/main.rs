#![allow(dead_code)]

pub mod amm;
pub mod common;

use amm::openbooks;
use amm::utils;
use anyhow::Result;
use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

fn get_pool_pda(amm_program: Pubkey, amm_pool: Pubkey, cluster_url: &str) -> Result<()> {
    let client = RpcClient::new(cluster_url.to_string());
    let amm_info: common::AmmInfo =
        common::rpc::get_account::<common::AmmInfo>(&client, &amm_pool)?.unwrap();
    let quote_token_decimal: u64 = amm_info.pc_decimals;
    let quote_token_decimal_f64: f64 = quote_token_decimal as f64;
    let base_token_decimal: u64 = amm_info.coin_decimals;
    let base_token_decimal_f64: f64 = base_token_decimal as f64;
    let amm_keys = utils::load_amm_keys(&client, &amm_program, &amm_pool).unwrap();
    let market_keys =
        openbooks::get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)?;
    let calculate_result = amm::calculate_pool_vault_amounts(
        &client,
        &amm_program,
        &amm_pool,
        &amm_keys,
        &market_keys,
        amm::utils::CalculateMethod::CalculateWithLoadAccount,
    )?;
    let quote_token_amount: u64 = calculate_result.pool_pc_vault_amount;
    let quote_token_amount_f64: f64 = quote_token_amount as f64;
    let base_token_amount: u64 = calculate_result.pool_coin_vault_amount;
    let base_token_amount_f64: f64 = base_token_amount as f64;
    let base_toke_price: f64 = (quote_token_amount_f64 / (10_f64.powf(quote_token_decimal_f64)))
        / (base_token_amount_f64 / (10_f64.powf(base_token_decimal_f64)));
    let liqudity_as_quote_token: f64 =
        quote_token_amount_f64 / (10_f64.powf(quote_token_decimal_f64)) * 2_f64;
    println!("Base Token Price: {}", base_toke_price);
    println!("Liquidity as quotetoken: {}", liqudity_as_quote_token);
    Ok(())
}

fn main() -> Result<()> {
    let cluster_url = "https://api.mainnet-beta.solana.com/";
    let amm_program = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let amm_pool = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;

    get_pool_pda(amm_program, amm_pool, &cluster_url)?;

    Ok(())
}
