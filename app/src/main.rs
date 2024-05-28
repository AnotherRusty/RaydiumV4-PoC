use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use raydium_amm_poc::get_pool_pda_data_on_raydium;
use raydium_amm_poc::PoolTokenPairResult;

fn main() -> Result<()> {
    let cluster_url: &str = "https://api.mainnet-beta.solana.com/";
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let ray_to_sol_amm_pool_key: Pubkey =
        Pubkey::from_str("AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA")?;
    let sol_to_usdt_amm_pool_key: Pubkey =
        Pubkey::from_str("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX")?;

    let ray_to_sol_pool: PoolTokenPairResult =
        get_pool_pda_data_on_raydium(amm_program_key, ray_to_sol_amm_pool_key, &cluster_url)?;

    let sol_to_usdt_pool: PoolTokenPairResult =
        get_pool_pda_data_on_raydium(amm_program_key, sol_to_usdt_amm_pool_key, &cluster_url)?;

    print!("Ray Price is {} in Sol\n", ray_to_sol_pool.base_toke_price);
    print!(
        "Ray Price is {} in usd\n",
        ray_to_sol_pool.base_toke_price * sol_to_usdt_pool.base_toke_price
    );
    print!("liquidity is {} in usd\n", ray_to_sol_pool.liquidity);
    print!(
        "Ray Price is {} in usd\n",
        ray_to_sol_pool.liquidity * sol_to_usdt_pool.base_toke_price
    );
    Ok(())
}
