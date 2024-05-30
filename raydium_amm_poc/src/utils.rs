use crate::types::{AmmInfo, AmmKeys, MarketPubkeys};
use anyhow::{format_err, Result};
use raydium_amm::{processor, processor::Processor};
use safe_transmute::{
    to_bytes::{transmute_one_to_bytes, transmute_to_bytes},
    transmute_many_pedantic, transmute_one_pedantic,
};
use serum_dex::state::{gen_vault_signer_key, AccountFlag, Market, MarketState, MarketStateV2};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::{
    borrow::Cow,
    convert::{identity, TryFrom},
};

pub fn get_multiple_accounts(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>> {
    Ok(client.get_multiple_accounts(pubkeys)?)
}

pub fn get_account<T>(client: &RpcClient, amm_pool_key: &Pubkey) -> Result<Option<T>>
where
    T: Clone,
{
    if let Some(account) = client
        .get_account_with_commitment(amm_pool_key, CommitmentConfig::processed())?
        .value
    {
        let account_data = account.data.as_slice();
        let ret = unsafe { &*(&account_data[0] as *const u8 as *const T) };
        Ok(Some(ret.clone()))
    } else {
        Ok(None)
    }
}

pub fn get_amm_keys(
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

#[cfg(target_endian = "little")]
fn remove_dex_account_padding<'a>(data: &'a [u8]) -> Result<Cow<'a, [u64]>> {
    use serum_dex::state::{ACCOUNT_HEAD_PADDING, ACCOUNT_TAIL_PADDING};
    let head = &data[..ACCOUNT_HEAD_PADDING.len()];
    if data.len() < ACCOUNT_HEAD_PADDING.len() + ACCOUNT_TAIL_PADDING.len() {
        return Err(format_err!(
            "dex account length {} is too small to contain valid padding",
            data.len()
        ));
    }
    if head != ACCOUNT_HEAD_PADDING {
        return Err(format_err!("dex account head padding mismatch"));
    }
    let tail = &data[data.len() - ACCOUNT_TAIL_PADDING.len()..];
    if tail != ACCOUNT_TAIL_PADDING {
        return Err(format_err!("dex account tail padding mismatch"));
    }
    let inner_data_range = ACCOUNT_HEAD_PADDING.len()..(data.len() - ACCOUNT_TAIL_PADDING.len());
    let inner: &'a [u8] = &data[inner_data_range];
    let words: Cow<'a, [u64]> = match transmute_many_pedantic::<u64>(inner) {
        Ok(word_slice) => Cow::Borrowed(word_slice),
        Err(transmute_error) => {
            let word_vec = transmute_error.copy().map_err(|e| e.without_src())?;
            Cow::Owned(word_vec)
        }
    };
    Ok(words)
}

#[cfg(target_endian = "little")]
pub fn get_keys_for_market<'a>(
    client: &'a RpcClient,
    market_key: &'a Pubkey,
    market: &'a Pubkey,
) -> Result<MarketPubkeys> {
    let account_data: Vec<u8> = client.get_account_data(&market)?;
    let words: Cow<[u64]> = remove_dex_account_padding(&account_data)?;
    let market_state: MarketState = {
        let account_flags = Market::account_flags(&account_data)?;
        if account_flags.intersects(AccountFlag::Permissioned) {
            // println!("MarketStateV2");
            let state = transmute_one_pedantic::<MarketStateV2>(transmute_to_bytes(&words))
                .map_err(|e| e.without_src())?;
            state.check_flags(true)?;
            state.inner
        } else {
            // println!("MarketStateV1");
            let state: MarketState =
                transmute_one_pedantic::<MarketState>(transmute_to_bytes(&words))
                    .map_err(|e| e.without_src())?;
            state.check_flags(true)?;
            state
        }
    };
    let vault_signer_key: Pubkey =
        gen_vault_signer_key(market_state.vault_signer_nonce, market, market_key)?;
    assert_eq!(
        transmute_to_bytes(&identity(market_state.own_address)),
        market.as_ref()
    );
    Ok(MarketPubkeys {
        market: Box::new(*market),
        req_q: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.req_q))).unwrap(),
        ),
        event_q: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.event_q))).unwrap(),
        ),
        bids: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.bids))).unwrap(),
        ),
        asks: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.asks))).unwrap(),
        ),
        coin_vault: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.coin_vault))).unwrap(),
        ),
        pc_vault: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.pc_vault))).unwrap(),
        ),
        vault_signer_key: Box::new(vault_signer_key),
        coin_mint: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.coin_mint))).unwrap(),
        ),
        pc_mint: Box::new(
            Pubkey::try_from(transmute_one_to_bytes(&identity(market_state.pc_mint))).unwrap(),
        ),
        coin_lot_size: market_state.coin_lot_size,
        pc_lot_size: market_state.pc_lot_size,
    })
}
