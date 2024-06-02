use anyhow::Result;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use crate::openbook;
use crate::utils::AmmKeys;

pub fn swap(
    amm_program: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &openbook::MarketPubkeys,
    user_owner: &Pubkey,
    user_source: &Pubkey,
    user_destination: &Pubkey,
    amount_specified: u64,
    other_amount_threshold: u64,
    swap_base_in: bool,
) -> Result<Instruction> {
    let swap_instruction = if swap_base_in {
        raydium_amm::instruction::swap_base_in(
            &amm_program,
            &amm_keys.amm_pool,
            &amm_keys.amm_authority,
            &amm_keys.amm_open_order,
            &amm_keys.amm_coin_vault,
            &amm_keys.amm_pc_vault,
            &amm_keys.market_program,
            &amm_keys.market,
            &market_keys.bids,
            &market_keys.asks,
            &market_keys.event_q,
            &market_keys.coin_vault,
            &market_keys.pc_vault,
            &market_keys.vault_signer_key,
            user_source,
            user_destination,
            user_owner,
            amount_specified,
            other_amount_threshold,
        )?
    } else {
        raydium_amm::instruction::swap_base_out(
            &amm_program,
            &amm_keys.amm_pool,
            &amm_keys.amm_authority,
            &amm_keys.amm_open_order,
            &amm_keys.amm_coin_vault,
            &amm_keys.amm_pc_vault,
            &amm_keys.market_program,
            &amm_keys.market,
            &market_keys.bids,
            &market_keys.asks,
            &market_keys.event_q,
            &market_keys.coin_vault,
            &market_keys.pc_vault,
            &market_keys.vault_signer_key,
            user_source,
            user_destination,
            user_owner,
            other_amount_threshold,
            amount_specified,
        )?
    };

    Ok(swap_instruction)
}