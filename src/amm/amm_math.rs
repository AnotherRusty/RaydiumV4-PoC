use crate::amm;
use crate::common;
use amm::{openbooks, utils::AmmKeys, utils::CalculateMethod, utils::CalculateResult};
use anyhow::Result;
use arrayref::array_ref;
use common::rpc;
// use common::state::AmmInfo;
// use raydium_amm::state::AmmInfo;
// use raydium_amm::math::{CheckedCeilDiv, U128};
use safe_transmute::{to_bytes::transmute_to_bytes, transmute_one_pedantic};
use solana_client::rpc_client::RpcClient;
use solana_program::{account_info::IntoAccountInfo, program_pack::Pack};
use solana_sdk::{
    commitment_config::CommitmentConfig, message::Message, pubkey::Pubkey, transaction::Transaction,
};

pub const TEN_THOUSAND: u64 = 10000;

pub fn calculate_pool_vault_amounts(
    client: &RpcClient,
    amm_program: &Pubkey,
    amm_pool: &Pubkey,
    amm_keys: &AmmKeys,
    market_keys: &openbooks::MarketPubkeys,
    calculate_method: CalculateMethod,
) -> Result<CalculateResult> {
    let result = match calculate_method {
        CalculateMethod::CalculateWithLoadAccount => {
            // reload accounts data to calculate amm pool vault amount
            // get multiple accounts at the same time to ensure data consistency
            let load_pubkeys = vec![
                *amm_pool,
                amm_keys.amm_target,
                amm_keys.amm_pc_vault,
                amm_keys.amm_coin_vault,
                amm_keys.amm_open_order,
                amm_keys.market,
                *market_keys.event_q,
            ];
            let rsps = rpc::get_multiple_accounts(client, &load_pubkeys)?;
            let accounts = array_ref![rsps, 0, 7];
            let [amm_account, amm_target_account, amm_pc_vault_account, amm_coin_vault_account, amm_open_orders_account, market_account, market_event_q_account] =
                accounts;
            // let amm = transmute_one_pedantic::<raydium_amm::state::AmmInfo>(transmute_to_bytes(
            //     &amm_account.as_ref().unwrap().clone().data,
            // ))
            // .map_err(|e| e.without_src())?;
            let amm = rpc::get_account::<raydium_amm::state::AmmInfo>(client, &amm_pool)?.unwrap();
            let _amm_target: raydium_amm::state::TargetOrders =
                transmute_one_pedantic::<raydium_amm::state::TargetOrders>(transmute_to_bytes(
                    &amm_target_account.as_ref().unwrap().clone().data,
                ))
                .map_err(|e| e.without_src())?;
            let amm_pc_vault = spl_token::state::Account::unpack(
                &amm_pc_vault_account.as_ref().unwrap().clone().data,
            )
            .unwrap();
            let amm_coin_vault = spl_token::state::Account::unpack(
                &amm_coin_vault_account.as_ref().unwrap().clone().data,
            )
            .unwrap();
            let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
                if raydium_amm::state::AmmStatus::from_u64(amm.status).orderbook_permission() {
                    let amm_open_orders_account =
                        &mut amm_open_orders_account.as_ref().unwrap().clone();
                    let market_account = &mut market_account.as_ref().unwrap().clone();
                    let market_event_q_account =
                        &mut market_event_q_account.as_ref().unwrap().clone();

                    let amm_open_orders_info =
                        (&amm.open_orders, amm_open_orders_account).into_account_info();
                    let market_account_info = (&amm.market, market_account).into_account_info();
                    let market_event_queue_info =
                        (&(*market_keys.event_q), market_event_q_account).into_account_info();

                    let amm_authority = Pubkey::find_program_address(
                        &[raydium_amm::processor::AUTHORITY_AMM],
                        &amm_program,
                    )
                    .0;
                    let lamports = &mut 0;
                    let data = &mut [0u8];
                    let owner = Pubkey::default();
                    let amm_authority_info = solana_program::account_info::AccountInfo::new(
                        &amm_authority,
                        false,
                        false,
                        lamports,
                        data,
                        &owner,
                        false,
                        0,
                    );
                    let (market_state, open_orders) =
                        raydium_amm::processor::Processor::load_serum_market_order(
                            &market_account_info,
                            &amm_open_orders_info,
                            &amm_authority_info,
                            &amm,
                            false,
                        )?;
                    let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
                        raydium_amm::math::Calculator::calc_total_without_take_pnl(
                            amm_pc_vault.amount,
                            amm_coin_vault.amount,
                            &open_orders,
                            &amm,
                            &market_state,
                            &market_event_queue_info,
                            &amm_open_orders_info,
                        )?;
                    (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount)
                } else {
                    let (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount) =
                        raydium_amm::math::Calculator::calc_total_without_take_pnl_no_orderbook(
                            amm_pc_vault.amount,
                            amm_coin_vault.amount,
                            &amm,
                        )?;
                    (amm_pool_pc_vault_amount, amm_pool_coin_vault_amount)
                };

            // // deduct pnl
            // let (pool_pc_vault_without_pnl, pool_coin_vault_without_pnl) = pool_vault_deduct_pnl(
            //     amm_pool_pc_vault_amount,
            //     amm_pool_coin_vault_amount,
            //     &mut amm,
            //     &amm_target,
            // )?;
            CalculateResult {
                pool_pc_vault_amount: amm_pool_pc_vault_amount,
                pool_coin_vault_amount: amm_pool_coin_vault_amount,
                pool_lp_amount: amm.lp_amount,
                swap_fee_numerator: amm.fees.swap_fee_numerator,
                swap_fee_denominator: amm.fees.swap_fee_denominator,
            }
        }
        CalculateMethod::Simulate(fee_payer) => {
            let amm = rpc::get_account::<raydium_amm::state::AmmInfo>(client, amm_pool)?.unwrap();
            let simulate_pool_info_instruction = raydium_amm::instruction::simulate_get_pool_info(
                amm_program,
                amm_pool,
                &amm_keys.amm_authority,
                &amm_keys.amm_open_order,
                &amm_keys.amm_coin_vault,
                &amm_keys.amm_pc_vault,
                &amm_keys.amm_lp_mint,
                &amm_keys.market,
                &market_keys.event_q,
                None,
            )?;
            let mut message = Message::new(&[simulate_pool_info_instruction], Some(&fee_payer));
            message.recent_blockhash = client.get_latest_blockhash()?;
            let txn = Transaction::new_unsigned(message);
            let result =
                rpc::simulate_transaction(&client, &txn, false, CommitmentConfig::confirmed())?;
            // println!("{:#?}", result);
            let mut ret = raydium_amm::state::GetPoolData::default();
            if result.value.err.is_none() {
                if let Some(logs) = result.value.logs {
                    for log in logs {
                        if let Some(_) = log.find("GetPoolData: ") {
                            let begin = log.find("{").unwrap();
                            let end = log.rfind("}").unwrap() + 1;
                            let json_str = log.get(begin..end).unwrap();
                            ret = raydium_amm::state::GetPoolData::from_json(json_str)
                        }
                    }
                }
            }
            CalculateResult {
                pool_pc_vault_amount: ret.pool_pc_amount,
                pool_coin_vault_amount: ret.pool_coin_amount,
                pool_lp_amount: ret.pool_lp_supply,
                swap_fee_numerator: amm.fees.swap_fee_numerator,
                swap_fee_denominator: amm.fees.swap_fee_denominator,
            }
        }
    };
    Ok(result)
}
