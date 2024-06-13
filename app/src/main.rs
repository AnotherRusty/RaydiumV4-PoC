use anyhow::Result;
// use backoff;
use dotenv::dotenv;
use raydium_amm::{
    log::{InitLog, LogType},
    // math::{CheckedCeilDiv, U128},
    // processor,
    // state::{AmmStatus, TargetOrders},
};
use raydium_amm_poc::amm_math::{
    calc_coin_in_pc, calc_coin_market_cap, load_state, simulate_calc_swap_token_amount,
    swap_with_slippage, PoolState,
};
use raydium_amm_poc::raydium_amm::maths::SwapDirection;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::env;
use std::str::FromStr;
// use std::{collections::HashMap, env, str::FromStr, time::Duration};
// use yellowstone_grpc_client::{GeyserGrpcClient, GeyserGrpcClientError};
// use yellowstone_grpc_proto::prelude::{
//     SubscribeRequest, SubscribeRequestFilterAccounts, SubscribeRequestFilterBlocks,
//     SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterEntry, SubscribeRequestFilterSlots,
//     SubscribeRequestFilterTransactions,
// };

// type SlotsFilterMap = HashMap<String, SubscribeRequestFilterSlots>;
// type AccountFilterMap = HashMap<String, SubscribeRequestFilterAccounts>;
// type TransactionsFilterMap = HashMap<String, SubscribeRequestFilterTransactions>;
// type EntryFilterMap = HashMap<String, SubscribeRequestFilterEntry>;
// type BlocksFilterMap = HashMap<String, SubscribeRequestFilterBlocks>;
// type BlocksMetaFilterMap = HashMap<String, SubscribeRequestFilterBlocksMeta>;

#[derive(Debug)]
pub enum SimulationMode {
    MainNetMode,
    DevNetMode,
}

fn simulate_swap_tx(
    client: &RpcClient,
    amm_program_key: &Pubkey,
    state_coin_pc: &PoolState,
    wallet_pubkey: &Pubkey,
    slippage_bps: u64,
    in_amount_specific: u64,
    out_amount_specific: u64,
) -> Result<()> {
    let user_source = spl_associated_token_account::get_associated_token_address(
        &wallet_pubkey,
        &state_coin_pc.pool_amm_keys.amm_coin_mint,
    );
    let user_destination = spl_associated_token_account::get_associated_token_address(
        &wallet_pubkey,
        &state_coin_pc.pool_amm_keys.amm_pc_mint,
    );
    let amount_threshold_for_base_out = swap_with_slippage(
        state_coin_pc.pool.pool_pc_vault_amount,
        state_coin_pc.pool.pool_coin_vault_amount,
        state_coin_pc.pool.swap_fee_numerator,
        state_coin_pc.pool.swap_fee_denominator,
        SwapDirection::Coin2PC,
        in_amount_specific,
        true,
        slippage_bps,
    )?;

    let amount_threshold_for_base_in = swap_with_slippage(
        state_coin_pc.pool.pool_pc_vault_amount,
        state_coin_pc.pool.pool_coin_vault_amount,
        state_coin_pc.pool.swap_fee_numerator,
        state_coin_pc.pool.swap_fee_denominator,
        SwapDirection::Coin2PC,
        out_amount_specific,
        false,
        slippage_bps,
    )?;

    let _simulate_base_in = simulate_calc_swap_token_amount(
        &client,
        &amm_program_key,
        &state_coin_pc.pool_amm_keys,
        &state_coin_pc.pool_market_keys,
        &wallet_pubkey,
        &user_source,
        &user_destination,
        in_amount_specific,
        amount_threshold_for_base_out,
        true,
    )
    .unwrap();

    let _simulate_base_out = simulate_calc_swap_token_amount(
        &client,
        &amm_program_key,
        &state_coin_pc.pool_amm_keys,
        &state_coin_pc.pool_market_keys,
        &wallet_pubkey,
        &user_source,
        &user_destination,
        out_amount_specific,
        amount_threshold_for_base_in,
        false,
    )
    .unwrap();

    Ok(())
}

// async fn get_pool_info_() -> Result<()> {
//     // Configure and connect the gRPC client
//     let endpoint = env::var("ENDPOINT").expect("Error: gRPC endpoint variable not found");
//     let x_token = env::var("X_TOKNE").expect("Error: gRPC x_token variable not found");
//     let mut client = GeyserGrpcClient::connect_with_timeout(
//         endpoint,
//         Some(x_token),
//         None,
//         Some(Duration::from_secs(10)),
//         Some(Duration::from_secs(10)),
//         false,
//     )
//     .await
//     .map_err(|e| backoff::Error::transient(anyhow::Error::new(e)))
//     .unwrap();
//     // Create a subscription request for blocks
//     let subscribe_request = SubscribeRequest {
//         slots: HashMap::default(),
//         accounts: HashMap::default(),
//         transactions: HashMap::default(),
//         entry: HashMap::default(),
//         blocks: HashMap::default(),
//         blocks_meta: HashMap::default(),
//         commitment: None,
//         accounts_data_slice: Vec::default(),
//         ping: None,
//     };

//     let (mut subscribe_tx, mut stream) = client.subscribe().await?;
//     subscribe_tx
//         .send(subscribe_request)
//         .await
//         .map_err(GeyserGrpcClientError::SubscribeSendError)?;
//     while let Some(message) = stream.next().await {
//         match message {
//             Ok(msg) =>
//             {
//                 #[allow(clippy::single_match)]
//                 match msg.update_oneof {
//                     Some(UpdateOneof::Transaction(tx)) => {
//                         let tx: TransactionPretty = tx.into();
//                         info!(
//                             "new transaction update: filters {:?}, transaction: {:#?}",
//                             msg.filters, tx
//                         );
//                         continue;
//                     }
//                     _ => {}
//                 }
//             }
//             Err(error) => {
//                 break;
//             }
//         }
//     }

//     Ok(())
// }

fn fetch_pool_info(
    client: &RpcClient,
    amm_program_key: &Pubkey,
    coin_pc_pool: &Pubkey,
) -> Result<PoolState> {
    let sol_usdt_pool: Pubkey = Pubkey::from_str("7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX")?;
    let state_sol_usd = load_state(&client, &amm_program_key, &sol_usdt_pool).unwrap();

    let state_coin_pc = load_state(&client, &amm_program_key, &coin_pc_pool).unwrap();

    let coin_in_pc = calc_coin_in_pc(&state_coin_pc.pool).unwrap();
    let sol_in_usd = calc_coin_in_pc(&state_sol_usd.pool).unwrap();
    println!("Sol Price is {} USD", sol_in_usd);

    let coin_in_usd = coin_in_pc * sol_in_usd;
    println!("Coin Price is {} USD", coin_in_usd);

    let total_supply = calc_coin_market_cap(&state_coin_pc, &client).unwrap();
    println!("Total Supply is {}", total_supply);

    let mcap = total_supply * coin_in_usd;
    println!("MarkteCap is {} USD", mcap);

    Ok(state_coin_pc)
}

fn listen_for_new_pools(url: &String, addresses: Vec<String>) -> Result<()> {
    let filter = RpcTransactionLogsFilter::Mentions(addresses);
    let config = RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::finalized()),
    };
    let (_pubsub_client_subscription, log_receiver) =
        PubsubClient::logs_subscribe(url, filter, config).unwrap();

    for log_response in log_receiver {
        let logs: Vec<String> = log_response.value.logs;
        if let Some(ray_log_entry) = logs.iter().find(|log| log.contains("ray_log:")) {
            // Extract the ray_log value
            if let Some(start) = ray_log_entry.find("ray_log:") {
                let ray_log_value = &ray_log_entry[start + "ray_log: ".len()..];
                let bytes = base64::decode_config(ray_log_value, base64::STANDARD).unwrap();
                let log_type = LogType::from_u8(bytes[0]);
                if log_type.into_u8() == LogType::Init.into_u8() {
                    let log: InitLog = bincode::deserialize(&bytes).unwrap();
                    println!("{:?}", log);
                }
            }
        }
    }

    Ok(())
}

fn listen_for_new_block(
    url: &String,
    client: &RpcClient,
    amm_program_key: &Pubkey,
    coin_pc_pool: &Pubkey,
) -> Result<()> {
    let (_pubsub_client_subscription, slot_info) = PubsubClient::slot_subscribe(url).unwrap();

    for _slot in slot_info {
        let _ = fetch_pool_info(&client, &amm_program_key, &coin_pc_pool);
    }

    Ok(())
}

fn main() -> Result<()> {
    let slippage_bps = 50u64; // 0.5%
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let client: RpcClient = RpcClient::new(rpc_url.to_string());
    let web_socket_url = env::var("WEB_SOCKET_URL").expect("WEB_SOCKET_URL must be set");
    let amm_program_key: Pubkey = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?;
    let coin_pc_pool: Pubkey = Pubkey::from_str("879F697iuDJGMevRkRcnW21fcXiAeLJK1ffsw2ATebce")?; // MEW/SOL pool
    let wallet_pubkey: Pubkey = Pubkey::from_str("EccxYg7rViwYfn9EMoNu7sUaV82QGyFt6ewiQaH1GYjv")?;
    let in_amount_specific: u64 = 10000000000; // this value includes decimals
    let out_amount_specific: u64 = 10000000000; // this value includes decimals
    let mut listener_list: Vec<String> = Vec::new();

    println!("===================Fetch Pool Info===================");
    let state_coin_pc = fetch_pool_info(&client, &amm_program_key, &coin_pc_pool)?;

    println!("===================Simulation Swap Tx===================");
    let _ = simulate_swap_tx(
        &client,
        &amm_program_key,
        &state_coin_pc,
        &wallet_pubkey,
        slippage_bps,
        in_amount_specific,
        out_amount_specific,
    );

    println!("===================Listen for new pools===================");
    listener_list.push(amm_program_key.to_string());
    let _ = listen_for_new_pools(&web_socket_url, listener_list);

    println!("===================Listen for new blocks===================");
    let _ = listen_for_new_block(&web_socket_url, &client, &amm_program_key, &coin_pc_pool);
    Ok(())
}
