use crate::raydium_amm::state::AmmInfo;
use raydium_amm::{
    error::AmmError,
    math::{U128, U256},
};
use serum_dex::{
    matching::Side,
    state::{EventView, MarketState, OpenOrders, ToAlignedBytes},
};
use solana_program::{account_info::AccountInfo, log::sol_log_compute_units, msg};
use std::{cmp::Eq, convert::identity, convert::TryInto};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum SwapDirection {
    /// Input token pc, output token coin
    PC2Coin = 1u64,
    /// Input token coin, output token pc
    Coin2PC = 2u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Calculator {}

impl Calculator {
    pub fn to_u128(val: u64) -> Result<u128, AmmError> {
        val.try_into().map_err(|_| AmmError::ConversionFailure)
    }

    pub fn to_u64(val: u128) -> Result<u64, AmmError> {
        val.try_into().map_err(|_| AmmError::ConversionFailure)
    }

    pub fn calc_x_power(last_x: U256, last_y: U256, current_x: U256, current_y: U256) -> U256 {
        // must be use u256, because u128 may be overflow
        let x_power = last_x
            .checked_mul(last_y)
            .unwrap()
            .checked_mul(current_x)
            .unwrap()
            .checked_div(current_y)
            .unwrap();
        x_power
    }

    pub fn calc_exact_vault_in_serum<'a>(
        open_orders: &'a OpenOrders,
        market_state: &'a Box<MarketState>,
        event_q_account: &'a AccountInfo,
        amm_open_account: &'a AccountInfo,
    ) -> Result<(u64, u64), AmmError> {
        let event_q = market_state.load_event_queue_mut(event_q_account).unwrap();
        let mut native_pc_total = open_orders.native_pc_total;
        let mut native_coin_total = open_orders.native_coin_total;
        msg!("calc_exact len:{}", event_q.len());
        sol_log_compute_units();
        for event in event_q.iter() {
            if identity(event.owner) != (*amm_open_account.key).to_aligned_bytes() {
                continue;
            }
            // msg!("{:?}", event.as_view().unwrap());
            match event.as_view().unwrap() {
                EventView::Fill {
                    side,
                    maker,
                    native_qty_paid,
                    native_qty_received,
                    native_fee_or_rebate: _,
                    fee_tier: _,
                    order_id: _,
                    owner: _,
                    owner_slot: _,
                    client_order_id: _,
                } => {
                    match side {
                        Side::Bid if maker => {
                            native_pc_total -= native_qty_paid;
                            native_coin_total += native_qty_received;
                        }
                        Side::Ask if maker => {
                            native_coin_total -= native_qty_paid;
                            native_pc_total += native_qty_received;
                        }
                        _ => (),
                    };
                }
                _ => {
                    continue;
                }
            }
        }
        sol_log_compute_units();
        Ok((native_pc_total, native_coin_total))
    }

    pub fn calc_total_without_take_pnl<'a>(
        pc_amount: u64,
        coin_amount: u64,
        open_orders: &'a OpenOrders,
        amm: &'a AmmInfo,
        market_state: &'a Box<MarketState>,
        event_q_account: &'a AccountInfo,
        amm_open_account: &'a AccountInfo,
    ) -> Result<(u64, u64), AmmError> {
        let (pc_total_in_serum, coin_total_in_serum) = Self::calc_exact_vault_in_serum(
            open_orders,
            market_state,
            event_q_account,
            amm_open_account,
        )?;

        let total_pc_without_take_pnl = pc_amount
            .checked_add(pc_total_in_serum)
            .ok_or(AmmError::CheckedAddOverflow)?
            .checked_sub(amm.state_data.need_take_pnl_pc)
            .ok_or(AmmError::CheckedSubOverflow)?;
        let total_coin_without_take_pnl = coin_amount
            .checked_add(coin_total_in_serum)
            .ok_or(AmmError::CheckedAddOverflow)?
            .checked_sub(amm.state_data.need_take_pnl_coin)
            .ok_or(AmmError::CheckedSubOverflow)?;
        Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
    }

    pub fn calc_total_without_take_pnl_no_orderbook<'a>(
        pc_amount: u64,
        coin_amount: u64,
        amm: &'a AmmInfo,
    ) -> Result<(u64, u64), AmmError> {
        let total_pc_without_take_pnl = pc_amount
            .checked_sub(amm.state_data.need_take_pnl_pc)
            .ok_or(AmmError::CheckedSubOverflow)?;
        let total_coin_without_take_pnl = coin_amount
            .checked_sub(amm.state_data.need_take_pnl_coin)
            .ok_or(AmmError::CheckedSubOverflow)?;
        Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
    }

    pub fn swap_token_amount_base_in(
        amount_in: U128,
        total_pc_without_take_pnl: U128,
        total_coin_without_take_pnl: U128,
        swap_direction: SwapDirection,
    ) -> U128 {
        let amount_out;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_out = pc - coin * pc / (coin + amount_in)
                // => amount_out = ((pc * coin + pc * amount_in) - coin * pc) / (coin + amount_in)
                // => amount_out =  pc * amount_in / (coin + amount_in)
                let denominator = total_coin_without_take_pnl.checked_add(amount_in).unwrap();
                amount_out = total_pc_without_take_pnl
                    .checked_mul(amount_in)
                    .unwrap()
                    .checked_div(denominator)
                    .unwrap();
            }
            SwapDirection::PC2Coin => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (pc + amount_in) * (coin - amount_out) = coin * pc
                // => amount_out = coin - coin * pc / (pc + amount_in)
                // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
                // => amount_out = coin * amount_in / (pc + amount_in)
                let denominator = total_pc_without_take_pnl.checked_add(amount_in).unwrap();
                amount_out = total_coin_without_take_pnl
                    .checked_mul(amount_in)
                    .unwrap()
                    .checked_div(denominator)
                    .unwrap();
            }
        }
        return amount_out;
    }

    pub fn swap_token_amount_base_out(
        amount_out: U128,
        total_pc_without_take_pnl: U128,
        total_coin_without_take_pnl: U128,
        swap_direction: SwapDirection,
    ) -> U128 {
        let amount_in;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_in = coin * pc / (pc - amount_out) - coin
                // => amount_in = (coin * pc - pc * coin + amount_out * coin) / (pc - amount_out)
                // => amount_in = (amount_out * coin) / (pc - amount_out)
                let denominator = total_pc_without_take_pnl.checked_sub(amount_out).unwrap();
                amount_in = total_coin_without_take_pnl
                    .checked_mul(amount_out)
                    .unwrap()
                    .checked_ceil_div(denominator)
                    .unwrap()
                    .0;
            }
            SwapDirection::PC2Coin => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (pc + amount_in) * (coin - amount_out) = coin * pc
                // => amount_out = coin - coin * pc / (pc + amount_in)
                // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
                // => amount_out = coin * amount_in / (pc + amount_in)

                // => amount_in = coin * pc / (coin - amount_out) - pc
                // => amount_in = (coin * pc - pc * coin + pc * amount_out) / (coin - amount_out)
                // => amount_in = (pc * amount_out) / (coin - amount_out)
                let denominator = total_coin_without_take_pnl.checked_sub(amount_out).unwrap();
                amount_in = total_pc_without_take_pnl
                    .checked_mul(amount_out)
                    .unwrap()
                    .checked_ceil_div(denominator)
                    .unwrap()
                    .0;
            }
        }
        return amount_in;
    }
}

pub trait CheckedCeilDiv: Sized {
    /// Perform ceiling division
    fn checked_ceil_div(&self, rhs: Self) -> Option<(Self, Self)>;
}

impl CheckedCeilDiv for U128 {
    fn checked_ceil_div(&self, mut rhs: Self) -> Option<(Self, Self)> {
        let mut quotient = self.checked_div(rhs)?;
        // Avoid dividing a small number by a big one and returning 1, and instead
        // fail.
        let zero = U128::from(0);
        let one = U128::from(1);
        if quotient.is_zero() {
            // return None;
            if self.checked_mul(U128::from(2))? >= rhs {
                return Some((one, zero));
            } else {
                return Some((zero, zero));
            }
        }

        // Ceiling the destination amount if there's any remainder, which will
        // almost always be the case.
        let remainder = self.checked_rem(rhs)?;
        if remainder > zero {
            quotient = quotient.checked_add(one)?;
            // calculate the minimum amount needed to get the dividend amount to
            // avoid truncating too much
            rhs = self.checked_div(quotient)?;
            let remainder = self.checked_rem(quotient)?;
            if remainder > zero {
                rhs = rhs.checked_add(one)?;
            }
        }
        Some((quotient, rhs))
    }
}
