use anyhow::{format_err, Result};
use std::str::FromStr;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};
pub struct AmmKeys {
    pub amm_pool: Pubkey,
    pub amm_coin_mint: Pubkey,
    pub amm_pc_mint: Pubkey,
    pub amm_authority: Pubkey,
    pub amm_target: Pubkey,
    pub amm_coin_vault: Pubkey,
    pub amm_pc_vault: Pubkey,
    pub amm_lp_mint: Pubkey,
    pub amm_open_order: Pubkey,
    pub market_program: Pubkey,
    pub market: Pubkey,
    pub nonce: u8,
}

pub fn load_amm_keys(
    client: &RpcClient,
    amm_program: &Pubkey,
    amm_pool: &Pubkey,
) -> Result<AmmKeys> {
    let amm = get_account::<raydium_amm::state::AmmInfo>(client, &amm_pool)?.unwrap();
    Ok(AmmKeys {
        amm_pool: *amm_pool,
        amm_target: amm.target_orders,
        amm_coin_vault: amm.coin_vault,
        amm_pc_vault: amm.pc_vault,
        amm_lp_mint: amm.lp_mint,
        amm_open_order: amm.open_orders,
        amm_coin_mint: amm.coin_vault_mint,
        amm_pc_mint: amm.pc_vault_mint,
        amm_authority: raydium_amm::processor::Processor::authority_id(
            amm_program,
            raydium_amm::processor::AUTHORITY_AMM,
            amm.nonce as u8,
        )?,
        market: amm.market,
        market_program: amm.market_program,
        nonce: amm.nonce as u8,
    })
}

fn get_amm_poo_info() -> Result<()> {
    // config params
    let cluster_url = "https://api.devnet.solana.com/";
    let amm_program = Pubkey::from_str("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8")?;
    let amm_pool_id = Pubkey::from_str("BbZjQanvSaE9me4adAitmTTaSgASvzaVignt4HRSM7ww")?;
    let input_token_mint = Pubkey::from_str("GfmdKWR1KrttDsQkJfwtXovZw9bUBHYkPAEwB6wZqQvJ")?;
    let output_token_mint = Pubkey::from_str("2SiSpNowr7zUv5ZJHuzHszskQNaskWsNukhivCtuVLHo")?;

    let client = RpcClient::new(cluster_url.to_string());

    // load amm keys
    let amm_keys = load_amm_keys(&client, &amm_program, &amm_pool_id)?;

    Ok(())
}

fn main() -> Result<()> {
    get_amm_poo_info()?;
    Ok(())
}