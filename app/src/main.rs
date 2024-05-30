use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use raydium_amm_poc::loader::PoolLoader;
use raydium_amm_poc::math::PoolMath;
use raydium_amm_poc::types::SwapDirection;

fn main() -> Result<()> {
    let client: RpcClient = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let ray_sol_pool_key: Pubkey = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;
    let sol_usdt_pool_key: Pubkey = Pubkey::from_str("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX")?;

    // load everything
    let state_ray_sol = PoolLoader::load_state(&client, &amm_program_key, &ray_sol_pool_key).unwrap();
    let state_usd_sol = PoolLoader::load_state(&client, &amm_program_key, &sol_usdt_pool_key).unwrap();

    // only math, no rpc requests
    let ray_in_sol = PoolMath::calc_coin_in_sol(&state_ray_sol).unwrap();
    let sol_in_usd = PoolMath::calc_coin_in_sol(&state_usd_sol).unwrap();
    let liquidity = PoolMath::calc_liquidity(&state_ray_sol).unwrap() * sol_in_usd;
    let m_cap = liquidity / 2.0;
    let ray_in_usd = ray_in_sol * sol_in_usd;

    let amount_in = PoolMath::calc_swap_token_amount_base_in(&state_ray_sol, 10.0, SwapDirection::Coin2PC).unwrap();
    let amount_out = PoolMath::calc_swap_token_amount_base_out(&state_ray_sol, 10.0, SwapDirection::Coin2PC).unwrap();
    
    println!("Ray = {} sol", ray_in_sol);
    println!("Ray = {} usd", ray_in_usd);
    println!("liquidity = {} usd", liquidity);
    println!("MarketCap = {} usd", m_cap);
    println!("10 Ray in, {} sol out", amount_in);
    println!("10 sol out, {} Ray in", amount_out);

    Ok(())
}
