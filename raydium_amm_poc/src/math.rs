use anyhow::{Ok, Result};
use crate::types::{CalculateResult, SwapDirection};


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
        // liquidity = pc_amount * pc_price
        Ok(2.0 * (state.pool_pc_vault_amount as f64) / 10_f64.powf(state.pool_pc_decimals as f64))
    }

    pub fn calc_swap_token_amount_base_in(
        state: &CalculateResult,
        amount_in: f64,
        swap_direction: SwapDirection,
    ) -> Result<f64> {
        let total_pc_without_take_pnl: f64 = state.pool_pc_vault_amount as f64;
        let total_coin_without_take_pnl: f64 = state.pool_coin_vault_amount as f64;
        let amount_out;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_out = pc - coin * pc / (coin + amount_in)
                // => amount_out = ((pc * coin + pc * amount_in) - coin * pc) / (coin + amount_in)
                // => amount_out =  pc * amount_in / (coin + amount_in)
                let denominator = total_coin_without_take_pnl + amount_in;
                amount_out = (total_pc_without_take_pnl * amount_in) / denominator / 10_f64.powf(state.pool_pc_decimals as f64) * 10_f64.powf(state.pool_coin_decimals as f64);
            }
            SwapDirection::PC2Coin => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (pc + amount_in) * (coin - amount_out) = coin * pc
                // => amount_out = coin - coin * pc / (pc + amount_in)
                // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
                // => amount_out = coin * amount_in / (pc + amount_in)
                let denominator = total_pc_without_take_pnl + amount_in;
                amount_out = (total_coin_without_take_pnl * amount_in) / denominator * 10_f64.powf(state.pool_pc_decimals as f64) / 10_f64.powf(state.pool_coin_decimals as f64);
            }
        }
        Ok(amount_out)
    }

    pub fn calc_swap_token_amount_base_out(
        state: &CalculateResult,
        amount_out: f64,
        swap_direction: SwapDirection,
    ) -> Result<f64> {
        let total_pc_without_take_pnl: f64 = state.pool_pc_vault_amount as f64;
        let total_coin_without_take_pnl: f64 = state.pool_coin_vault_amount as f64;
        let amount_in;
        match swap_direction {
            SwapDirection::Coin2PC => {
                // (x + delta_x) * (y + delta_y) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // => amount_in = coin * pc / (pc - amount_out) - coin
                // => amount_in = (coin * pc - pc * coin + amount_out * coin) / (pc - amount_out)
                // => amount_in = (amount_out * coin) / (pc - amount_out)
                let denominator = total_pc_without_take_pnl - amount_out;
                amount_in = (total_coin_without_take_pnl * amount_out) / denominator * (10_f64.powf((state.pool_pc_decimals - state.pool_coin_decimals) as f64));
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
                let denominator = total_coin_without_take_pnl - amount_out;
                amount_in = (total_pc_without_take_pnl * amount_out) / denominator / (10_f64.powf((state.pool_pc_decimals - state.pool_coin_decimals) as f64));
            }
        }
        Ok(amount_in)
    }
}
