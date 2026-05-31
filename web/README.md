# ShredScope Web

Next.js frontend for the ShredStream decoder capstone.

This app was created with:

```bash
bunx --bun create-next-app@latest web --typescript --tailwind --eslint --app --src-dir --import-alias '@/*' --use-bun --yes
bunx --bun shadcn@latest init --preset b37aFvHii --template next
```

## Development

Create `.env.local`:

```bash
HELIUS_RPC_URL=https://mainnet.helius-rpc.com/?api-key=your-api-key
```

Then run:

```bash
bun install
bun dev
```

Open `http://localhost:3000`.

The live data route is `src/app/api/events/route.ts`. Shared event types live in `src/lib/live-events.ts`.

## Vercel

Use:

- Root Directory: `web`
- Framework Preset: Next.js
- Install Command: `bun install`
- Build Command: `bun run build`

Add `HELIUS_RPC_URL` in Vercel project environment variables.

## Architecture Note

Vercel hosts this frontend and polls Helius RPC through a server-side route. The Rust UDP ShredStream decoder should run separately if you want true raw-shred ingestion.
