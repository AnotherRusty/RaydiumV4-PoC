use serum_dex::state::{Market, MarketState, OpenOrders, ToAlignedBytes};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use std::{convert::identity, ops::Deref};
use raydium_amm::error::AmmError;

use crate::raydium_amm::AmmInfo;

pub const AUTHORITY_AMM: &'static [u8] = b"amm authority";
/// Program state handler.
pub struct Processor {}

impl Processor {
    /// Calculates the authority id by generating a program address.
    pub fn authority_id(
        program_id: &Pubkey,
        amm_seed: &[u8],
        nonce: u8,
    ) -> Result<Pubkey, AmmError> {
        Pubkey::create_program_address(&[amm_seed, &[nonce]], program_id)
            .map_err(|_| AmmError::InvalidProgramAddress.into())
    }

    pub fn load_serum_market_order<'a>(
        market_acc: &AccountInfo<'a>,
        open_orders_acc: &AccountInfo<'a>,
        authority_acc: &AccountInfo<'a>,
        amm: &AmmInfo,
        // Allow for the market flag to be set to AccountFlag::Disabled
        allow_disabled: bool,
    ) -> Result<(Box<MarketState>, Box<OpenOrders>), ProgramError> {
        let market_state = Market::load(market_acc, &amm.market_program, allow_disabled)?;
        let open_orders = market_state.load_orders_mut(
            open_orders_acc,
            Some(authority_acc),
            &amm.market_program,
            None,
            None,
        )?;
        if identity(open_orders.market) != market_acc.key.to_aligned_bytes() {
            return Err(AmmError::InvalidMarket.into());
        }
        if identity(open_orders.owner) != authority_acc.key.to_aligned_bytes() {
            return Err(AmmError::InvalidOwner.into());
        }
        if *open_orders_acc.key != amm.open_orders {
            return Err(AmmError::InvalidOpenOrders.into());
        }
        return Ok((
            Box::new(*market_state.deref()),
            Box::new(*open_orders.deref()),
        ));
    }
}
