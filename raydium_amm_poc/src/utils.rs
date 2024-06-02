use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

// use crate::raydium_amm::{processor, state::AmmInfo, Processor};
use raydium_amm::{processor, processor::Processor};
use crate::raydium_amm::state::AmmInfo;

#[derive(Clone, Copy, Debug)]
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
    amm_program_key: &Pubkey,
    amm_pool_key: &Pubkey,
    amm_info: &AmmInfo,
) -> Result<AmmKeys> {
    Ok(AmmKeys {
        amm_pool: *amm_pool_key,
        amm_target: amm_info.target_orders,
        amm_coin_vault: amm_info.coin_vault,
        amm_pc_vault: amm_info.pc_vault,
        amm_lp_mint: amm_info.lp_mint,
        amm_open_order: amm_info.open_orders,
        amm_coin_mint: amm_info.coin_vault_mint,
        amm_pc_mint: amm_info.pc_vault_mint,
        amm_authority: Processor::authority_id(
            amm_program_key,
            processor::AUTHORITY_AMM,
            amm_info.nonce as u8,
        )?,
        market: amm_info.market,
        market_program: amm_info.market_program,
        nonce: amm_info.nonce as u8,
    })
}
