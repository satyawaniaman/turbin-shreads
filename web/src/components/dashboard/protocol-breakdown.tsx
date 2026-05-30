"use client";

import {
  type Protocol,
  PROTOCOL_CONFIG,
  PROTOCOL_LIST,
} from "@/lib/live-events";

interface ProtocolBreakdownProps {
  protocolCounts: Record<Protocol, number>;
  totalEvents: number;
  activeFilter: Protocol | "all";
  onFilterClick: (p: Protocol | "all") => void;
}

export function ProtocolBreakdown({
  protocolCounts,
  totalEvents,
  activeFilter,
  onFilterClick,
}: ProtocolBreakdownProps) {
  return (
    <div className="rounded-xl border border-hairline-strong bg-surface-card p-4">
      <div className="mb-3 flex items-center justify-between">
        <h2 className="text-sm font-medium text-charcoal">
          Protocol Breakdown
        </h2>
        {activeFilter !== "all" && (
          <button
            onClick={() => onFilterClick("all")}
            className="text-xs text-mute-text transition-colors hover:text-ink"
          >
            Clear filter
          </button>
        )}
      </div>

      <div className="space-y-2.5">
        {PROTOCOL_LIST.map((protocol) => {
          const count = protocolCounts[protocol];
          const pct = totalEvents > 0 ? (count / totalEvents) * 100 : 0;
          const config = PROTOCOL_CONFIG[protocol];
          const isActive = activeFilter === protocol;

          return (
            <button
              key={protocol}
              onClick={() =>
                onFilterClick(isActive ? "all" : protocol)
              }
              className={`group flex w-full items-center gap-3 rounded-lg px-2 py-1.5 text-left transition-all ${
                isActive
                  ? "bg-surface-elevated"
                  : "hover:bg-surface-elevated/50"
              }`}
            >
              {/* Color dot */}
              <span
                className="inline-block size-2 shrink-0 rounded-full"
                style={{ backgroundColor: config.color }}
              />

              {/* Label + bar */}
              <div className="flex flex-1 flex-col gap-1">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-medium text-ink">
                    {config.label}
                  </span>
                  <span className="font-mono text-xs text-mute-text">
                    {count}
                    <span className="ml-1 text-stone">
                      ({pct.toFixed(0)}%)
                    </span>
                  </span>
                </div>
                {/* Progress bar */}
                <div className="h-1 w-full overflow-hidden rounded-full bg-surface-elevated">
                  <div
                    className="h-full rounded-full transition-all duration-500"
                    style={{
                      width: `${Math.max(pct, 1)}%`,
                      backgroundColor: config.color,
                      boxShadow: `0 0 8px ${config.glow}`,
                    }}
                  />
                </div>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
