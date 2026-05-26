# ShredStream Decoder Capstone

Decode raw Solana turbine shreds into readable transaction-level events for Pump.fun, Jupiter, Raydium, and SPL Token activity.

This project is a Rust decoder engine plus a Vercel-ready Next.js dashboard. The dashboard is included to make the capstone easy to review even when the evaluator does not have a live ShredStream feed.

## What This Project Does

Solana validators split block data into small network packets called shreds. This project listens for those shred packets over UDP, reconstructs transaction entries, and decodes selected program instructions into structured events.

Supported decoders:

- Pump.fun token creation and trades
- Jupiter v6 swaps
- Raydium AMM and CPMM swaps/pool initialization
- SPL Token transfers, checked transfers, mints, and burns

Core pipeline:

1. Receive raw UDP shred packets.
2. Parse Solana shred headers and payloads.
3. Rebuild missing/partial data with Reed-Solomon FEC.
4. Deserialize Solana entries and transactions.
5. Match known program IDs and decode instruction data.
6. Print JSON/text output that can be sent to a dashboard API or database.

## Why It Matters

Most Solana applications consume finalized RPC data. This project works closer to the network layer by reading shreds directly, which can surface activity earlier and gives students a practical view of Solana's data propagation, transaction encoding, and high-throughput design.

## Dashboard

The frontend dashboard lives in `web/`. It shows decoded shred events in a polished, dark-themed interface inspired by developer-tool aesthetics.

### Dashboard Features

- **Metric Cards** — Total events, events/sec, latest slot, FEC recovered sets, average decode latency
- **Protocol Breakdown** — Visual breakdown of events by protocol (Pump.fun, Jupiter, Raydium AMM/CPMM, SPL Token) with clickable filters
- **Events Table** — Sortable table showing decoded events with protocol badges, monospace addresses, copy buttons, and Solana Explorer links
- **Event Inspector** — Click any row to open a detailed side panel showing all decoded fields
- **Search & Filters** — Search by signature, mint, or authority; filter by protocol and instruction kind
- **Live Helius RPC Feed** — The dashboard polls a server-side Next.js API route backed by Helius mainnet RPC; pause/resume and adjust refresh speed
- **CSV Export** — Export filtered events to CSV for analysis
- **Copy Buttons** — One-click copy for signatures, mints, and authority addresses
- **Solana Explorer Links** — Direct links to view transactions and addresses on Solana Explorer

### Live Data Source

The dashboard does not use mock data. It calls `web/src/app/api/events/route.ts`, which fetches recent mainnet transactions for the supported protocol program IDs through Helius RPC.

For local development and Vercel, configure:

```bash
HELIUS_RPC_URL=https://mainnet.helius-rpc.com/?api-key=your-api-key
```

The current dashboard live feed is RPC-based. The Rust ShredStream decoder is still the lower-level raw-shred engine and should run separately when using a true UDP ShredStream source.

### Run the Dashboard

```bash
cd web
bun install
bun run dev
```

Open:

```text
http://127.0.0.1:3000
```

## Deploy On Vercel

Use these Vercel settings:

- Root Directory: `web`
- Framework Preset: Next.js
- Install Command: `bun install`
- Build Command: `bun run build`
- Output Directory: leave default

Vercel should host the dashboard. The live UDP decoder should run separately as a worker or server because the ShredStream listener is a long-running UDP process, not a normal HTTP request handler.

## Live ShredStream Demo

Run the Rust decoder locally:

```bash
cargo run -- --bind 0.0.0.0:8001 --format json
```

Then point your ShredStream UDP source at:

```text
<your-machine-ip>:8001
```

Optional filters:

```bash
cargo run -- --bind 0.0.0.0:8001 --format json --dex pumpfun --kind buy,sell
cargo run -- --bind 0.0.0.0:8001 --format json --dex jupiter_v6
```

For production, pipe this JSON stream into a small HTTP ingest service or database, then have the Vercel dashboard read from that API.

## CLI Usage

The command-line app is useful for terminal demos, logging, and piping decoded events into another tool.

Text output:

```bash
cargo run -- --bind 0.0.0.0:8001
```

JSON output:

```bash
cargo run -- --bind 0.0.0.0:8001 --format json
```

Filter examples:

```bash
cargo run -- --dex pumpfun --kind buy,sell
cargo run -- --dex jupiter_v6 --format json
```

## Example Programs

Run focused examples with:

```bash
cargo run --example listen_and_decode
cargo run --example shred_to_json
cargo run --example pumpfun_trades
cargo run --example pumpfun_new_tokens
cargo run --example jupiter_swaps
cargo run --example jupiter_whale_swaps
cargo run --example raydium_swaps
cargo run --example raydium_new_pools
cargo run --example token_transfers
cargo run --example token_mints
```

## Installation

Install Rust from [rustup.rs](https://rustup.rs), then clone the repository and build:

```bash
cargo build
```

Run tests/checks:

```bash
cargo test
```

## Project Structure

```text
src/main.rs                          CLI entry point
src/pipeline/                        UDP listener and decode pipeline
src/shred/                           Shred header and payload parsing
src/fec/                             Reed-Solomon recovery and slot accumulation
src/entry/                           Solana entry deserialization
src/decoder/                         Program-specific instruction decoders
src/types.rs                         Shared decoded event types
examples/                            Focused examples for each supported strategy
web/src/app/page.tsx                 Dashboard page (main UI)
web/src/app/api/events/route.ts      Live Helius RPC data route
web/src/app/layout.tsx               Root layout with fonts and metadata
web/src/app/globals.css              Dark theme design tokens
web/src/components/dashboard/        Dashboard UI components
web/src/hooks/use-dashboard.ts       Dashboard state management hook
web/src/lib/live-events.ts           Shared live event types and helper utilities
web/package.json                     Next.js build and dev scripts
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

## Architecture

```text
ShredStream UDP feed
        |
        v
Rust decoder worker (cargo run)
        |
        v
JSON event stream (stdout / HTTP API / database)
        |
        v
Next.js dashboard on Vercel (web/)
```

## Demo Script

1. Open the dashboard at the Vercel URL or `http://127.0.0.1:3000`.
2. Observe metric cards updating as live Helius RPC events are fetched.
3. View the protocol breakdown to see event distribution across Pump.fun, Jupiter, Raydium, and SPL Token.
4. Filter to a specific protocol (e.g., Pump.fun) and observe the table update.
5. Search for a specific signature or mint address.
6. Click an event row to open the inspector and examine decoded fields.
7. Export the filtered events to CSV.
8. Show the Rust CLI command that would stream live decoded JSON:
   ```bash
   cargo run -- --bind 0.0.0.0:8001 --format json --dex pumpfun --kind buy,sell
   ```
9. Explain that production would connect the Rust worker to an API/database that the dashboard reads from.

## Capstone Testing Checklist

For a reviewer without ShredStream access:

1. Run `cargo test` — all tests should pass.
2. Run `cd web && bun install && bun run dev`.
3. Open `http://127.0.0.1:3000`.
4. Confirm the dashboard loads with metric cards, protocol breakdown, and events table.
5. Verify events are fetched automatically from Helius RPC.
6. Click a table row to open the event inspector.
7. Test search, protocol filter, and kind filter.
8. Click a signature link to verify it opens Solana Explorer.
9. Click a copy button to verify clipboard copy works.
10. Export events to CSV and verify the file downloads.

For a reviewer with live ShredStream access:

1. Run `cargo run -- --bind 0.0.0.0:8001 --format json`.
2. Configure the shred sender to send UDP packets to port `8001`.
3. Confirm decoded JSON lines appear in the terminal.
4. Optional: connect the JSON output to an ingest API used by the Vercel dashboard.

## Notes And Limitations

- This is a decoder/demo project, not a trading bot and not a wallet app.
- It does not sign transactions or require private keys.
- Live mode needs a valid Solana shred UDP source.
- The Next.js dashboard uses live Helius RPC data, not mock data.
- The UDP listener should be deployed outside Vercel as a long-running worker.
- Program decoders are examples and can be extended for more protocols or instruction variants.

## Future Improvements

- Add an HTTP ingest service for decoded JSON events.
- Export dashboard rows to CSV.
- Add persistent storage with SQLite or Supabase.
- Add more decoders for additional Solana programs.
- Add historical charts, alerts, and saved protocol filters.
- Add whale detection for large swaps and transfers.
- Add token detail view for per-mint event history.
