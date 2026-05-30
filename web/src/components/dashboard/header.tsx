"use client";

import { Pause, Play, Code2, Zap } from "lucide-react";

interface HeaderProps {
  isStreaming: boolean;
  streamSpeed: number;
  onToggleStreaming: () => void;
  onSetStreamSpeed: (ms: number) => void;
}

export function Header({
  isStreaming,
  streamSpeed,
  onToggleStreaming,
  onSetStreamSpeed,
}: HeaderProps) {
  return (
    <header className="flex h-16 shrink-0 items-center justify-between border-b border-hairline px-4 md:px-6">
      {/* ── Left: Branding ── */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <Zap className="size-5 text-accent-orange" />
          <h1 className="font-display text-lg font-medium tracking-tight text-ink md:text-xl">
            ShredStream
          </h1>
        </div>
        <span className="hidden text-xs text-mute-text sm:inline">
          Turbine Shred Decoder
        </span>
      </div>

      {/* ── Center: Status ── */}
      <div className="flex items-center gap-2">
        <div
          className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium ${
            isStreaming
              ? "bg-accent-green/10 text-accent-green"
              : "bg-accent-yellow/10 text-accent-yellow"
          }`}
        >
          <span
            className={`inline-block size-1.5 rounded-full ${
              isStreaming
                ? "animate-pulse-dot bg-accent-green"
                : "bg-accent-yellow"
            }`}
          />
          {isStreaming ? "Live" : "Paused"}
        </div>
      </div>

      {/* ── Right: Controls ── */}
      <div className="flex items-center gap-2">
        {/* Speed selector */}
        <select
          value={streamSpeed}
          onChange={(e) => onSetStreamSpeed(Number(e.target.value))}
          className="hidden h-8 rounded-lg border border-hairline-strong bg-surface-card px-2 text-xs text-charcoal outline-none transition-colors focus:border-ink sm:block"
          aria-label="Stream speed"
        >
          <option value={2000}>Real-time (2s)</option>
          <option value={5000}>Fast (5s)</option>
          <option value={10000}>Normal (10s)</option>
          <option value={30000}>Slow (30s)</option>
        </select>

        {/* Play/Pause */}
        <button
          onClick={onToggleStreaming}
          className="inline-flex h-8 items-center gap-1.5 rounded-lg border border-hairline-strong bg-surface-card px-3 text-xs font-medium text-ink transition-colors hover:bg-surface-elevated"
          aria-label={isStreaming ? "Pause stream" : "Resume stream"}
        >
          {isStreaming ? (
            <>
              <Pause className="size-3" />
              <span className="hidden sm:inline">Pause</span>
            </>
          ) : (
            <>
              <Play className="size-3" />
              <span className="hidden sm:inline">Resume</span>
            </>
          )}
        </button>

        {/* GitHub */}
        <a
          href="https://github.com"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex h-8 w-8 items-center justify-center rounded-lg border border-hairline-strong bg-surface-card text-charcoal transition-colors hover:bg-surface-elevated hover:text-ink"
          aria-label="View source on GitHub"
        >
          <Code2 className="size-3.5" />
        </a>
      </div>
    </header>
  );
}
