use crate::common;

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;

use common::AmmInfo;

#[derive(Clone, Copy, Debug)]
pub struct AmmKeys {
    pub amm_pool_key: Pubkey,
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

#[derive(Clone, Copy, Debug)]
pub struct CalculateResult {
    pub pool_pc_vault_amount: u64,
    pub pool_coin_vault_amount: u64,
    pub pool_lp_amount: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
}

/**
 * load amm keys
 *
 * # Arguments
 *
 * * 'amm_info' - RaydiumV3 amm pool info
 * * 'amm_program_key' - RaydiumV3 program address
 * * 'amm_pool_key' - Raydium pool id
 *
 * # Returns
 */
pub fn load_amm_keys(
    amm_program_key: &Pubkey,
    amm_pool_key: &Pubkey,
    amm_info: &AmmInfo,
) -> Result<AmmKeys> {
    Ok(AmmKeys {
        amm_pool_key: *amm_pool_key,
        amm_target: amm_info.target_orders,
        amm_coin_vault: amm_info.coin_vault,
        amm_pc_vault: amm_info.pc_vault,
        amm_lp_mint: amm_info.lp_mint,
        amm_open_order: amm_info.open_orders,
        amm_coin_mint: amm_info.coin_vault_mint,
        amm_pc_mint: amm_info.pc_vault_mint,
        amm_authority: raydium_amm::processor::Processor::authority_id(
            amm_program_key,
            raydium_amm::processor::AUTHORITY_AMM,
            amm_info.nonce as u8,
        )?,
        market: amm_info.market,
        market_program: amm_info.market_program,
        nonce: amm_info.nonce as u8,
    })
}
