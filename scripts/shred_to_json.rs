//! Usage:
//!   cargo run --example shred_to_json -- --bind 0.0.0.0:8001

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
        .with_writer(std::io::stderr)
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".to_string()))
        .init();

    let args = Args::parse();

    let mut pipeline = ShredPipeline::new(args.bind.clone());
    pipeline = pipeline.on_instruction(Box::new(|inst| {
        if let Ok(json) = serde_json::to_string(inst) {
            println!("{json}");
        }
    }));

    eprintln!("Listening on {} - outputting JSON to stdout", args.bind);
    pipeline.run().await
}
