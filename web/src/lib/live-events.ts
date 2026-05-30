// Shared live event types and display helpers.

export type Protocol =
  | "PumpFun"
  | "JupiterV6"
  | "RaydiumAmm"
  | "RaydiumCpmm"
  | "SplToken";

export type InstructionKind =
  | "Buy"
  | "Sell"
  | "Create"
  | "Swap"
  | "Transfer"
  | "TransferChecked"
  | "MintTo"
  | "Burn"
  | "PoolInit"
  | "Unknown";

export type FecStatus = "complete" | "recovered" | "partial";

export interface DecodedEvent {
  id: string;
  slot: number;
  protocol: Protocol;
  kind: InstructionKind;
  signature: string;
  mint: string;
  inputMint: string | null;
  outputMint: string | null;
  inputAmount: number | null;
  outputAmount: number | null;
  slippageBps: number | null;
  authority: string;
  fecStatus: FecStatus;
  decodeLatencyMs: number;
  timestamp: number;
}

export interface LiveEventsResponse {
  events: DecodedEvent[];
  fetchedAt: number;
  source: "helius-rpc";
}

export const PROTOCOL_CONFIG: Record<
  Protocol,
  { label: string; color: string; glow: string; shortLabel: string }
> = {
  PumpFun: {
    label: "Pump.fun",
    shortLabel: "PUMP",
    color: "#ff801f",
    glow: "rgba(255, 89, 0, 0.22)",
  },
  JupiterV6: {
    label: "Jupiter v6",
    shortLabel: "JUP",
    color: "#3b9eff",
    glow: "rgba(0, 117, 255, 0.34)",
  },
  RaydiumAmm: {
    label: "Raydium AMM",
    shortLabel: "RAY",
    color: "#11ff99",
    glow: "rgba(34, 255, 153, 0.18)",
  },
  RaydiumCpmm: {
    label: "Raydium CPMM",
    shortLabel: "CPMM",
    color: "#ff2047",
    glow: "rgba(255, 32, 71, 0.34)",
  },
  SplToken: {
    label: "SPL Token",
    shortLabel: "SPL",
    color: "#ffc53d",
    glow: "rgba(255, 197, 61, 0.22)",
  },
};

export const PROTOCOL_LIST: Protocol[] = [
  "PumpFun",
  "JupiterV6",
  "RaydiumAmm",
  "RaydiumCpmm",
  "SplToken",
];

export const KIND_LIST: InstructionKind[] = [
  "Buy",
  "Sell",
  "Create",
  "Swap",
  "Transfer",
  "TransferChecked",
  "MintTo",
  "Burn",
  "PoolInit",
  "Unknown",
];

export function shortenAddress(address: string, chars = 4): string {
  if (address.length <= chars * 2 + 3) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

export function formatAmount(amount: number | null, decimals = 9): string {
  if (amount === null) return "-";
  const value = amount / Math.pow(10, decimals);
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(2)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(2)}K`;
  if (value >= 1) return value.toFixed(2);
  return value.toFixed(4);
}

export function formatNumber(n: number): string {
  return n.toLocaleString("en-US");
}

export function getExplorerUrl(
  type: "tx" | "address",
  value: string,
  cluster: "mainnet-beta" | "devnet" = "mainnet-beta"
): string {
  return `https://explorer.solana.com/${type}/${value}?cluster=${cluster}`;
}
