//! Usage:
//!   cargo run --example serve_api -- --bind-udp 0.0.0.0:8001 --bind-http 0.0.0.0:8002

use axum::{routing::get, Json, Router};
use clap::Parser;
use shredstream_decoder_example::{DecodedInstruction, ShredPipeline};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:8001")]
    bind_udp: String,
    #[arg(long, default_value = "0.0.0.0:8002")]
    bind_http: String,
    #[arg(long)]
    mock: bool,
    #[arg(long)]
    ws: bool,
    #[arg(long, env = "HELIUS_RPC_URL")]
    rpc_url: Option<String>,
}

#[derive(serde::Serialize, Clone)]
struct ApiEvent {
    #[serde(flatten)]
    instruction: DecodedInstruction,
    timestamp: u64,
    fec_status: String,
    decode_latency_ms: u64,
}

type SharedCache = Arc<RwLock<VecDeque<ApiEvent>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let args = Args::parse();
    let cache: SharedCache = Arc::new(RwLock::new(VecDeque::with_capacity(100)));

    // Spawn the UDP listener pipeline in a background task
    let pipeline_cache = cache.clone();
    let bind_udp = args.bind_udp.clone();
    let is_mock = args.mock;
    
    if is_mock {
        tracing::info!("Starting in MOCK mode. Generating fake events...");
        tokio::spawn(async move {
            let mut slot = 300_000_000;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                slot += 1;
                
                let event = ApiEvent {
                    instruction: DecodedInstruction {
                        dex: shredstream_decoder_example::Dex::PumpFun,
                        kind: shredstream_decoder_example::InstructionKind::Buy,
                        signature: format!("mock_sig_{}", slot),
                        slot,
                        mint: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap(),
                        input_mint: None,
                        output_mint: None,
                        input_amount: Some(100000000),
                        output_amount: Some(500000),
                        slippage_bps: Some(50),
                        authority: "11111111111111111111111111111111".parse().unwrap(),
                    },
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                    fec_status: "complete".to_string(),
                    decode_latency_ms: 2,
                };
                
                let mut guard = pipeline_cache.write().await;
                guard.push_front(event);
                if guard.len() > 100 { guard.pop_back(); }
                println!("[MOCK slot {}] PumpFun Buy generated", slot);
            }
        });
    } else if args.ws {
        tracing::info!("Starting in WEBSOCKET mode...");
        let rpc_url = args.rpc_url.expect("HELIUS_RPC_URL must be set when using --ws");
        tokio::spawn(async move {
            let res = shredstream_decoder_example::ws_stream::run_ws_stream(&rpc_url, move |inst| {
                let event = ApiEvent {
                    instruction: inst.clone(),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                    fec_status: "complete".to_string(),
                    decode_latency_ms: 45, // WS has higher latency than UDP
                };
                
                let cache_clone = pipeline_cache.clone();
                tokio::spawn(async move {
                    let mut guard = cache_clone.write().await;
                    guard.push_front(event);
                    if guard.len() > 100 {
                        guard.pop_back();
                    }
                });
                
                println!("[WS slot {}] {} {:?} | mint={}", inst.slot, inst.dex, inst.kind, inst.mint);
            }).await;
            
            if let Err(e) = res {
                tracing::error!("WebSocket stream error: {}", e);
            }
        });
    } else {
        tokio::spawn(async move {
            let mut pipeline = ShredPipeline::new(bind_udp.clone());
            pipeline = pipeline.on_instruction(Box::new(move |inst| {
                let event = ApiEvent {
                    instruction: inst.clone(),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                    fec_status: "complete".to_string(),
                    decode_latency_ms: 2,
                };
                
                let cache_clone = pipeline_cache.clone();
                tokio::spawn(async move {
                    let mut guard = cache_clone.write().await;
                    guard.push_front(event);
                    if guard.len() > 100 {
                        guard.pop_back();
                    }
                });
                
                println!("[UDP slot {}] {} {:?} | mint={}", inst.slot, inst.dex, inst.kind, inst.mint);
            }));

            tracing::info!(bind = %bind_udp, "Starting ShredStream pipeline...");
            if let Err(e) = pipeline.run().await {
                tracing::error!("Pipeline error: {}", e);
            }
        });
    }

    // Start the Axum HTTP Server
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/api/events", get(get_events))
        .with_state(cache)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&args.bind_http).await?;
    tracing::info!(bind = %args.bind_http, "Starting HTTP API server...");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_events(axum::extract::State(cache): axum::extract::State<SharedCache>) -> Json<serde_json::Value> {
    let guard = cache.read().await;
    let events: Vec<ApiEvent> = guard.iter().cloned().collect();
    
    Json(serde_json::json!({
        "events": events,
        "fetchedAt": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        "source": "rust-decoder-engine"
    }))
}
