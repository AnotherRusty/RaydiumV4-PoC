use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;

use raydium_amm_poc::math::PoolCalculator;

fn main() -> Result<()> {
    let client: RpcClient = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let ray_to_sol_amm_pool_key: Pubkey = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;
    let sol_to_usdt_amm_pool_key: Pubkey = Pubkey::from_str("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX")?;

    let pc_and_liquidity_in_sol = PoolCalculator::calc_pc_and_liquidity_in_sol(&client, &amm_program_key, &ray_to_sol_amm_pool_key).unwrap();
    let sol_in_usd = PoolCalculator::calc_pc_and_liquidity_in_sol(&client, &amm_program_key, &sol_to_usdt_amm_pool_key).unwrap().base_toke_price;

    println!("Ray is {} Sol", pc_and_liquidity_in_sol.base_toke_price);
    println!("Ray is {} USD", pc_and_liquidity_in_sol.base_toke_price * sol_in_usd);
    println!("Liquidity is {} Sol", pc_and_liquidity_in_sol.liquidity);
    println!("Liquidity is {} USD", pc_and_liquidity_in_sol.liquidity * sol_in_usd);

    Ok(())
}
