# Turbine Real-Time Solana Decoder

A high-performance Rust-powered real-time Solana analytics platform that streams, decodes, and visualizes DeFi activity from Pump.fun, Jupiter, Raydium, and SPL Token programs.

## Architecture Overview

```text
┌──────────────────────────────────────────────┐
│              Solana Mainnet                  │
└───────────────────┬──────────────────────────┘
                    │
                    │ logsSubscribe (WebSocket)
                    ▼
┌──────────────────────────────────────────────┐
│         Rust Decoder Engine (DigitalOcean)   │
│                                              │
│  • WebSocket Client                          │
│  • Transaction Fetcher                       │
│  • Protocol Decoders                         │
│  • Event Processing Pipeline                 │
│  • In-Memory Event Queue                     │
└───────────────────┬──────────────────────────┘
                    │
                    │ GET /api/events
                    ▼
┌──────────────────────────────────────────────┐
│             Axum HTTP API (:8002)            │
└───────────────────┬──────────────────────────┘
                    │
                    │ Polling
                    ▼
┌──────────────────────────────────────────────┐
│         Next.js Dashboard (Vercel)           │
│                                              │
│  • Live Event Feed                           │
│  • Protocol Analytics                        │
│  • Event Inspector                           │
│  • Search & Filters                          │
│  • CSV Export                                │
└──────────────────────────────────────────────┘
```

## Data Flow

```text
Solana Transaction
        ↓
WebSocket Notification
        ↓
Fetch Full Transaction
        ↓
Custom Rust Decoder
        ↓
Structured Event
        ↓
Axum API
        ↓
Next.js Dashboard
```

## What This Project Does

This project monitors live Solana activity and converts raw blockchain transactions into human-readable events.

Supported Protocols:

- Pump.fun
- Jupiter v6
- Raydium AMM
- Raydium CPMM
- SPL Token Program

The Rust backend continuously listens to Solana transaction logs, fetches matching transactions, decodes protocol-specific instructions, and exposes them through a low-latency HTTP API consumed by the dashboard.

## Key Engineering Components

### Rust Decoder Engine

The backend is responsible for:

- Maintaining a persistent WebSocket connection to Solana
- Monitoring selected protocol program IDs
- Fetching full transaction data
- Decoding protocol-specific instructions
- Calculating metadata such as latency and transaction details
- Storing recent events in an in-memory queue
- Serving decoded events through an Axum API

### Axum API Layer

The backend exposes:

```http
GET /api/events
```

which returns a stream of recently decoded blockchain activity.

```json
{
  "events": [
    {
      "instruction": {
        "dex": "PumpFun",
        "kind": "Buy",
        "signature": "...",
        "slot": 423403682,
        "mint": "...",
        "input_amount": 510050001,
        "output_amount": 4036362905078,
        "authority": "..."
      },
      "timestamp": 1716300000000,
      "fec_status": "complete",
      "decode_latency_ms": 45
    }
  ]
}
```

### Next.js Dashboard

The frontend visualizes live protocol activity through:

- Real-time transaction feed
- Protocol breakdown charts
- Event inspector panel
- Search and filtering
- Solana Explorer integration
- CSV export functionality

## Fault-Tolerant Design

The system supports automatic fallback mode.

Primary Flow:

```text
Rust Decoder Engine
        ↓
Axum API
        ↓
Dashboard
```

Fallback Flow:

```text
Helius REST API
        ↓
Dashboard
```

If the Rust backend becomes unavailable, the dashboard automatically switches to Helius and continues functioning without interruption.

## Why This Project Matters

Most blockchain dashboards rely entirely on periodic RPC polling.

This project introduces a dedicated Rust event-processing layer that:

- Streams blockchain activity in near real time
- Performs protocol-aware transaction decoding
- Separates ingestion, processing, and visualization
- Provides a scalable architecture for future protocol integrations

The result is a production-style pipeline that demonstrates backend systems engineering, blockchain data processing, and modern frontend visualization in a single project.

## Tech Stack

### Backend

- Rust
- Tokio
- Axum
- Solana SDK
- WebSocket Streaming
- Systemd Deployment

### Frontend

- Next.js
- React
- TypeScript
- Tailwind CSS

### Infrastructure

- DigitalOcean
- Vercel
- Helius RPC & WebSocket APIs

## Running Locally

Backend:

```bash
cargo run --release --example serve_api -- --ws --bind-http 0.0.0.0:8002
```

Frontend:

```bash
cd web
bun install
bun run dev
```

Environment Variables:

```env
HELIUS_RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY

DECODER_API_URL=http://localhost:8002/api/events
```
