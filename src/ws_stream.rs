use crate::decoder::DecoderRegistry;
use crate::types::DecodedInstruction;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashSet;

const PROGRAM_IDS: &[&str] = &[
    "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",  // PumpFun
    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",  // Jupiter V6
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", // Raydium AMM
    "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C", // Raydium CPMM
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",   // SPL Token
];

/// Connect to Solana WebSocket, subscribe to logs, decode transactions
pub async fn run_ws_stream(
    rpc_url: &str,
    on_instruction: impl Fn(&DecodedInstruction) + Send + Sync + 'static,
) -> anyhow::Result<()> {
    let registry = DecoderRegistry::new();
    let ws_url = rpc_url.replace("https://", "wss://");
    
    // We only want to process a transaction once, even if it's mentioned multiple times
    let mut seen_signatures = HashSet::new();

    loop {
        tracing::info!("Connecting to Solana WebSocket...");
        
        let connect_result = connect_async(&ws_url).await;
        if let Err(e) = connect_result {
            tracing::error!("WebSocket connection failed: {}. Retrying in 3s...", e);
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            continue;
        }
        
        let (mut ws, _) = connect_result.unwrap();

        // Subscribe to logs for each program
        for (i, pid) in PROGRAM_IDS.iter().enumerate() {
            let msg = serde_json::json!({
                "jsonrpc": "2.0",
                "id": i + 1,
                "method": "logsSubscribe",
                "params": [
                    { "mentions": [pid] },
                    { "commitment": "confirmed" }
                ]
            });
            ws.send(Message::Text(msg.to_string().into())).await?;
        }

        tracing::info!("WebSocket subscribed to {} programs", PROGRAM_IDS.len());

        // Process incoming notifications
        while let Some(msg_result) = ws.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    let v: serde_json::Value = match serde_json::from_str(&text) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    // Extract signature from logsSubscribe notification
                    let signature = v["params"]["result"]["value"]["signature"]
                        .as_str()
                        .map(|s| s.to_string());

                    if let Some(sig) = signature {
                        if !seen_signatures.insert(sig.clone()) {
                            continue; // Already processed
                        }
                        
                        // Prevent the set from growing indefinitely
                        if seen_signatures.len() > 1000 {
                            seen_signatures.clear();
                        }
                        
                        // Fetch full transaction via RPC
                        match fetch_and_decode(rpc_url, &sig, &registry).await {
                            Ok(decoded) => {
                                for inst in &decoded {
                                    on_instruction(inst);
                                }
                            }
                            Err(e) => {
                                tracing::error!("fetch_and_decode error for {}: {}", sig, e);
                            }
                        }
                    } else if v.get("method").is_some() {
                        tracing::debug!("WebSocket notification without signature: {}", text);
                    }
                },
                Ok(Message::Ping(ping)) => {
                    // Respond to keepalive pings
                    let _ = ws.send(Message::Pong(ping)).await;
                },
                Ok(Message::Close(_)) => {
                    break;
                },
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                },
                _ => {}
            }
        }

        tracing::warn!("WebSocket disconnected. Reconnecting in 3s...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}

/// Fetch a transaction by signature and decode it
async fn fetch_and_decode(
    rpc_url: &str,
    signature: &str,
    registry: &DecoderRegistry,
) -> anyhow::Result<Vec<DecodedInstruction>> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [signature, {
            "encoding": "base64",
            "commitment": "confirmed",
            "maxSupportedTransactionVersion": 0
        }]
    });

    let client = reqwest::Client::new();
    let resp = client.post(rpc_url)
        .json(&body)
        .send().await?
        .json::<serde_json::Value>().await?;

    let result = &resp["result"];
    if result.is_null() { return Ok(vec![]); }

    let slot = result["slot"].as_u64().unwrap_or(0);

    // Decode base64 transaction data
    let tx_data = result["transaction"][0].as_str().unwrap_or("");
    let tx_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD, tx_data
    )?;
    
    // We expect a VersionedTransaction but we must handle it gracefully
    let tx: solana_sdk::transaction::VersionedTransaction = bincode::deserialize(&tx_bytes)?;

    let accounts = tx.message.static_account_keys();
    let instructions: Vec<(u8, Vec<u8>, Vec<u8>)> = tx.message
        .instructions()
        .iter()
        .map(|ix| (ix.program_id_index, ix.accounts.clone(), ix.data.clone()))
        .collect();

    Ok(registry.decode_transaction(accounts, &instructions, signature, slot))
}
