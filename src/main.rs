#![allow(dead_code)]

pub mod common;

use anyhow::Result;
use common::rpc;
use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

fn get_pool_pda(amm_pool: Pubkey, cluster_url: &str) -> Result<()> {
    let client = RpcClient::new(cluster_url.to_string());
    let amm_info: raydium_amm::state::AmmInfo =
        rpc::get_account::<raydium_amm::state::AmmInfo>(&client, &amm_pool)?.unwrap();

    println!("amm_pool:{:?}", amm_info);

    Ok(())
}

fn main() -> Result<()> {
    let cluster_url = "https://api.mainnet-beta.solana.com/";
    let amm_pool = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;

    get_pool_pda(amm_pool, &cluster_url)?;

    Ok(())
}
