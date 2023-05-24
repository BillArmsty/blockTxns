// use chrono::prelude::*;
use std::env;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use web3::helpers as w3h;
use web3::contract::{ Contract, Options };
use web3::types::H160;
use web3::types::{ BlockId, BlockNumber, TransactionId, U256, U64 };

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    //Deserializing the JSON file to a BTreeMap instance
    let file = File::open("src/signatures.json").unwrap();
    let reader = BufReader::new(file);
    let code_sig_lookup: BTreeMap<String, Vec<String>> = serde_json::from_reader(reader).unwrap();

    let websocket = web3::transports::WebSocket
        ::new(&env::var("INFURA_MAIN").unwrap()).await
        .unwrap();
    let web3s = web3::Web3::new(websocket);

    let latest_block = web3s
        .eth()
        .block(BlockId::Number(BlockNumber::Latest)).await
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
        let tx = match web3s.eth().transaction(TransactionId::Hash(transaction_hash)).await {
            Ok(Some(tx)) => tx,
            _ => {
                println!("Error fetching transaction {:?}", transaction_hash);
                continue;
            }
        };

        //Determining if an address is a valid smart contract address
        let smart_contract_addr = match tx.to {
            Some(addr) =>
                match web3s.eth().code(addr, None).await {
                    Ok(code) => {
                        if code == web3::types::Bytes::from([]) {
                            println!("No code at address , skipping");
                            continue;
                        } else {
                            println!("Code found at address ");
                            addr
                        }
                    }
                    _ => {
                        println!("Error fetching code at address skipping.");
                        continue;
                    }
                }
            _ => {
                println!("To address is not a valid address, skipping.");
                continue;
            }
        };

        //Invoking the name function to get the token name

        let smart_contract = Contract::from_json(
            web3s.eth(),
            smart_contract_addr,
            include_bytes!("erc20_abi.json")
        );

        let token_name: String = match
            smart_contract
                .expect("Failed to get smart contract")
                .query("name", (), None, Options::default(), None).await
        {
            Ok(result) => result,
            Err(error) => {
                println!("Error fetching token name, skipping. {:?}", error);
                continue;
            }
        };
        let from_addr = tx.from.unwrap_or(H160::zero());
        let to_addr = tx.to.unwrap_or(H160::zero());
        let eth_value = wei_to_eth(tx.value);

        //Look up token transaction function signatures
        let input_str: String = w3h::to_string(&tx.input);
        if input_str.len() < 12 {
            continue;
        }
        let func_code = input_str[3..11].to_string();
        let func_signature: String = match code_sig_lookup.get(&func_code) {
            Some(func_sig) => format!("{:?}", func_sig),
            _ => {
                println!("No function signature found for code");
                "[unknown]".to_string()
            }
        };

        println!(
            "[{}] ({} -> {}) from: {}, to: {}, value: {}, gas: {}",
            tx.transaction_index.unwrap_or(U64::from(0)),
            &token_name,
            &func_signature,
            w3h::to_string(&from_addr),
            w3h::to_string(&to_addr),
            eth_value,
            tx.gas
        );
    }
}

fn wei_to_eth(wei_val: U256) -> f64 {
    let res = wei_val.as_u128() as f64;
    let res = res / 1_000_000_000_000_000_000.0;
    res
}