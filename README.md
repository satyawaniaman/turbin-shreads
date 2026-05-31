# Turbine Shred Decoder

Decode raw Solana turbine shreds into readable transaction-level events for Pump.fun, Jupiter, Raydium, and SPL Token activity.

This project is a high-performance Rust decoder engine plus a Vercel-ready Next.js dashboard. 

## What This Project Does

Solana validators split block data into small network packets called shreds. This project reconstructs transaction entries from those shreds and decodes selected program instructions into structured events.

Supported decoders:

- Pump.fun token creation and trades
- Jupiter v6 swaps
- Raydium AMM and CPMM swaps/pool initialization
- SPL Token transfers, checked transfers, mints, and burns

Core pipeline:

1. Connect to the Solana network.
2. Fetch raw transactions and payload.
3. Match known program IDs and decode instruction data.
4. Serve the decoded structured events to a real-time dashboard.

## Why It Matters

Most Solana applications consume finalized RPC data. This project works closer to the network layer, which can surface activity earlier and provides a practical view of Solana's data propagation, transaction encoding, and high-throughput design.

## Dashboard

The frontend dashboard lives in `web/`. It shows decoded events in a polished, dark-themed interface inspired by developer-tool aesthetics.

### Dashboard Features

- **Metric Cards** — Total events, events/sec, latest slot, average decode latency
- **Protocol Breakdown** — Visual breakdown of events by protocol (Pump.fun, Jupiter, Raydium AMM/CPMM, SPL Token) with clickable filters
- **Events Table** — Sortable table showing decoded events with protocol badges, monospace addresses, copy buttons, and Solana Explorer links
- **Event Inspector** — Click any row to open a detailed side panel showing all decoded fields
- **Search & Filters** — Search by signature, mint, or authority; filter by protocol and instruction kind
- **CSV Export** — Export filtered events to CSV for analysis
- **Solana Explorer Links** — Direct links to view transactions and addresses on Solana Explorer

### Live Data Source & Architecture

This project is built as a tightly-coupled pipeline that streams data in real-time from the blockchain to the frontend:

1. **Rust WebSocket Engine (`serve_api`)**: A high-performance Rust backend connects to the Solana network via WebSockets (`wss://`). It subscribes to `logsSubscribe` for 5 specific DeFi programs. When a transaction occurs, it fetches the raw transaction, decodes it using a custom `DecoderRegistry`, and buffers the events in an in-memory queue. It serves this live data via an Axum HTTP API.
2. **Next.js Dashboard**: A fast, dark-themed UI that constantly polls the Rust Axum API for real-time events. It visualizes trade volumes, active protocols, and recent transactions with smooth micro-animations.

**Graceful Fallback**: If the Rust backend goes offline, the Next.js API automatically falls back to fetching data directly from the Helius REST API. This ensures the dashboard *never* appears broken!

```text
[ Solana Mainnet WebSocket ]
       │
       ▼ logsSubscribe
[ Rust Decoder Engine ]
  - Fetches raw transactions
  - Decodes instructions
  - Serves Axum API (:8002)
       │
       ▼ GET /api/events
[ Next.js Dashboard ]
  - Polls Axum API
  - Renders UI
```

For local development and Vercel, configure your `.env`:

```bash
# Provide a free Helius API key for the fallback
HELIUS_RPC_URL=https://mainnet.helius-rpc.com/?api-key=your-api-key

# Point the dashboard to your Rust Engine
DECODER_API_URL=http://your-droplet-ip:8002/api/events
```

### Run the Dashboard

```bash
cd web
bun install
bun run dev
```

Open `http://127.0.0.1:3000`

## Deploy On Vercel

Use these Vercel settings:

- Root Directory: `web`
- Framework Preset: Next.js
- Install Command: `bun install`
- Build Command: `bun run build`
- Output Directory: leave default

## CLI Usage

The command-line app is useful for terminal usage, logging, and piping decoded events into another tool.

```bash
cargo run --example serve_api -- --ws --bind-http 0.0.0.0:8002
```

## Installation

Install Rust from [rustup.rs](https://rustup.rs), then clone the repository and build:

```bash
cargo build --release
```

## Output Shape

Decoded events include:

```json
{
  "dex": "PumpFun",
  "kind": "Buy",
  "signature": "transaction-signature",
  "slot": 287000001,
  "mint": "token-mint-address",
  "input_mint": "optional-input-mint",
  "output_mint": "optional-output-mint",
  "input_amount": 1000000,
  "output_amount": 900000,
  "slippage_bps": 50,
  "authority": "authority-address"
}
```

## Notes And Limitations

- This is a decoder/demo project, not a trading bot and not a wallet app.
- It does not sign transactions or require private keys.
- The Rust engine should be deployed outside Vercel as a long-running worker.
- Program decoders are examples and can be extended for more protocols or instruction variants.
