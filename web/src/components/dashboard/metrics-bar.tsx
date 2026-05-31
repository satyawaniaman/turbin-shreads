"use client";

import { Activity, Layers, Clock } from "lucide-react";
import { formatNumber } from "@/lib/live-events";

interface MetricsBarProps {
  totalEvents: number;
  eventsPerSecond: number;
  latestSlot: number;
  fecRecovered: number;
  avgLatency: number;
}

interface MetricCardProps {
  label: string;
  value: string;
  icon: React.ReactNode;
  accentColor?: string;
  glowColor?: string;
}

function MetricCard({ label, value, icon, accentColor, glowColor }: MetricCardProps) {
  return (
    <div
      className="group relative flex flex-1 flex-col gap-2 overflow-hidden rounded-xl border border-hairline-strong bg-surface-card p-4 transition-all hover:border-hairline-strong/80"
      style={{
        boxShadow: glowColor ? `inset 0 1px 24px ${glowColor}` : undefined,
      }}
    >
      {/* Subtle top glow line */}
      {accentColor && (
        <div
          className="absolute inset-x-0 top-0 h-px opacity-40 transition-opacity group-hover:opacity-70"
          style={{ backgroundColor: accentColor }}
        />
      )}
      <div className="flex items-center gap-2">
        <span style={{ color: accentColor ?? "#a1a4a5" }}>{icon}</span>
        <span className="text-xs text-mute-text">{label}</span>
      </div>
      <span className="font-mono text-xl font-medium tracking-tight text-ink">
        {value}
      </span>
    </div>
  );
}

export function MetricsBar({
  totalEvents,
  latestSlot,
  avgLatency,
}: Omit<MetricsBarProps, "eventsPerSecond" | "fecRecovered">) {
  return (
    <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
      <MetricCard
        label="Total Events"
        value={formatNumber(totalEvents)}
        icon={<Activity className="size-3.5" />}
        accentColor="#3b9eff"
        glowColor="rgba(0, 117, 255, 0.06)"
      />
      <MetricCard
        label="Latest Slot"
        value={formatNumber(latestSlot)}
        icon={<Layers className="size-3.5" />}
        accentColor="#ff801f"
        glowColor="rgba(255, 89, 0, 0.06)"
      />
      <MetricCard
        label="Avg Latency"
        value={`${avgLatency}ms`}
        icon={<Clock className="size-3.5" />}
        accentColor="#ff2047"
        glowColor="rgba(255, 32, 71, 0.06)"
      />
    </div>
  );
}
