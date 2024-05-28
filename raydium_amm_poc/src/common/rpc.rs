use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey};

/**
 * get pool account
 *
 * # Arguments
 *
 * * 'client' - solana mainnet rpc url client
 * * 'amm_pool_key' - RaydiumV3 pool id
 *
 * # Returns
 */
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

/**
 * get multiple pool account
 *
 * # Arguments
 *
 * * 'client' - solana mainnet rpc url client
 * * 'pubkeys' - array of RaydiumV3 pool id
 *
 * # Returns
 */
pub fn get_multiple_accounts(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>> {
    Ok(client.get_multiple_accounts(pubkeys)?)
}
