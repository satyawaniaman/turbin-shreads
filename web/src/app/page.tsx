"use client";

import { useDashboard } from "@/hooks/use-dashboard";
import { Header } from "@/components/dashboard/header";
import { MetricsBar } from "@/components/dashboard/metrics-bar";
import { ProtocolBreakdown } from "@/components/dashboard/protocol-breakdown";
import { FiltersBar } from "@/components/dashboard/filters-bar";
import { EventsTable } from "@/components/dashboard/events-table";
import { EventInspector } from "@/components/dashboard/event-inspector";

export default function Home() {
  const dashboard = useDashboard();

  return (
    <div className="flex h-screen flex-col bg-canvas">
      <Header
        isStreaming={dashboard.isStreaming}
        streamSpeed={dashboard.streamSpeed}
        onToggleStreaming={dashboard.toggleStreaming}
        onSetStreamSpeed={dashboard.setStreamSpeed}
      />

      <div className="flex flex-1 overflow-hidden">
        {/* ── Main content ── */}
        <main className="flex flex-1 flex-col gap-4 overflow-y-auto p-4 md:p-6">
          {dashboard.error && (
            <div className="rounded-xl border border-accent-red/30 bg-accent-red/5 px-4 py-3 text-sm text-accent-red">
              Live feed error: {dashboard.error}
            </div>
          )}

          {dashboard.lastUpdated && (
            <div className="text-xs text-mute-text">
              Source: Helius mainnet RPC · Last updated{" "}
              {new Date(dashboard.lastUpdated).toLocaleTimeString()}
            </div>
          )}

          {/* Metrics */}
          <MetricsBar
            totalEvents={dashboard.totalEvents}
            eventsPerSecond={dashboard.eventsPerSecond}
            latestSlot={dashboard.latestSlot}
            fecRecovered={dashboard.fecRecovered}
            avgLatency={dashboard.avgLatency}
          />

          {/* Protocol breakdown + Filters */}
          <div className="grid gap-4 lg:grid-cols-[280px_1fr]">
            <ProtocolBreakdown
              protocolCounts={dashboard.protocolCounts}
              totalEvents={dashboard.totalEvents}
              activeFilter={dashboard.protocolFilter}
              onFilterClick={dashboard.setProtocolFilter}
            />

            <div className="flex flex-col gap-3">
              <FiltersBar
                searchQuery={dashboard.searchQuery}
                protocolFilter={dashboard.protocolFilter}
                kindFilter={dashboard.kindFilter}
                filteredCount={dashboard.filteredEvents.length}
                totalCount={dashboard.totalEvents}
                onSearchChange={dashboard.setSearchQuery}
                onProtocolChange={dashboard.setProtocolFilter}
                onKindChange={dashboard.setKindFilter}
                onExportCSV={dashboard.exportCSV}
                onClearEvents={dashboard.clearEvents}
              />

              <EventsTable
                events={dashboard.paginatedEvents}
                selectedEvent={dashboard.selectedEvent}
                onSelectEvent={dashboard.setSelectedEvent}
                currentPage={dashboard.currentPage}
                totalPages={dashboard.totalPages}
                perPage={dashboard.perPage}
                totalFiltered={dashboard.filteredEvents.length}
                onPageChange={dashboard.setCurrentPage}
                onPerPageChange={dashboard.setPerPage}
              />
            </div>
          </div>
        </main>

        {/* ── Inspector panel ── */}
        <EventInspector
          event={dashboard.selectedEvent}
          onClose={() => dashboard.setSelectedEvent(null)}
        />
      </div>
    </div>
  );
}
