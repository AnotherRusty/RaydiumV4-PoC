use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey};

pub fn get_account<T>(client: &RpcClient, addr: &Pubkey) -> Result<Option<T>>
where
    T: Clone,
{
    if let Some(account) = client
        .get_account_with_commitment(addr, CommitmentConfig::processed())?
        .value
    {
        let account_data = account.data.as_slice();
        let ret = unsafe { &*(&account_data[0] as *const u8 as *const T) };
        Ok(Some(ret.clone()))
    } else {
        Ok(None)
    }
}

pub fn get_multiple_accounts(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>> {
    Ok(client.get_multiple_accounts(pubkeys)?)
}

// pub fn get_whirlpool_data(pubkey_string: &String, account_map: &AccountMap) -> Whirlpool {
//     let data = account_map.get(pubkey_string).unwrap();
//     let whirlpool_data =
//         whirlpool_base::state::Whirlpool::try_deserialize(&mut data.as_slice()).unwrap();
//     return whirlpool_data;
// }