//! Usage:
//!   cargo run --example listen_and_decode -- --bind 0.0.0.0:8001

use clap::Parser;
use shredstream_decoder_example::ShredPipeline;

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
    pipeline = pipeline.on_instruction(Box::new(|inst| {
        println!(
            "[slot {}] {} {:?} | mint={} | in={:?} out={:?} | sig={}",
            inst.slot,
            inst.dex,
            inst.kind,
            inst.mint,
            inst.input_amount,
            inst.output_amount,
            inst.signature,
        );
    }));

    tracing::info!(bind = %args.bind, "Listening for shreds...");
    pipeline.run().await
}
