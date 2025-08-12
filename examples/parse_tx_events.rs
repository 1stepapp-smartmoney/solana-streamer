use anyhow::Result;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_streamer_sdk::streaming::event_parser::{
    protocols::MutilEventParser, EventParser, Protocol,
};
use std::str::FromStr;
use std::sync::Arc;

/// Get transaction data based on transaction signature
#[tokio::main]
async fn main() -> Result<()> {
    let signatures = vec![
        "42agNk1heHabNAVRzEKqQEt5adGkQzRYf9M1Q81uBJPCCHyP4cyCA1RNkgxXrtEAWeeGcytyh2TsnkBDgqnHeq4z",
    ];
    // Validate signature format
    let mut valid_signatures = Vec::new();
    for sig_str in &signatures {
        match solana_sdk::signature::Signature::from_str(sig_str) {
            Ok(_) => valid_signatures.push(*sig_str),
            Err(e) => println!("Invalid signature format: {}", e),
        }
    }
    if valid_signatures.is_empty() {
        println!("No valid transaction signatures");
        return Ok(());
    }
    for signature in valid_signatures {
        println!("Starting transaction parsing: {}", signature);
        get_single_transaction_details(signature).await?;
        println!("Transaction parsing completed: {}\n\n\n", signature);
    }

    Ok(())
}

/// Get details of a single transaction
async fn get_single_transaction_details(signature_str: &str) -> Result<()> {
    use solana_sdk::signature::Signature;
    use solana_transaction_status::UiTransactionEncoding;

    let signature = Signature::from_str(signature_str)?;

    // Create Solana RPC client
    let rpc_url = "https://api.mainnet-beta.solana.com";
    println!("Connecting to Solana RPC: {}", rpc_url);

    let client = solana_client::nonblocking::rpc_client::RpcClient::new(rpc_url.to_string());

    match client
        .get_transaction_with_config(
            &signature,
            solana_client::rpc_config::RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Binary),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )
        .await
    {
        Ok(transaction) => {
            println!("Transaction signature: {}", signature_str);
            println!("Block slot: {}", transaction.slot);

            if let Some(block_time) = transaction.block_time {
                println!("Block time: {}", block_time);
            }

            if let Some(meta) = &transaction.transaction.meta {
                println!("Transaction fee: {} lamports", meta.fee);
                println!("Status: {}", if meta.err.is_none() { "Success" } else { "Failed" });
                if let Some(err) = &meta.err {
                    println!("Error details: {:?}", err);
                }
                // Compute units consumed
                if let solana_transaction_status::option_serializer::OptionSerializer::Some(units) =
                    &meta.compute_units_consumed
                {
                    println!("Compute units consumed: {}", units);
                }
                // Display logs (all)
                if let solana_transaction_status::option_serializer::OptionSerializer::Some(logs) =
                    &meta.log_messages
                {
                    println!("Transaction logs (all {} entries):", logs.len());
                    for (i, log) in logs.iter().enumerate() {
                        println!("  [{}] {}", i + 1, log);
                    }
                }
            }
            let protocols = vec![
                Protocol::Bonk,
                Protocol::RaydiumClmm,
                Protocol::PumpSwap,
                Protocol::PumpFun,
                Protocol::RaydiumCpmm,
                Protocol::RaydiumAmmV4,
            ];
            let parser: Arc<dyn EventParser> = Arc::new(MutilEventParser::new(protocols));
            let start_time = std::time::Instant::now();
            let events = parser
                .parse_transaction(
                    transaction.transaction.clone(),
                    &signature.to_string(),
                    Some(transaction.slot),
                    None,
                    0,
                    None,
                )
                .await
                .unwrap_or_else(|_e| vec![]);

            let end_time = std::time::Instant::now();
            let duration = end_time.duration_since(start_time);
            println!("Parsing time: {:?}", duration);
            for event in events {
                println!("{:?}\n", event);
            }
        }
        Err(e) => {
            println!("Failed to get transaction: {}", e);
        }
    }

    Ok(())
}
