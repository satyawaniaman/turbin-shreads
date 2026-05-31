"use client";

import { useState, useCallback, useMemo, useEffect, useRef } from "react";
import {
  type DecodedEvent,
  type Protocol,
  type InstructionKind,
  type LiveEventsResponse,
  PROTOCOL_LIST,
} from "@/lib/live-events";

export interface DashboardState {
  events: DecodedEvent[];
  filteredEvents: DecodedEvent[];
  paginatedEvents: DecodedEvent[];
  selectedEvent: DecodedEvent | null;
  searchQuery: string;
  protocolFilter: Protocol | "all";
  kindFilter: InstructionKind | "all";
  isStreaming: boolean;
  streamSpeed: number;
  error: string | null;
  lastUpdated: number | null;
  source: string;
  perPage: number;
  currentPage: number;
  totalPages: number;

  // Computed metrics
  totalEvents: number;
  eventsPerSecond: number;
  latestSlot: number;
  fecRecovered: number;
  avgLatency: number;
  protocolCounts: Record<Protocol, number>;

  // Actions
  setSearchQuery: (q: string) => void;
  setProtocolFilter: (p: Protocol | "all") => void;
  setKindFilter: (k: InstructionKind | "all") => void;
  setSelectedEvent: (e: DecodedEvent | null) => void;
  toggleStreaming: () => void;
  setStreamSpeed: (ms: number) => void;
  setPerPage: (n: number) => void;
  setCurrentPage: (p: number) => void;
  clearEvents: () => void;
  exportCSV: () => void;
}

export function useDashboard(): DashboardState {
  const [events, setEvents] = useState<DecodedEvent[]>([]);
  const [selectedEvent, setSelectedEvent] = useState<DecodedEvent | null>(null);
  const [searchQuery, setSearchQueryState] = useState("");
  const [protocolFilter, setProtocolFilterState] = useState<Protocol | "all">("all");
  const [kindFilter, setKindFilterState] = useState<InstructionKind | "all">("all");
  const [isStreaming, setIsStreaming] = useState(true);
  const [streamSpeed, setStreamSpeed] = useState(2000);
  const [eventsPerSecond, setEventsPerSecond] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<number | null>(null);
  const [source, setSource] = useState<string>("");
  const [perPage, setPerPage] = useState(25);
  const [currentPage, setCurrentPage] = useState(1);

  // Track newly discovered signatures in the last window for EPS calc.
  const recentCountRef = useRef(0);
  const seenSignaturesRef = useRef<Set<string>>(new Set());

  // ── Filtered events (newest first) ──
  const filteredEvents = useMemo(() => {
    let result = events;

    if (protocolFilter !== "all") {
      result = result.filter((e) => e.protocol === protocolFilter);
    }

    if (kindFilter !== "all") {
      result = result.filter((e) => e.kind === kindFilter);
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      result = result.filter(
        (e) =>
          e.signature.toLowerCase().includes(q) ||
          e.mint.toLowerCase().includes(q) ||
          e.authority.toLowerCase().includes(q) ||
          (e.inputMint && e.inputMint.toLowerCase().includes(q)) ||
          (e.outputMint && e.outputMint.toLowerCase().includes(q))
      );
    }

    // Sort newest first (highest timestamp / slot at top)
    return [...result].sort((a, b) => b.timestamp - a.timestamp);
  }, [events, protocolFilter, kindFilter, searchQuery]);

  // ── Pagination ──
  const totalPages = Math.max(1, Math.ceil(filteredEvents.length / perPage));

  const paginatedEvents = useMemo(() => {
    const start = (currentPage - 1) * perPage;
    return filteredEvents.slice(start, start + perPage);
  }, [filteredEvents, currentPage, perPage]);

  // ── Computed metrics ──
  const totalEvents = events.length;

  const latestSlot = useMemo(
    () => (events.length > 0 ? Math.max(...events.map((e) => e.slot)) : 0),
    [events]
  );

  const fecRecovered = useMemo(
    () => events.filter((e) => e.fecStatus === "recovered").length,
    [events]
  );

  const avgLatency = useMemo(() => {
    if (events.length === 0) return 0;
    const sum = events.reduce((acc, e) => acc + e.decodeLatencyMs, 0);
    return parseFloat((sum / events.length).toFixed(1));
  }, [events]);

  const protocolCounts = useMemo(() => {
    const counts = {} as Record<Protocol, number>;
    for (const p of PROTOCOL_LIST) counts[p] = 0;
    for (const e of events) counts[e.protocol]++;
    return counts;
  }, [events]);

  const fetchLiveEvents = useCallback(async () => {
    const response = await fetch("/api/events", { cache: "no-store" });
    const payload = await response.json();

    if (!response.ok) {
      throw new Error(payload.error || "Failed to fetch live events");
    }

    const data = payload as LiveEventsResponse;
    setLastUpdated(data.fetchedAt);
    setSource(data.source);
    setError(null);
    setEvents((prev) => {
      const merged = new Map(prev.map((event) => [event.signature, event]));
      let newCount = 0;

      for (const event of data.events) {
        if (!seenSignaturesRef.current.has(event.signature)) {
          newCount++;
          seenSignaturesRef.current.add(event.signature);
        }
        merged.set(event.signature, event);
      }

      recentCountRef.current += newCount;
      return [...merged.values()]
        .sort((a, b) => b.timestamp - a.timestamp)
        .slice(0, 250);
    });
  }, []);

  // ── Live Helius polling ──
  useEffect(() => {
    if (!isStreaming) return;

    let cancelled = false;
    const load = async () => {
      try {
        await fetchLiveEvents();
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to fetch live events");
        }
      }
    };

    load();
    const interval = setInterval(load, streamSpeed);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [fetchLiveEvents, isStreaming, streamSpeed]);

  // ── EPS calculation ──
  useEffect(() => {
    const interval = setInterval(() => {
      setEventsPerSecond(
        parseFloat((recentCountRef.current / 3).toFixed(1))
      );
      recentCountRef.current = 0;
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  // ── Actions ──
  const toggleStreaming = useCallback(() => {
    setIsStreaming((prev) => !prev);
  }, []);

  const setSearchQuery = useCallback((q: string) => {
    setSearchQueryState(q);
    setCurrentPage(1);
  }, []);

  const setProtocolFilter = useCallback((p: Protocol | "all") => {
    setProtocolFilterState(p);
    setCurrentPage(1);
  }, []);

  const setKindFilter = useCallback((k: InstructionKind | "all") => {
    setKindFilterState(k);
    setCurrentPage(1);
  }, []);

  const updatePerPage = useCallback((n: number) => {
    setPerPage(n);
    setCurrentPage(1);
  }, []);

  const clearEvents = useCallback(() => {
    seenSignaturesRef.current.clear();
    recentCountRef.current = 0;
    setEvents([]);
    setSelectedEvent(null);
    setCurrentPage(1);
  }, []);

  const exportCSV = useCallback(() => {
    const headers = [
      "slot",
      "protocol",
      "kind",
      "signature",
      "mint",
      "input_mint",
      "output_mint",
      "input_amount",
      "output_amount",
      "slippage_bps",
      "authority",
      "fec_status",
      "decode_latency_ms",
      "timestamp",
    ];

    const rows = filteredEvents.map((e) =>
      [
        e.slot,
        e.protocol,
        e.kind,
        e.signature,
        e.mint,
        e.inputMint ?? "",
        e.outputMint ?? "",
        e.inputAmount ?? "",
        e.outputAmount ?? "",
        e.slippageBps ?? "",
        e.authority,
        e.fecStatus,
        e.decodeLatencyMs,
        new Date(e.timestamp).toISOString(),
      ].join(",")
    );

    const csv = [headers.join(","), ...rows].join("\n");
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `shredstream-events-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }, [filteredEvents]);

  return {
    events,
    filteredEvents,
    paginatedEvents,
    selectedEvent,
    searchQuery,
    protocolFilter,
    kindFilter,
    isStreaming,
    streamSpeed,
    error,
    lastUpdated,
    source,
    perPage,
    currentPage,
    totalPages,
    totalEvents,
    eventsPerSecond,
    latestSlot,
    fecRecovered,
    avgLatency,
    protocolCounts,
    setSearchQuery,
    setProtocolFilter,
    setKindFilter,
    setSelectedEvent,
    toggleStreaming,
    setStreamSpeed,
    setPerPage: updatePerPage,
    setCurrentPage,
    clearEvents,
    exportCSV,
  };
}
