"use client";

import { Search, X, Download, Trash2 } from "lucide-react";
import {
  type Protocol,
  type InstructionKind,
  PROTOCOL_CONFIG,
  PROTOCOL_LIST,
  KIND_LIST,
} from "@/lib/live-events";

interface FiltersBarProps {
  searchQuery: string;
  protocolFilter: Protocol | "all";
  kindFilter: InstructionKind | "all";
  filteredCount: number;
  totalCount: number;
  onSearchChange: (q: string) => void;
  onProtocolChange: (p: Protocol | "all") => void;
  onKindChange: (k: InstructionKind | "all") => void;
  onExportCSV: () => void;
  onClearEvents: () => void;
}

export function FiltersBar({
  searchQuery,
  protocolFilter,
  kindFilter,
  filteredCount,
  totalCount,
  onSearchChange,
  onProtocolChange,
  onKindChange,
  onExportCSV,
  onClearEvents,
}: FiltersBarProps) {
  const hasActiveFilters =
    searchQuery.trim() !== "" ||
    protocolFilter !== "all" ||
    kindFilter !== "all";

  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      {/* ── Left: Search + Filters ── */}
      <div className="flex flex-1 flex-col gap-2 sm:flex-row sm:items-center">
        {/* Search */}
        <div className="relative flex-1 sm:max-w-xs">
          <Search className="absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-mute-text" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search signature, mint, authority…"
            className="h-8 w-full rounded-lg border border-hairline-strong bg-surface-card pl-8 pr-8 text-xs text-ink placeholder:text-stone outline-none transition-colors focus:border-ink"
            id="search-events"
          />
          {searchQuery && (
            <button
              onClick={() => onSearchChange("")}
              className="absolute right-2 top-1/2 -translate-y-1/2 rounded p-0.5 text-mute-text hover:text-ink"
              aria-label="Clear search"
            >
              <X className="size-3" />
            </button>
          )}
        </div>

        {/* Protocol filter */}
        <select
          value={protocolFilter}
          onChange={(e) =>
            onProtocolChange(e.target.value as Protocol | "all")
          }
          className="h-8 rounded-lg border border-hairline-strong bg-surface-card px-2 text-xs text-charcoal outline-none transition-colors focus:border-ink"
          aria-label="Filter by protocol"
          id="filter-protocol"
        >
          <option value="all">All Protocols</option>
          {PROTOCOL_LIST.map((p) => (
            <option key={p} value={p}>
              {PROTOCOL_CONFIG[p].label}
            </option>
          ))}
        </select>

        {/* Kind filter */}
        <select
          value={kindFilter}
          onChange={(e) =>
            onKindChange(e.target.value as InstructionKind | "all")
          }
          className="h-8 rounded-lg border border-hairline-strong bg-surface-card px-2 text-xs text-charcoal outline-none transition-colors focus:border-ink"
          aria-label="Filter by instruction kind"
          id="filter-kind"
        >
          <option value="all">All Kinds</option>
          {KIND_LIST.map((k) => (
            <option key={k} value={k}>
              {k}
            </option>
          ))}
        </select>

        {/* Active filter indicator */}
        {hasActiveFilters && (
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-mute-text">
              {filteredCount} of {totalCount}
            </span>
            <button
              onClick={() => {
                onSearchChange("");
                onProtocolChange("all");
                onKindChange("all");
              }}
              className="text-xs text-accent-blue transition-colors hover:text-ink"
            >
              Clear all
            </button>
          </div>
        )}
      </div>

      {/* ── Right: Actions ── */}
      <div className="flex items-center gap-2">
        <button
          onClick={onExportCSV}
          className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-hairline-strong bg-surface-card px-3 text-xs font-medium text-charcoal transition-colors hover:bg-surface-elevated hover:text-ink"
          aria-label="Export filtered events as CSV"
          id="export-csv"
        >
          <Download className="size-3" />
          <span className="hidden sm:inline">Export CSV</span>
        </button>
        <button
          onClick={onClearEvents}
          className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-hairline-strong bg-surface-card px-3 text-xs font-medium text-charcoal transition-colors hover:border-accent-red/30 hover:bg-accent-red/5 hover:text-accent-red"
          aria-label="Clear all events"
          id="clear-events"
        >
          <Trash2 className="size-3" />
          <span className="hidden sm:inline">Clear</span>
        </button>
      </div>
    </div>
  );
}
