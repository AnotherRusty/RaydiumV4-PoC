//! State transition types

use raydium_amm::error::AmmError;
use serum_dex::state::ToAlignedBytes;
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use bytemuck::{from_bytes, from_bytes_mut, Pod, Zeroable};
use safe_transmute::{self, trivial::TriviallyTransmutable};
use std::{
    cell::{Ref, RefMut},
    convert::identity,
    mem::size_of,
};

pub const TEN_THOUSAND: u64 = 10000;
pub const MAX_ORDER_LIMIT: usize = 10;

pub trait Loadable: Pod {
    fn load_mut<'a>(account: &'a AccountInfo) -> Result<RefMut<'a, Self>, ProgramError> {
        // TODO verify if this checks for size
        Ok(RefMut::map(account.try_borrow_mut_data()?, |data| {
            from_bytes_mut(data)
        }))
    }
    fn load<'a>(account: &'a AccountInfo) -> Result<Ref<'a, Self>, ProgramError> {
        Ok(Ref::map(account.try_borrow_data()?, |data| {
            from_bytes(data)
        }))
    }

    fn load_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        Ok(from_bytes(data))
    }
}

macro_rules! impl_loadable {
    ($type_name:ident) => {
        unsafe impl Zeroable for $type_name {}
        unsafe impl Pod for $type_name {}
        unsafe impl TriviallyTransmutable for $type_name {}
        impl Loadable for $type_name {}
    };
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TargetOrder {
    pub price: u64,
    pub vol: u64,
}
#[cfg(target_endian = "little")]
unsafe impl Zeroable for TargetOrder {}
#[cfg(target_endian = "little")]
unsafe impl Pod for TargetOrder {}
#[cfg(target_endian = "little")]
unsafe impl TriviallyTransmutable for TargetOrder {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TargetOrders {
    pub owner: [u64; 4],
    pub buy_orders: [TargetOrder; 50],
    pub padding1: [u64; 8],
    pub target_x: u128,
    pub target_y: u128,
    pub plan_x_buy: u128,
    pub plan_y_buy: u128,
    pub plan_x_sell: u128,
    pub plan_y_sell: u128,
    pub placed_x: u128,
    pub placed_y: u128,
    pub calc_pnl_x: u128,
    pub calc_pnl_y: u128,
    pub sell_orders: [TargetOrder; 50],
    pub padding2: [u64; 6],
    pub replace_buy_client_id: [u64; MAX_ORDER_LIMIT],
    pub replace_sell_client_id: [u64; MAX_ORDER_LIMIT],
    pub last_order_numerator: u64,
    pub last_order_denominator: u64,

    pub plan_orders_cur: u64,
    pub place_orders_cur: u64,

    pub valid_buy_order_num: u64,
    pub valid_sell_order_num: u64,

    pub padding3: [u64; 10],

    pub free_slot_bits: u128,
}
impl_loadable!(TargetOrders);

#[cfg(test)]
impl Default for TargetOrders {
    #[inline]
    fn default() -> TargetOrders {
        TargetOrders {
            owner: [0; 4],
            buy_orders: [TargetOrder::default(); 50],
            padding1: [0; 8],
            target_x: 0,
            target_y: 0,
            plan_x_buy: 0,
            plan_y_buy: 0,
            plan_x_sell: 0,
            plan_y_sell: 0,
            placed_x: 0,
            placed_y: 0,
            calc_pnl_x: 0,
            calc_pnl_y: 0,
            sell_orders: [TargetOrder::default(); 50],
            padding2: [0; 6],
            replace_buy_client_id: [0; MAX_ORDER_LIMIT],
            replace_sell_client_id: [0; MAX_ORDER_LIMIT],
            last_order_denominator: 0,
            last_order_numerator: 0,
            plan_orders_cur: 0,
            place_orders_cur: 0,
            valid_buy_order_num: 0,
            valid_sell_order_num: 0,
            padding3: [0; 10],
            free_slot_bits: std::u128::MAX,
        }
    }
}

impl TargetOrders {
    /// init
    #[inline]
    pub fn check_init(&mut self, x: u128, y: u128, owner: &Pubkey) -> Result<(), ProgramError> {
        if identity(self.owner) != Pubkey::default().to_aligned_bytes() {
            return Err(AmmError::AlreadyInUse.into());
        }
        self.owner = owner.to_aligned_bytes();
        self.last_order_numerator = 0; // 3
        self.last_order_denominator = 0; // 1

        self.plan_orders_cur = 0;
        self.place_orders_cur = 0;

        self.valid_buy_order_num = 0;
        self.valid_sell_order_num = 0;

        self.target_x = 0;
        self.target_y = 0;
        self.plan_x_buy = 0;
        self.plan_y_buy = 0;
        self.plan_x_sell = 0;
        self.plan_y_sell = 0;
        self.placed_x = 0;
        self.placed_y = 0;
        self.calc_pnl_x = x;
        self.calc_pnl_y = y;
        self.free_slot_bits = std::u128::MAX;
        Ok(())
    }

    /// load_mut_checked
    #[inline]
    pub fn load_mut_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
        owner: &Pubkey,
    ) -> Result<RefMut<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidTargetAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load_mut(account)?;
        if identity(data.owner) != owner.to_aligned_bytes() {
            return Err(AmmError::InvalidTargetOwner.into());
        }
        Ok(data)
    }

    /// load_checked
    #[inline]
    pub fn load_checked<'a>(
        account: &'a AccountInfo,
        program_id: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Ref<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(AmmError::InvalidTargetAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(AmmError::ExpectedAccount.into());
        }
        let data = Self::load(account)?;
        if identity(data.owner) != owner.to_aligned_bytes() {
            return Err(AmmError::InvalidTargetOwner.into());
        }
        Ok(data)
    }
}

fn validate_fraction(numerator: u64, denominator: u64) -> Result<(), AmmError> {
    if numerator >= denominator || denominator == 0 {
        Err(AmmError::InvalidFee)
    } else {
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct AmmInfo {
    /// Initialized status.
    pub status: u64,
    /// Nonce used in program address.
    pub nonce: u64,
    /// max order count
    pub order_num: u64,
    /// within this range, 5 => 5% range
    pub depth: u64,
    /// coin decimal
    pub coin_decimals: u64,
    /// pc decimal
    pub pc_decimals: u64,
    /// amm machine state
    pub state: u64,
    /// amm reset_flag
    pub reset_flag: u64,
    /// min size 1->0.000001
    pub min_size: u64,
    /// vol_max_cut_ratio numerator, sys_decimal_value as denominator
    pub vol_max_cut_ratio: u64,
    /// amount wave numerator, sys_decimal_value as denominator
    pub amount_wave: u64,
    /// coinLotSize 1 -> 0.000001
    pub coin_lot_size: u64,
    /// pcLotSize 1 -> 0.000001
    pub pc_lot_size: u64,
    /// min_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub min_price_multiplier: u64,
    /// max_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub max_price_multiplier: u64,
    /// system decimal value, used to normalize the value of coin and pc amount
    pub sys_decimal_value: u64,
    /// All fee information
    pub fees: Fees,
    /// Statistical data
    pub state_data: StateData,
    /// Coin vault
    pub coin_vault: Pubkey,
    /// Pc vault
    pub pc_vault: Pubkey,
    /// Coin vault mint
    pub coin_vault_mint: Pubkey,
    /// Pc vault mint
    pub pc_vault_mint: Pubkey,
    /// lp mint
    pub lp_mint: Pubkey,
    /// open_orders key
    pub open_orders: Pubkey,
    /// market key
    pub market: Pubkey,
    /// market program key
    pub market_program: Pubkey,
    /// target_orders key
    pub target_orders: Pubkey,
    /// padding
    pub padding1: [u64; 8],
    /// amm owner key
    pub amm_owner: Pubkey,
    /// pool lp amount
    pub lp_amount: u64,
    /// client order id
    pub client_order_id: u64,
    /// padding
    pub padding2: [u64; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateData {
    /// delay to take pnl coin
    pub need_take_pnl_coin: u64,
    /// delay to take pnl pc
    pub need_take_pnl_pc: u64,
    /// total pnl pc
    pub total_pnl_pc: u64,
    /// total pnl coin
    pub total_pnl_coin: u64,
    /// ido pool open time
    pub pool_open_time: u64,
    /// swap coin in amount
    pub swap_coin_in_amount: u128,
    /// swap pc out amount
    pub swap_pc_out_amount: u128,
    /// charge pc as swap fee while swap pc to coin
    pub swap_acc_pc_fee: u64,
    /// swap pc in amount
    pub swap_pc_in_amount: u128,
    /// swap coin out amount
    pub swap_coin_out_amount: u128,
    /// charge coin as swap fee while swap coin to pc
    pub swap_acc_coin_fee: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fees {
    /// numerator of the min_separate
    pub min_separate_numerator: u64,
    /// denominator of the min_separate
    pub min_separate_denominator: u64,
    /// numerator of the fee
    pub trade_fee_numerator: u64,
    /// denominator of the fee and 'trade_fee_denominator' must be equal to 'min_separate_denominator'
    pub trade_fee_denominator: u64,
    /// numerator of the pnl
    pub pnl_numerator: u64,
    /// denominator of the pnl
    pub pnl_denominator: u64,
    /// numerator of the swap_fee
    pub swap_fee_numerator: u64,
    /// denominator of the swap_fee
    pub swap_fee_denominator: u64,
}

#[repr(u64)]
pub enum AmmStatus {
    Uninitialized = 0u64,
    Initialized = 1u64,
    Disabled = 2u64,
    WithdrawOnly = 3u64,
    // pool only can add or remove liquidity, can't swap and plan orders
    LiquidityOnly = 4u64,
    // pool only can add or remove liquidity and plan orders, can't swap
    OrderBookOnly = 5u64,
    // pool only can add or remove liquidity and swap, can't plan orders
    SwapOnly = 6u64,
    // pool status after created and will auto update to SwapOnly during swap after open_time
    WaitingTrade = 7u64,
}
impl AmmStatus {
    pub fn from_u64(status: u64) -> Self {
        match status {
            0u64 => AmmStatus::Uninitialized,
            1u64 => AmmStatus::Initialized,
            2u64 => AmmStatus::Disabled,
            3u64 => AmmStatus::WithdrawOnly,
            4u64 => AmmStatus::LiquidityOnly,
            5u64 => AmmStatus::OrderBookOnly,
            6u64 => AmmStatus::SwapOnly,
            7u64 => AmmStatus::WaitingTrade,
            _ => unreachable!(),
        }
    }

    pub fn into_u64(&self) -> u64 {
        match self {
            AmmStatus::Uninitialized => 0u64,
            AmmStatus::Initialized => 1u64,
            AmmStatus::Disabled => 2u64,
            AmmStatus::WithdrawOnly => 3u64,
            AmmStatus::LiquidityOnly => 4u64,
            AmmStatus::OrderBookOnly => 5u64,
            AmmStatus::SwapOnly => 6u64,
            AmmStatus::WaitingTrade => 7u64,
        }
    }
    pub fn valid_status(status: u64) -> bool {
        match status {
            1u64 | 2u64 | 3u64 | 4u64 | 5u64 | 6u64 | 7u64 => return true,
            _ => return false,
        }
    }

    pub fn deposit_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    pub fn withdraw_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => true,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    pub fn swap_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => false,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }

    pub fn orderbook_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => false,
            AmmStatus::WaitingTrade => false,
        }
    }
}