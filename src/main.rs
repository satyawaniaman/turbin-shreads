use clap::Parser;
use shredstream_decoder_example::{Dex, InstructionKind, ShredPipeline};
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "shredstream-decoder-example")]
#[command(about = "Decode Solana turbine shreds into DEX transactions")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:8001")]
    bind: String,

    #[arg(long, default_value = "text")]
    format: String,

    #[arg(long)]
    dex: Option<String>,

    #[arg(long)]
    kind: Option<String>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up tracing based on verbosity
    let log_level = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| log_level.to_string());

    if cli.format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

    // Build pipeline
    let mut pipeline = ShredPipeline::new(cli.bind.clone());

    // Apply DEX filter
    if let Some(dex_str) = &cli.dex {
        let dexes: Vec<Dex> = dex_str
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                // Handle both snake_case and PascalCase
                Dex::from_str(s)
                    .or_else(|_| match s.to_lowercase().as_str() {
                        "pumpfun" | "pump_fun" => Ok(Dex::PumpFun),
                        "jupiter" | "jupiter_v6" | "jupiterv6" => Ok(Dex::JupiterV6),
                        "raydium_amm" | "raydiumamm" => Ok(Dex::RaydiumAmm),
                        "raydium_cpmm" | "raydiumcpmm" => Ok(Dex::RaydiumCpmm),
                        "spl_token" | "spltoken" | "spl" => Ok(Dex::SplToken),
                        _ => Err(strum::ParseError::VariantNotFound),
                    })
                    .ok()
            })
            .collect();

        if !dexes.is_empty() {
            tracing::info!(?dexes, "DEX filter applied");
            pipeline = pipeline.with_dex_filter(dexes);
        }
    }

    // Apply kind filter
    if let Some(kind_str) = &cli.kind {
        let kinds: Vec<InstructionKind> = kind_str
            .split(',')
            .filter_map(|s| InstructionKind::from_str(s.trim()).ok())
            .collect();

        if !kinds.is_empty() {
            tracing::info!(?kinds, "Instruction kind filter applied");
            pipeline = pipeline.with_kind_filter(kinds);
        }
    }

    // Set up output callback
    let json_output = cli.format == "json";
    pipeline = pipeline.on_instruction(Box::new(move |inst| {
        if json_output {
            if let Ok(json) = serde_json::to_string(inst) {
                println!("{json}");
            }
        } else {
            println!(
                "[slot {}] {} {:?} mint={} sig={}",
                inst.slot, inst.dex, inst.kind, inst.mint, inst.signature,
            );
        }
    }));

    tracing::info!(
        bind = %cli.bind,
        format = %cli.format,
        "Starting shred decoder - connect OrbitFlare ShredStream to this address"
    );

    pipeline.run().await
}
