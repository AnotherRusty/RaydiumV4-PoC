use anyhow::Result;
use solana_client::{
    rpc_client::RpcClient,
    rpc_request::RpcRequest,
    rpc_response::{RpcResult, RpcSimulateTransactionResult},
};
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey, transaction::Transaction,
};

use solana_transaction_status::UiTransactionEncoding;

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

pub fn get_multiple_accounts(
    client: &RpcClient,
    pubkeys: &[Pubkey],
) -> Result<Vec<Option<Account>>> {
    Ok(client.get_multiple_accounts(pubkeys)?)
}

pub fn simulate_transaction(
    client: &RpcClient,
    transaction: &Transaction,
    sig_verify: bool,
    cfg: CommitmentConfig,
) -> RpcResult<RpcSimulateTransactionResult> {
    let serialized_encoded = base64::encode(bincode::serialize(transaction).unwrap());
    client.send(
        RpcRequest::SimulateTransaction,
        serde_json::json!([serialized_encoded, {
            "sigVerify": sig_verify, "commitment": cfg.commitment, "encoding": Some(UiTransactionEncoding::Base64)
        }]),
    )
}
