"use client";

import { ExternalLink, Inbox, ChevronLeft, ChevronRight } from "lucide-react";
import {
  type DecodedEvent,
  PROTOCOL_CONFIG,
  shortenAddress,
  formatAmount,
  getExplorerUrl,
} from "@/lib/live-events";
import { CopyButton } from "./copy-button";

interface EventsTableProps {
  events: DecodedEvent[];
  selectedEvent: DecodedEvent | null;
  onSelectEvent: (e: DecodedEvent) => void;
  // Pagination
  currentPage: number;
  totalPages: number;
  perPage: number;
  totalFiltered: number;
  onPageChange: (p: number) => void;
  onPerPageChange: (n: number) => void;
}

function FecBadge({ status }: { status: DecodedEvent["fecStatus"] }) {
  const styles = {
    complete: "bg-accent-green/10 text-accent-green",
    recovered: "bg-accent-yellow/10 text-accent-yellow",
    partial: "bg-accent-red/10 text-accent-red",
  };

  return (
    <span
      className={`inline-flex rounded-full px-1.5 py-0.5 text-[10px] font-medium ${styles[status]}`}
    >
      {status}
    </span>
  );
}

function ProtocolBadge({ protocol }: { protocol: DecodedEvent["protocol"] }) {
  const config = PROTOCOL_CONFIG[protocol];
  return (
    <span
      className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[10px] font-medium"
      style={{
        backgroundColor: `${config.color}15`,
        color: config.color,
      }}
    >
      <span
        className="inline-block size-1.5 rounded-full"
        style={{ backgroundColor: config.color }}
      />
      {config.shortLabel}
    </span>
  );
}

const PER_PAGE_OPTIONS = [10, 25, 50, 100];

export function EventsTable({
  events,
  selectedEvent,
  onSelectEvent,
  currentPage,
  totalPages,
  perPage,
  totalFiltered,
  onPageChange,
  onPerPageChange,
}: EventsTableProps) {

  if (events.length === 0 && totalFiltered === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 rounded-xl border border-hairline-strong bg-surface-card py-16">
        <Inbox className="size-10 text-stone" />
        <div className="text-center">
          <p className="text-sm font-medium text-charcoal">No events found</p>
          <p className="mt-1 text-xs text-mute-text">
            Adjust your filters or wait for new events to arrive.
          </p>
        </div>
      </div>
    );
  }

  const startIdx = (currentPage - 1) * perPage + 1;
  const endIdx = Math.min(currentPage * perPage, totalFiltered);

  return (
    <div className="flex flex-1 flex-col overflow-hidden rounded-xl border border-hairline-strong bg-surface-card">
      {/* Table header */}
      <div className="grid grid-cols-[80px_90px_72px_1fr_1fr_80px_56px_64px] gap-2 border-b border-hairline px-3 py-2 text-[10px] font-medium uppercase tracking-wider text-stone">
        <span>Slot</span>
        <span>Protocol</span>
        <span>Kind</span>
        <span>Signature</span>
        <span>Mint</span>
        <span className="text-right">Amount</span>
        <span className="text-right">Latency</span>
        <span className="text-center">FEC</span>
      </div>

      {/* Table body */}
      <div className="flex-1 overflow-y-auto" id="events-table-body">
        {events.map((event) => {
          const isSelected = selectedEvent?.id === event.id;
          return (
            <div
              key={event.id}
              role="button"
              tabIndex={0}
              onClick={() => onSelectEvent(event)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  onSelectEvent(event);
                }
              }}
              className={`grid w-full cursor-pointer grid-cols-[80px_90px_72px_1fr_1fr_80px_56px_64px] items-center gap-2 border-b border-hairline px-3 py-2 text-left text-xs transition-colors ${
                isSelected
                  ? "bg-surface-elevated"
                  : "hover:bg-surface-elevated/50"
              }`}
            >
              {/* Slot */}
              <span className="font-mono text-charcoal">
                {event.slot.toLocaleString()}
              </span>

              {/* Protocol */}
              <span>
                <ProtocolBadge protocol={event.protocol} />
              </span>

              {/* Kind */}
              <span className="text-mute-text">{event.kind}</span>

              {/* Signature */}
              <span className="flex items-center gap-1 overflow-hidden">
                <a
                  href={getExplorerUrl("tx", event.signature)}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={(e) => e.stopPropagation()}
                  className="truncate font-mono text-accent-blue transition-colors hover:text-ink"
                  title={event.signature}
                >
                  {shortenAddress(event.signature, 6)}
                </a>
                <CopyButton value={event.signature} />
                <a
                  href={getExplorerUrl("tx", event.signature)}
                  target="_blank"
                  rel="noopener noreferrer"
                  onClick={(e) => e.stopPropagation()}
                  className="text-stone transition-colors hover:text-accent-blue"
                  title="View on Solana Explorer"
                >
                  <ExternalLink className="size-2.5" />
                </a>
              </span>

              {/* Mint */}
              <span className="flex items-center gap-1 overflow-hidden">
                <span
                  className="truncate font-mono text-charcoal"
                  title={event.mint}
                >
                  {shortenAddress(event.mint, 4)}
                </span>
                <CopyButton value={event.mint} />
              </span>

              {/* Amount */}
              <span className="text-right font-mono text-charcoal">
                {formatAmount(event.inputAmount)}
              </span>

              {/* Latency */}
              <span className="text-right font-mono text-mute-text">
                {event.decodeLatencyMs}ms
              </span>

              {/* FEC */}
              <span className="flex justify-center">
                <FecBadge status={event.fecStatus} />
              </span>
            </div>
          );
        })}
      </div>

      {/* ── Pagination footer ── */}
      <div className="flex items-center justify-between border-t border-hairline px-3 py-2">
        {/* Left: per-page selector + count info */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <span className="text-[10px] text-mute-text">Rows</span>
            <select
              value={perPage}
              onChange={(e) => onPerPageChange(Number(e.target.value))}
              className="h-6 rounded border border-hairline-strong bg-surface-deep px-1.5 text-[10px] text-charcoal outline-none transition-colors focus:border-ink"
              aria-label="Rows per page"
              id="per-page-select"
            >
              {PER_PAGE_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n}
                </option>
              ))}
            </select>
          </div>
          <span className="text-[10px] text-stone">
            {startIdx}–{endIdx} of {totalFiltered}
          </span>
        </div>

        {/* Right: page navigation */}
        <div className="flex items-center gap-1">
          <button
            onClick={() => onPageChange(currentPage - 1)}
            disabled={currentPage <= 1}
            className="inline-flex h-6 w-6 items-center justify-center rounded border border-hairline-strong text-charcoal transition-colors hover:bg-surface-elevated disabled:pointer-events-none disabled:opacity-30"
            aria-label="Previous page"
          >
            <ChevronLeft className="size-3" />
          </button>
          <span className="min-w-[3rem] text-center text-[10px] text-mute-text">
            {currentPage} / {totalPages}
          </span>
          <button
            onClick={() => onPageChange(currentPage + 1)}
            disabled={currentPage >= totalPages}
            className="inline-flex h-6 w-6 items-center justify-center rounded border border-hairline-strong text-charcoal transition-colors hover:bg-surface-elevated disabled:pointer-events-none disabled:opacity-30"
            aria-label="Next page"
          >
            <ChevronRight className="size-3" />
          </button>
        </div>
      </div>
    </div>
  );
}
