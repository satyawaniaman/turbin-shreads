//! Usage:
//!   cargo run --example jupiter_swaps -- --bind 0.0.0.0:8001

use clap::Parser;
use shredstream_decoder_example::{Dex, ShredPipeline};

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
        .with_dex_filter(vec![Dex::JupiterV6])
        .on_instruction(Box::new(|inst| {
            println!(
                "JUP {:?} [slot {}] {} -> {} | in={:?} out={:?} slippage={:?}bps | sig={}",
                inst.kind,
                inst.slot,
                inst.input_mint.map(|m| m.to_string()).unwrap_or_default(),
                inst.output_mint.map(|m| m.to_string()).unwrap_or_default(),
                inst.input_amount,
                inst.output_amount,
                inst.slippage_bps,
                inst.signature,
            );
        }));

    tracing::info!(bind = %args.bind, "Streaming Jupiter swaps...");
    pipeline.run().await
}
