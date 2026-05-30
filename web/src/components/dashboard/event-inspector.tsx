"use client";

import { X, ExternalLink } from "lucide-react";
import {
  type DecodedEvent,
  PROTOCOL_CONFIG,
  formatAmount,
  getExplorerUrl,
} from "@/lib/live-events";
import { CopyButton } from "./copy-button";

interface EventInspectorProps {
  event: DecodedEvent | null;
  onClose: () => void;
}

function DetailRow({
  label,
  value,
  mono = false,
  copyable = false,
  explorerType,
}: {
  label: string;
  value: string | null;
  mono?: boolean;
  copyable?: boolean;
  explorerType?: "tx" | "address";
}) {
  if (value === null || value === undefined) return null;

  return (
    <div className="flex flex-col gap-1 border-b border-hairline py-2.5 last:border-b-0">
      <span className="text-[10px] font-medium uppercase tracking-wider text-stone">
        {label}
      </span>
      <div className="flex items-center gap-1.5">
        <span
          className={`break-all text-xs leading-relaxed ${
            mono ? "font-mono text-charcoal" : "text-ink"
          }`}
        >
          {value}
        </span>
        {copyable && <CopyButton value={value} />}
        {explorerType && (
          <a
            href={getExplorerUrl(explorerType, value)}
            target="_blank"
            rel="noopener noreferrer"
            className="shrink-0 text-stone transition-colors hover:text-accent-blue"
            title="View on Solana Explorer"
          >
            <ExternalLink className="size-3" />
          </a>
        )}
      </div>
    </div>
  );
}

export function EventInspector({ event, onClose }: EventInspectorProps) {
  if (!event) return null;

  const config = PROTOCOL_CONFIG[event.protocol];

  return (
    <div className="flex w-80 shrink-0 animate-slide-right flex-col border-l border-hairline bg-surface-card">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-hairline p-4">
        <div className="flex items-center gap-2">
          <span
            className="inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium"
            style={{
              backgroundColor: `${config.color}15`,
              color: config.color,
            }}
          >
            <span
              className="inline-block size-1.5 rounded-full"
              style={{ backgroundColor: config.color }}
            />
            {config.label}
          </span>
          <span className="rounded-md bg-surface-elevated px-2 py-0.5 text-[10px] font-medium text-charcoal">
            {event.kind}
          </span>
        </div>
        <button
          onClick={onClose}
          className="rounded-lg p-1 text-mute-text transition-colors hover:bg-surface-elevated hover:text-ink"
          aria-label="Close inspector"
        >
          <X className="size-4" />
        </button>
      </div>

      {/* Details */}
      <div className="flex-1 overflow-y-auto p-4">
        <DetailRow
          label="Slot"
          value={event.slot.toLocaleString()}
          mono
        />
        <DetailRow
          label="Signature"
          value={event.signature}
          mono
          copyable
          explorerType="tx"
        />
        <DetailRow
          label="Mint"
          value={event.mint}
          mono
          copyable
          explorerType="address"
        />
        <DetailRow
          label="Input Mint"
          value={event.inputMint}
          mono
          copyable
          explorerType="address"
        />
        <DetailRow
          label="Output Mint"
          value={event.outputMint}
          mono
          copyable
          explorerType="address"
        />
        <DetailRow
          label="Input Amount"
          value={
            event.inputAmount !== null
              ? `${formatAmount(event.inputAmount)} (raw: ${event.inputAmount.toLocaleString()})`
              : null
          }
        />
        <DetailRow
          label="Output Amount"
          value={
            event.outputAmount !== null
              ? `${formatAmount(event.outputAmount)} (raw: ${event.outputAmount.toLocaleString()})`
              : null
          }
        />
        <DetailRow
          label="Authority"
          value={event.authority}
          mono
          copyable
          explorerType="address"
        />
        <DetailRow
          label="Slippage"
          value={
            event.slippageBps !== null
              ? `${event.slippageBps} bps (${(event.slippageBps / 100).toFixed(2)}%)`
              : null
          }
        />
        <DetailRow
          label="FEC Status"
          value={event.fecStatus}
        />
        <DetailRow
          label="Decode Latency"
          value={`${event.decodeLatencyMs}ms`}
        />
        <DetailRow
          label="Timestamp"
          value={new Date(event.timestamp).toISOString()}
          mono
        />
      </div>

      {/* Footer link */}
      <div className="border-t border-hairline p-3">
        <a
          href={getExplorerUrl("tx", event.signature)}
          target="_blank"
          rel="noopener noreferrer"
          className="flex h-8 w-full items-center justify-center gap-1.5 rounded-lg bg-ink text-xs font-medium text-canvas transition-opacity hover:opacity-90"
        >
          View on Solana Explorer
          <ExternalLink className="size-3" />
        </a>
      </div>
    </div>
  );
}
