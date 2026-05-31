import { NextResponse } from "next/server";
import type { DecodedEvent, FecStatus, InstructionKind, Protocol } from "@/lib/live-events";

export const dynamic = "force-dynamic";
export const revalidate = 0;

const RPC_URL = process.env.HELIUS_RPC_URL;
const SIGNATURE_LIMIT = Number(process.env.LIVE_SIGNATURE_LIMIT || 4);
const MAX_TRANSACTIONS = Number(process.env.LIVE_TRANSACTION_LIMIT || 16);

const PROGRAMS: Record<Protocol, string> = {
  PumpFun: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
  JupiterV6: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
  RaydiumAmm: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
  RaydiumCpmm: "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C",
  SplToken: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
};

type RpcResponse<T> = {
  result?: T;
  error?: { message?: string };
};

type SignatureInfo = {
  signature: string;
  slot: number;
  blockTime: number | null;
};

type ParsedInstruction = {
  programId?: string;
  parsed?: {
    type?: string;
    info?: Record<string, unknown>;
  };
};

type TokenBalance = {
  mint: string;
  owner?: string;
  uiTokenAmount?: {
    amount?: string;
  };
};

type ParsedTransaction = {
  slot: number;
  blockTime: number | null;
  transaction: {
    signatures: string[];
    message: {
      accountKeys: Array<{ pubkey: string; signer?: boolean } | string>;
      instructions: ParsedInstruction[];
    };
  };
  meta?: {
    preTokenBalances?: TokenBalance[];
    postTokenBalances?: TokenBalance[];
    logMessages?: string[];
    innerInstructions?: Array<{ instructions: ParsedInstruction[] }>;
  };
};

async function rpc<T>(method: string, params: unknown[]): Promise<T> {
  if (!RPC_URL) {
    throw new Error("Missing HELIUS_RPC_URL environment variable");
  }

  const response = await fetch(RPC_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: `${Date.now()}-${method}`,
      method,
      params,
    }),
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error(`Helius RPC ${response.status}`);
  }

  const json = (await response.json()) as RpcResponse<T>;
  if (json.error) {
    throw new Error(json.error.message || "Helius RPC error");
  }

  return json.result as T;
}

function getAccountKey(value: { pubkey: string; signer?: boolean } | string): string {
  return typeof value === "string" ? value : value.pubkey;
}

function getSigner(tx: ParsedTransaction): string {
  const signer = tx.transaction.message.accountKeys.find((key) => {
    return typeof key !== "string" && key.signer;
  });
  return signer ? getAccountKey(signer) : getAccountKey(tx.transaction.message.accountKeys[0] || "");
}

function flattenInstructions(tx: ParsedTransaction): ParsedInstruction[] {
  return [
    ...tx.transaction.message.instructions,
    ...(tx.meta?.innerInstructions || []).flatMap((entry) => entry.instructions),
  ];
}

function detectKind(protocol: Protocol, tx: ParsedTransaction): InstructionKind {
  const logs = (tx.meta?.logMessages || []).join(" ").toLowerCase();
  const instructions = flattenInstructions(tx);

  if (protocol === "SplToken") {
    const tokenType = instructions
      .map((ix) => ix.parsed?.type)
      .find(Boolean)
      ?.toLowerCase();
    if (tokenType === "transfer") return "Transfer";
    if (tokenType === "transferchecked") return "TransferChecked";
    if (tokenType === "mintto") return "MintTo";
    if (tokenType === "burn") return "Burn";
    return "Transfer";
  }

  if (protocol === "JupiterV6") return "Swap";
  if (logs.includes("create")) return "Create";
  if (logs.includes("buy")) return "Buy";
  if (logs.includes("sell")) return "Sell";
  if (logs.includes("initialize") || logs.includes("pool")) return "PoolInit";
  if (logs.includes("swap")) return "Swap";
  return protocol === "PumpFun" ? "Buy" : "Swap";
}

function tokenDelta(balance?: TokenBalance): number | null {
  const amount = balance?.uiTokenAmount?.amount;
  return amount ? Number(amount) : null;
}

function getMintAndAmounts(tx: ParsedTransaction) {
  const pre = tx.meta?.preTokenBalances || [];
  const post = tx.meta?.postTokenBalances || [];
  const primaryPost = post[0];
  const primaryPre = pre.find((balance) => balance.mint === primaryPost?.mint) || pre[0];
  const inputMint = primaryPre?.mint || null;
  const outputMint = primaryPost?.mint || null;
  const inputAmount = tokenDelta(primaryPre);
  const outputAmount = tokenDelta(primaryPost);

  return {
    mint: outputMint || inputMint || "unknown",
    inputMint,
    outputMint,
    inputAmount,
    outputAmount,
  };
}

function getFecStatus(tx: ParsedTransaction): FecStatus {
  return tx.meta ? "complete" : "partial";
}

function normalizeTransaction(
  protocol: Protocol,
  tx: ParsedTransaction,
  latencyMs: number
): DecodedEvent {
  const signature = tx.transaction.signatures[0];
  const tokenFields = getMintAndAmounts(tx);
  return {
    id: signature,
    slot: tx.slot,
    protocol,
    kind: detectKind(protocol, tx),
    signature,
    ...tokenFields,
    slippageBps: null,
    authority: getSigner(tx),
    fecStatus: getFecStatus(tx),
    decodeLatencyMs: latencyMs,
    timestamp: (tx.blockTime || Math.floor(Date.now() / 1000)) * 1000,
  };
}

async function fetchProtocolEvents(protocol: Protocol): Promise<DecodedEvent[]> {
  const programId = PROGRAMS[protocol];
  const signatures = await rpc<SignatureInfo[]>("getSignaturesForAddress", [
    programId,
    { limit: SIGNATURE_LIMIT, commitment: "confirmed" },
  ]);

  const events: DecodedEvent[] = [];
  for (const item of signatures) {
    if (events.length >= MAX_TRANSACTIONS) break;
    const startedAt = Date.now();
    const tx = await rpc<ParsedTransaction | null>("getTransaction", [
      item.signature,
      {
        encoding: "jsonParsed",
        commitment: "confirmed",
        maxSupportedTransactionVersion: 0,
      },
    ]);
    if (!tx) continue;
    events.push(normalizeTransaction(protocol, tx, Date.now() - startedAt));
  }

  return events;
}

export async function GET() {
  try {
    const decoderUrl = process.env.DECODER_API_URL;
    if (decoderUrl) {
      // Fetch directly from our custom Rust decoder backend
      const res = await fetch(decoderUrl, { cache: "no-store" });
      if (!res.ok) throw new Error(`Decoder API error: ${res.statusText}`);
      
      const data = await res.json();
      
      // Map the Rust ApiEvent struct to the Next.js DecodedEvent format
      const events: DecodedEvent[] = data.events.map((e: any) => ({
        id: e.signature,
        slot: e.slot,
        protocol: e.dex,
        kind: e.kind,
        signature: e.signature,
        mint: e.mint,
        inputMint: e.input_mint,
        outputMint: e.output_mint,
        inputAmount: e.input_amount,
        outputAmount: e.output_amount,
        slippageBps: e.slippage_bps,
        authority: e.authority,
        fecStatus: e.fec_status,
        decodeLatencyMs: e.decode_latency_ms,
        timestamp: e.timestamp,
      }));

      return NextResponse.json({
        events,
        fetchedAt: Date.now(),
        source: "rust-decoder",
      });
    }

    // Fallback to Helius RPC fetching if no decoder is configured
    const settled = await Promise.allSettled(
      (Object.keys(PROGRAMS) as Protocol[]).map(fetchProtocolEvents)
    );

    const events = settled
      .flatMap((result) => (result.status === "fulfilled" ? result.value : []))
      .filter((event, index, all) => all.findIndex((item) => item.id === event.id) === index)
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, MAX_TRANSACTIONS);

    return NextResponse.json({
      events,
      fetchedAt: Date.now(),
      source: "helius-rpc",
    });
  } catch (error) {
    return NextResponse.json(
      {
        error: error instanceof Error ? error.message : "Failed to fetch live events",
      },
      { status: 500 }
    );
  }
}
