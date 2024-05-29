use solana_sdk::pubkey::Pubkey;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum SwapDirection {
    /// Input token pc, output token coin
    PC2Coin = 1u64,
    /// Input token coin, output token pc
    Coin2PC = 2u64,
}

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

#[derive(Debug)]
pub struct MarketPubkeys {
    pub market: Box<Pubkey>,
    pub req_q: Box<Pubkey>,
    pub event_q: Box<Pubkey>,
    pub bids: Box<Pubkey>,
    pub asks: Box<Pubkey>,
    pub coin_vault: Box<Pubkey>,
    pub pc_vault: Box<Pubkey>,
    pub vault_signer_key: Box<Pubkey>,
    pub coin_mint: Box<Pubkey>,
    pub pc_mint: Box<Pubkey>,
    pub coin_lot_size: u64,
    pub pc_lot_size: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct CalculateResult {
    pub pool_pc_vault_amount: u64,
    pub pool_coin_vault_amount: u64,
    pub pool_lp_amount: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
}

#[cfg_attr(feature = "client", derive(Debug))]
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
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
pub struct PoolTokenPairResult {
    pub base_toke_price: f64,
    pub liquidity: f64,
}
