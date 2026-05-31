//! Usage:
//!   cargo run --example token_mints -- --bind 0.0.0.0:8001

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
        .with_dex_filter(vec![Dex::SplToken])
        .with_kind_filter(vec![InstructionKind::MintTo])
        .on_instruction(Box::new(|inst| {
            println!(
                "MINT [slot {}] mint={} amount={:?} authority={} | sig={}",
                inst.slot, inst.mint, inst.input_amount, inst.authority, inst.signature,
            );
        }));

    tracing::info!(bind = %args.bind, "Streaming SPL token mints...");
    pipeline.run().await
}
