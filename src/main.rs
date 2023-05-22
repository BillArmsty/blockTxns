// use chrono::prelude::*;
use std::env;
use web3::helpers as w3h;
use web3::types::H160;
use web3::types::{ BlockId, BlockNumber, TransactionId, U256, U64 };

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let websocket = web3::transports::WebSocket
        ::new(&env::var("INFURA_MAIN").unwrap())
        .await
        .unwrap();
    let web3s = web3::Web3::new(websocket);

    let latest_block = web3s
        .eth()
        .block(BlockId::Number(BlockNumber::Latest))
        .await
        .unwrap()
        .unwrap();

    println!(
        "block number {}, parent: {:?}, transactions: {}, gas used: {:?}, gas limit : {:?},  ",
        latest_block.number.unwrap(),
        latest_block.parent_hash,
        latest_block.transactions.len(),
        latest_block.gas_used,
        latest_block.gas_limit

       
    );

    for transaction_hash in latest_block.transactions {
        let tx = match web3s
            .eth()
            .transaction(TransactionId::Hash(transaction_hash))
            .await

        {
            Ok(Some(tx)) => tx,
            _=> {
                println!("Error fetching transaction {:?}", transaction_hash);
                continue;
            }
        };

        let from_addr = tx.from.unwrap_or(H160::zero());
        let to_addr = tx.to.unwrap_or(H160::zero());
        let eth_value = wei_to_eth(tx.value);
        println!(
            "[{}] from: {:?}, to: {:?}, value: {:?}, gas: {:?}, gas price: {:?}",
            tx.transaction_index.unwrap_or(U64::from(0)),
            w3h::to_string(&from_addr),
            w3h::to_string(&to_addr),
            eth_value,
            tx.gas,
            tx.gas_price 
        );
    }
}

fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    let res = res / 1_000_000_000_000_000_000.0;
    res
}