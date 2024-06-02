use anyhow::Result;
use raydium_amm_poc::rpc::get_account;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use raydium_amm_poc::amm_math::{
    calc_swap_token_amount_base_in, calc_swap_token_amount_base_out, load_state,
    simulate_calc_swap_token_amount, swap_with_slippage, calc_coin_in_pc, calc_pool_liquidity,
};
use raydium_amm_poc::openbook::get_keys_for_market;
use raydium_amm_poc::raydium_amm::maths::SwapDirection;
use raydium_amm_poc::raydium_amm::AmmInfo;
use raydium_amm_poc::utils::load_amm_keys;

fn simulate_transaction() -> Result<()> {
    //simulate maths swap
    let client: RpcClient = RpcClient::new("https://api.devnet.solana.com/".to_string());
    let amm_program_key: Pubkey = Pubkey::from_str("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8")?;
    let coin_pc_pool: Pubkey = Pubkey::from_str("AZzhMReRb1ZqVRoFh6Yk2YaVH8eVYJXQP3SmrDNdKB5V")?;
    let wallet_pub_key: Pubkey = Pubkey::from_str("2S6WuiBfjGVbCX4ism2ReASkS7raDgpSPgZgeqEdf4bu")?;
    let slippage_bps = 50u64;
    let amount_specified = 100000000000000000;

    let amm_info: AmmInfo = get_account::<AmmInfo>(&client, &coin_pc_pool)?.unwrap();
    let amm_keys = load_amm_keys(&amm_program_key, &coin_pc_pool, &amm_info).unwrap();
    let market_keys = get_keys_for_market(&client, &amm_keys.market_program, &amm_keys.market)?;
    let user_source = spl_associated_token_account::get_associated_token_address(&wallet_pub_key,&amm_info.coin_vault_mint,);
    let user_destination = spl_associated_token_account::get_associated_token_address(&wallet_pub_key,&amm_info.pc_vault_mint,);
    let state_coin_pc = load_state(&client, &amm_program_key, &coin_pc_pool).unwrap();
    let other_amount_threshold = swap_with_slippage(
        state_coin_pc.pool_pc_vault_amount,
        state_coin_pc.pool_coin_vault_amount,
        state_coin_pc.swap_fee_numerator,
        state_coin_pc.swap_fee_denominator,
        SwapDirection::Coin2PC,
        amount_specified,
        true,
        slippage_bps,
    )?;

    simulate_calc_swap_token_amount(
        &client,
        &amm_program_key,
        &amm_keys,
        &market_keys,
        &wallet_pub_key,
        &user_source,
        &user_destination,
        amount_specified,
        other_amount_threshold,
        true,
    )
    .unwrap();

    Ok(())
}

fn main() -> Result<()> {
    let client: RpcClient = RpcClient::new("https://api.mainnet-beta.solana.com/".to_string());
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let coin_pc_pool: Pubkey = Pubkey::from_str("FTLA9G7cj1MGxa715JZo3dL9SiNrw2suLCd2fRL5P9g7")?;
    let sol_usdt_pool_key: Pubkey = Pubkey::from_str("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX")?;
    let slippage_bps = 50u64; // 0.5%
    
    // load everything
    let state_coin_pc = load_state(&client, &amm_program_key, &coin_pc_pool).unwrap();
    let state_sol_usd = load_state(&client, &amm_program_key, &sol_usdt_pool_key).unwrap();
    
    // off-chain swap
    let amount_out = calc_swap_token_amount_base_in(&state_coin_pc, SwapDirection::Coin2PC, 1000000000).unwrap();
    let amount_in = calc_swap_token_amount_base_out(&state_coin_pc, SwapDirection::Coin2PC, 10000000000).unwrap();
    println!("===================Math Trade===================");
    println!("1000 BaseToken In -> {} QuoteToken Out",(amount_out as f64) / (10_f64.powf(state_coin_pc.pool_pc_decimals as f64)));
    println!("10 QuoteToken Out <- {} BaseToken Out", (amount_in as f64) / (10_f64.powf(state_coin_pc.pool_coin_decimals as f64)));
    let amount_threshold_for_amount_in = swap_with_slippage(
        state_coin_pc.pool_pc_vault_amount,
        state_coin_pc.pool_coin_vault_amount,
        state_coin_pc.swap_fee_numerator,
        state_coin_pc.swap_fee_denominator,
        SwapDirection::Coin2PC,
        1000000000,
        true,
        slippage_bps,
    )?;
    let amount_threshold_for_amount_out = swap_with_slippage(
        state_coin_pc.pool_pc_vault_amount,
        state_coin_pc.pool_coin_vault_amount,
        state_coin_pc.swap_fee_numerator,
        state_coin_pc.swap_fee_denominator,
        SwapDirection::Coin2PC,
        10000000000,
        false,
        slippage_bps,
    )?;
    println!("===================Simulation===================");
    println!("1000 BaseToken In -> {} min QuoteToken Out",(amount_threshold_for_amount_in as f64) / (10_f64.powf(state_coin_pc.pool_pc_decimals as f64)));
    println!("10 QuoteToken Out <- {} Max BaseToken In", (amount_threshold_for_amount_out as f64) / (10_f64.powf(state_coin_pc.pool_coin_decimals as f64)));
    println!("===================Coin Price===================");
    let coin_in_pc = calc_coin_in_pc(&state_coin_pc).unwrap();
    println!("1 BaseToken = {} QuoteToken", coin_in_pc);
    let sol_in_usd = calc_coin_in_pc(&state_sol_usd).unwrap();
    print!("1 BaseToken = {} usd\n", coin_in_pc * sol_in_usd);
    println!("=================Pool Liquidity=================");
    print!("Pool Liquidity = {} usd\n", sol_in_usd * calc_pool_liquidity(&state_coin_pc).unwrap());
    Ok(())
}
