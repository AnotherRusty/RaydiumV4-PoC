#![allow(dead_code)]

pub mod amm;
pub mod common;

use amm::{
    calculate_pool_vault_amounts, get_keys_for_market, load_amm_keys, AmmKeys, CalculateResult,
    MarketPubkeys,
};
use common::{get_account, AmmInfo};

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/**
 * calculate the pool pda data by pool id
 *
 * # Arguments
 *
 * * 'amm_program_key' - RaydiumV3 program address
 * * 'amm_pool_key' - RaydiumV3 pool id
 * * 'cluster_url' - solana mainnet rpc url
 *
 * # Returns
 */
fn get_pool_pda_data_on_raydium(
    amm_program_key: Pubkey,
    amm_pool_key: Pubkey,
    cluster_url: &str,
) -> Result<(f64, f64)> {
    let client: RpcClient = RpcClient::new(cluster_url.to_string());
    let amm_info: AmmInfo = get_account::<AmmInfo>(&client, &amm_pool_key)?.unwrap();
    let amm_keys: AmmKeys = load_amm_keys(&amm_program_key, &amm_pool_key, &amm_info).unwrap();
    let market_keys: MarketPubkeys =
        get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)?;
    let calculate_result: CalculateResult = calculate_pool_vault_amounts(
        &client,
        &amm_program_key,
        &amm_pool_key,
        &amm_keys,
        &market_keys,
    )?;
    let quote_token_decimal: f64 = amm_info.pc_decimals as f64;
    let base_token_decimal: f64 = amm_info.coin_decimals as f64;
    let quote_token_amount: f64 = calculate_result.pool_pc_vault_amount as f64;
    let base_token_amount: f64 = calculate_result.pool_coin_vault_amount as f64;
    let base_toke_price: f64 = (quote_token_amount / (10_f64.powf(quote_token_decimal)))
        / (base_token_amount / (10_f64.powf(base_token_decimal)));
    let liqudity_as_quote_token: f64 =
        quote_token_amount / (10_f64.powf(quote_token_decimal)) * 2_f64;

    Ok((base_toke_price, liqudity_as_quote_token))
}

fn main() -> Result<()> {
    let cluster_url: &str = "https://api.mainnet-beta.solana.com/";
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let amm_pool_key: Pubkey = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;

    println!(
        "{:?}",
        get_pool_pda_data_on_raydium(amm_program_key, amm_pool_key, &cluster_url)
    );

    Ok(())
}
