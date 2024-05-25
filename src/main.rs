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
    // let amm_info: AmmInfo = rpc::get_account::<AmmInfo>(&client, &amm_pool)?.unwrap();
    let amm_keys = utils::load_amm_keys(&client, &amm_program, &amm_pool).unwrap();
    let market_keys =
        openbooks::get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)
            .unwrap();
    let calculate_result = amm::calculate_pool_vault_amounts(
        &client,
        &amm_program,
        &amm_pool,
        &amm_keys,
        &market_keys,
        amm::utils::CalculateMethod::CalculateWithLoadAccount,
    ).unwrap();
    println!("calculate_result:{:?}", calculate_result);
    Ok(())
}

fn main() -> Result<()> {
    let cluster_url = "https://api.mainnet-beta.solana.com/";
    let amm_program = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let amm_pool = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;

    get_pool_pda(amm_program, amm_pool, &cluster_url)?;

    Ok(())
}
