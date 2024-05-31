#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::unknown_clippy_lints)]
#![allow(clippy::manual_range_contains)]

use anyhow::{Ok, Result};
use raydium_amm::math::U128;
use std::convert::TryInto;
use raydium_amm::error::AmmError;

use crate::types::{CalculateResult, SwapDirection};

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

pub fn to_u128(val: u64) -> Result<U128, AmmError> {
    val.try_into().map_err(|_| AmmError::ConversionFailure)
}

pub fn to_u64(val: U128) -> Result<u64, AmmError> {
    val.try_into().map_err(|_| AmmError::ConversionFailure)
}

#[derive(Clone, Debug, PartialEq)]
pub struct PoolMath {}

impl PoolMath {
    pub fn calc_coin_in_sol(
        state: &CalculateResult
    ) -> Result<f64> {
        // pc_amount * pc_price = coin_amount * coin_price
        // coin_price = pc_price * (pc_amount / coin_amount)
        
        Ok((state.pool_pc_vault_amount as f64) / 10_f64.powf(state.pool_pc_decimals as f64) / (state.pool_coin_vault_amount as f64) * 10_f64.powf(state.pool_coin_decimals as f64))
    }

    pub fn calc_liquidity(
        state: &CalculateResult
    ) -> Result<f64> {
        // liquidity = 2 * sol_amount * sol_price
        Ok(2.0 * (state.pool_pc_vault_amount as f64) / 10_f64.powf(state.pool_pc_decimals as f64))
    }

    pub fn calc_swap_token_amount_base_in(
        state: &CalculateResult,
        amount_in: u64,
        swap_direction: SwapDirection,
    ) -> Result<f64> {
        let total_pc_without_take_pnl = to_u128(state.pool_pc_vault_amount).unwrap();
        let total_coin_without_take_pnl = to_u128(state.pool_coin_vault_amount).unwrap();
        let amount_out;
        let amount_out_f64: f64;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_out = pc - coin * pc / (coin + amount_in)
                // => amount_out = ((pc * coin + pc * amount_in) - coin * pc) / (coin + amount_in)
                // => amount_out =  pc * amount_in / (coin + amount_in)
                // let denominator = total_coin_without_take_pnl + amount_in;
                let _amount_in = to_u128(amount_in * 10u64.pow(state.pool_coin_decimals as u32)).unwrap();
                let denominator = total_coin_without_take_pnl.checked_add(_amount_in).unwrap();
                amount_out = total_pc_without_take_pnl
                    .checked_mul(_amount_in)
                    .unwrap()
                    .checked_div(denominator)
                    .unwrap();
                amount_out_f64 = (to_u64(amount_out).unwrap() as f64) / 10_f64.powf(state.pool_pc_decimals as f64);
            }
            SwapDirection::PC2Coin => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (pc + amount_in) * (coin - amount_out) = coin * pc
                // => amount_out = coin - coin * pc / (pc + amount_in)
                // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
                // => amount_out = coin * amount_in / (pc + amount_in)
                let _amount_in = to_u128(amount_in * 10u64.pow(state.pool_pc_decimals as u32)).unwrap();
                let denominator = total_pc_without_take_pnl.checked_add(_amount_in).unwrap();
                amount_out = total_coin_without_take_pnl
                    .checked_mul(_amount_in)
                    .unwrap()
                    .checked_div(denominator)
                    .unwrap();
                amount_out_f64 = (to_u64(amount_out).unwrap() as f64) / 10_f64.powf(state.pool_coin_decimals as f64);
            }
        }

        Ok(amount_out_f64)
    }

    pub fn calc_swap_token_amount_base_out(
        state: &CalculateResult,
        amount_out: u64,
        swap_direction: SwapDirection,
    ) -> Result<f64> {
        let total_pc_without_take_pnl = to_u128(state.pool_pc_vault_amount).unwrap();
        let total_coin_without_take_pnl = to_u128(state.pool_coin_vault_amount).unwrap();
        let amount_in;
        let amount_in_f64: f64;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_in = coin * pc / (pc - amount_out) - coin
                // => amount_in = (coin * pc - pc * coin + amount_out * coin) / (pc - amount_out)
                // => amount_in = (amount_out * coin) / (pc - amount_out)
                let _amount_out = to_u128(amount_out * 10u64.pow(state.pool_pc_decimals as u32)).unwrap();
                let denominator = total_pc_without_take_pnl.checked_sub(_amount_out).unwrap();
                amount_in = total_coin_without_take_pnl
                    .checked_mul(_amount_out)
                    .unwrap()
                    .checked_ceil_div(denominator)
                    .unwrap()
                    .0;
                amount_in_f64 = (to_u64(amount_in).unwrap() as f64) / 10_f64.powf(state.pool_coin_decimals as f64);
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
                let _amount_out = to_u128(amount_out * 10u64.pow(state.pool_coin_decimals as u32)).unwrap();
                let denominator = total_coin_without_take_pnl.checked_sub(_amount_out).unwrap();
                amount_in = total_pc_without_take_pnl
                    .checked_mul(_amount_out)
                    .unwrap()
                    .checked_ceil_div(denominator)
                    .unwrap()
                    .0;
                amount_in_f64 = (to_u64(amount_in).unwrap() as f64) / 10_f64.powf(state.pool_pc_decimals as f64);
            }
        }
        Ok(amount_in_f64)
    }
}

