// use chrono::prelude::*;
use std::env;
use web3::types::{ BlockId, BlockNumber };

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
        "block number {}, parent: {:?}, transactions: {}, gas used: {:?}, gas limit : {:?}",
        latest_block.number.unwrap(),
        latest_block.parent_hash,
        latest_block.transactions.len(),
        latest_block.gas_used,
        latest_block.gas_limit

       
    );
}