use std::{env, sync::Arc, vec};

use cainome::{
    cairo_serde::{ContractAddress, U256},
    rs::abigen,
};
use starknet::{
    accounts::{Account, Call, SingleOwnerAccount},
    core::types::FieldElement,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};

use crate::{trade::utils::get_set_primary_price_call, types::SatoruAction};

abigen!(
    OrderHandler,
    "./resources/satoru_OrderHandler.contract_class.json",
);

abigen!(
    DataStore,
    "./resources/satoru_DataStore.contract_class.json",
    type_aliases {
        satoru::order::order::OrderType as Order_;
        satoru::data::data_store::DataStore::Event as Event_;
        satoru::order::order::DecreasePositionSwapType as Decrease_;
        satoru::utils::span32::Span32 as Span32_;
    }
);

abigen!(
    Oracle,
    "./resources/satoru_Oracle.contract_class.json",
    type_aliases {
        satoru::price::price::Price as Price_;
        satoru::oracle::oracle::Oracle::Event as Event__;
    }
);

pub async fn handle_order(
    account: Arc<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
    order: SatoruAction,
) {
    let set_price_call = get_set_primary_price_call(order.clone(), account.clone()).await;

    let execute_order_call = get_execute_order_call(order, account.clone());

    let _order_execution_multicall = account
        .execute(vec![set_price_call, execute_order_call])
        .send()
        .await
        .expect("Order execution multicall failed");
    // TODO: poll transaction status
}

fn get_execute_order_call(
    order: SatoruAction,
    account: Arc<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>>,
) -> Call {
    let order_handler_address =
        env::var("ORDER_HANDLER").expect("ORDER_HANDLER env variable not set");
    let order_handler = OrderHandler::new(
        FieldElement::from_hex_be(&order_handler_address)
            .expect("Conversion error: order_handler_address"),
        account.clone(),
    );

    let set_prices_params: SetPricesParams = SetPricesParams {
        signer_info: U256 { low: 1, high: 0 },
        tokens: vec![
            ContractAddress::from(
                FieldElement::from_hex_be("0x").expect("Cannot convert string to felt"),
            ),
            ContractAddress::from(
                FieldElement::from_hex_be("0x").expect("Cannot convert string to felt"),
            ),
        ],
        compacted_min_oracle_block_numbers: vec![63970, 63970],
        compacted_max_oracle_block_numbers: vec![64901, 64901],
        compacted_oracle_timestamps: vec![171119803, 10],
        compacted_decimals: vec![U256 { low: 1, high: 0 }, U256 { low: 1, high: 0 }],
        compacted_min_prices: vec![U256 {
            low: 2147483648010000,
            high: 0,
        }],
        compacted_min_prices_indexes: vec![U256 { low: 0, high: 0 }],
        compacted_max_prices: vec![U256 {
            low: 2147483648010000,
            high: 0,
        }],
        compacted_max_prices_indexes: vec![U256 { low: 0, high: 0 }],
        signatures: vec![
            vec![
                FieldElement::from_hex_be("0x").expect("Cannot convert string to felt"),
                FieldElement::from_hex_be("0x").expect("Cannot convert string to felt"),
            ],
            vec![
                FieldElement::from_hex_be("0x").expect("Cannot convert string to felt"),
                FieldElement::from_hex_be("0x").expect("Cannot convert string to felt"),
            ],
        ],
        price_feed_tokens: vec![],
    };

    order_handler.execute_order_getcall(
        &FieldElement::from_hex_be(&order.key).expect("Cannot convert string to felt"),
        &set_prices_params,
    )
}
