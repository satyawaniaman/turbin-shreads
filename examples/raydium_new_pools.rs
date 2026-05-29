//! Usage:
//!   cargo run --example raydium_new_pools -- --bind 0.0.0.0:8001

use clap::Parser;
use shredstream_decoder_example::{Dex, InstructionKind, ShredPipeline};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:8001")]
    bind: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let args = Args::parse();

    let mut pipeline = ShredPipeline::new(args.bind.clone());
    pipeline = pipeline
        .with_dex_filter(vec![Dex::RaydiumAmm, Dex::RaydiumCpmm])
        .with_kind_filter(vec![InstructionKind::PoolInit])
        .on_instruction(Box::new(|inst| {
            println!(
                "NEW POOL [slot {}] {} | mint={} | pair=({}, {}) | sig={}",
                inst.slot,
                inst.dex,
                inst.mint,
                inst.input_mint.map(|m| m.to_string()).unwrap_or_default(),
                inst.output_mint.map(|m| m.to_string()).unwrap_or_default(),
                inst.signature,
            );
        }));

    tracing::info!(bind = %args.bind, "Watching for new Raydium pools...");
    pipeline.run().await
}
