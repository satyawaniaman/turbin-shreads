//! Usage:
//!   cargo run --example jupiter_whale_swaps -- --bind 0.0.0.0:8001 --min-sol 50

use clap::Parser;
use shredstream_decoder_example::{Dex, InstructionKind, ShredPipeline};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "0.0.0.0:8001")]
    bind: String,

    #[arg(long, default_value = "10")]
    min_sol: f64,
}

const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let args = Args::parse();
    let min_lamports = (args.min_sol * LAMPORTS_PER_SOL) as u64;

    let mut pipeline = ShredPipeline::new(args.bind.clone());
    pipeline = pipeline
        .with_dex_filter(vec![Dex::JupiterV6])
        .on_instruction(Box::new(move |inst| {
            if inst.kind != InstructionKind::Buy {
                return;
            }
            let amount = inst.input_amount.unwrap_or(0);
            if amount < min_lamports {
                return;
            }
            let sol = amount as f64 / LAMPORTS_PER_SOL;
            println!(
                "WHALE BUY {:.2} SOL [slot {}] mint={} sig={}",
                sol, inst.slot, inst.mint, inst.signature,
            );
        }));

    tracing::info!(
        bind = %args.bind,
        min_sol = args.min_sol,
        "Watching for Jupiter whale swaps..."
    );
    pipeline.run().await
}
