use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use raydium_amm_poc::{loader::PoolLoader, math::PoolMath, types::SwapDirection, };

pub fn test_trade_math() -> Result<()> {
    let client: RpcClient = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    // test with CPMM + Openbook pool
    println!("=============Trade Test with CPMM + OpenBook Pool ========");
    let bonk_to_sol_pool_key: Pubkey = Pubkey::from_str("HVNwzt7Pxfu76KHCMQPTLuTCLTm6WnQ1esLv4eizseSv")?;
    // load bonk/ray pool state
    let state_bonk_sol = PoolLoader::load_state(&client, &amm_program_key, &bonk_to_sol_pool_key).unwrap();
    // off-chain trade result
    let amount_in = PoolMath::calc_swap_token_amount_base_in(&state_bonk_sol, 10000000, SwapDirection::Coin2PC).unwrap();
    let amount_out = PoolMath::calc_swap_token_amount_base_out(&state_bonk_sol, 1, SwapDirection::Coin2PC).unwrap();
    println!("In on-chain trade, 10000000 Bonk in -> 2.047878748 sol out");
    println!("In on-chain trade, 1 Sol out -> 4896057.43411 Bonk in");
    println!("In off-chian trade, 10000000 Bonk in -> {} Sol out", amount_in);
    println!("In off-chain trade, 1 Sol out, {} Bonk in\n", amount_out);
    println!("================Trade Test with Only CPMM Pool ===========");
    let woof_to_ray_pool_key: Pubkey = Pubkey::from_str("3HYhQC6ne6SAPVT5sPTKawRUxv9ZpYyLuk1ifrw8baov")?;
    // load woof/ray pool state
    let state_woof_ray = PoolLoader::load_state(&client, &amm_program_key, &woof_to_ray_pool_key).unwrap();
    // off-chain trade result
    let amount_in = PoolMath::calc_swap_token_amount_base_in(&state_woof_ray, 100000, SwapDirection::Coin2PC).unwrap();
    let amount_out = PoolMath::calc_swap_token_amount_base_out(&state_woof_ray, 1, SwapDirection::Coin2PC).unwrap();
    println!("In on-chain trade, 100000 Woof in -> 4.471361 Ray out");
    println!("In on-chain trade, 1 Ray out -> 22364.558904 Woof in");
    println!("In off-chian trade, 100000 Woof in -> {} Ray out", amount_in);
    println!("In off-chain trade, 1 Ray out, {} Woof in", amount_out);

    Ok(())
}

fn main() -> Result<()> {
    let client: RpcClient = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    //
    let ray_sol_pool_key: Pubkey = Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;
    let sol_usdt_pool_key: Pubkey = Pubkey::from_str("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX")?;

    // load everything
    let state_ray_sol = PoolLoader::load_state(&client, &amm_program_key, &ray_sol_pool_key).unwrap();
    let state_usd_sol = PoolLoader::load_state(&client, &amm_program_key, &sol_usdt_pool_key).unwrap();

    // only math, no rpc requests
    let ray_in_sol = PoolMath::calc_coin_in_sol(&state_ray_sol).unwrap();
    let sol_in_usd = PoolMath::calc_coin_in_sol(&state_usd_sol).unwrap();
    let ray_in_usd = ray_in_sol * sol_in_usd;
    let liquidity_in_sol = PoolMath::calc_liquidity(&state_ray_sol).unwrap() * sol_in_usd;
    
    println!("Ray = {} sol", ray_in_sol);
    println!("Ray = {} usd", ray_in_usd);
    println!("liquidity = {} usd", liquidity_in_sol);
    let _ = test_trade_math();

    Ok(())
}
