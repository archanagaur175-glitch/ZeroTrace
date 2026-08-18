import { useMemo, useState } from "react";
import { Ban, ChevronRight, Search } from "lucide-react";
import type { ConnectionRow } from "../../api/types";
import { formatAddr, formatBytes, formatRate } from "../../lib/format";

type ZoneFilter = "all" | "internet" | "lan" | "local";

const ZONE_LABEL: Record<ConnectionRow["zone"], string> = {
  internet: "Internet",
  lan: "LAN",
  local: "Localhost",
  unknown: "Unknown",
};

const ZONE_COLOR: Record<ConnectionRow["zone"], string> = {
  internet: "text-neon-cyan",
  lan: "text-slate-300",
  local: "text-slate-500",
  unknown: "text-neon-amber",
};

export function ConnectionTable({
  rows,
  onSelect,
  onBlock,
}: {
  rows: ConnectionRow[];
  onSelect: (r: ConnectionRow) => void;
  onBlock: (r: ConnectionRow) => void;
}) {
  const [query, setQuery] = useState("");
  const [zone, setZone] = useState<ZoneFilter>("all");

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return rows
      .filter((r) => {
        if (zone !== "all" && r.zone !== zone) return false;
        if (!q) return true;
        return (
          r.process.toLowerCase().includes(q) ||
          (r.remote_host ?? "").toLowerCase().includes(q) ||
          (r.remote_addr ?? "").toLowerCase().includes(q)
        );
      })
      .sort((a, b) => b.bytes_out + b.bytes_in - (a.bytes_out + a.bytes_in))
      .slice(0, 200);
  }, [rows, query, zone]);

  const filters: { id: ZoneFilter; label: string }[] = [
    { id: "all", label: "All" },
    { id: "internet", label: "Internet" },
    { id: "lan", label: "LAN" },
    { id: "local", label: "Local" },
  ];

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 px-1 pb-2">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-500" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Filter process, host, or IP"
            className="w-full rounded-md border border-base-700 bg-base-900 py-1.5 pl-8 pr-3 text-xs text-slate-200 placeholder:text-slate-600 focus:border-neon-cyan/50 focus:outline-none"
          />
        </div>
        <div className="flex gap-1">
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => setZone(f.id)}
              className={`rounded-md px-2.5 py-1 text-[11px] font-medium ${
                zone === f.id
                  ? "bg-neon-cyan/10 text-neon-cyan"
                  : "text-slate-500 hover:text-slate-300"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        <table className="w-full text-left text-xs">
          <thead className="sticky top-0 z-10 bg-base-950 text-[10px] uppercase tracking-wider text-slate-500">
            <tr>
              <th className="px-2 py-1.5">Process</th>
              <th className="px-2 py-1.5">Proto</th>
              <th className="px-2 py-1.5">State</th>
              <th className="px-2 py-1.5">Local</th>
              <th className="px-2 py-1.5">Remote</th>
              <th className="px-2 py-1.5">Zone</th>
              <th className="px-2 py-1.5 text-right">Up</th>
              <th className="px-2 py-1.5 text-right">Down</th>
              <th className="px-2 py-1.5" />
            </tr>
          </thead>
          <tbody>
            {filtered.map((r, i) => (
              <tr
                key={i}
                onClick={() => onSelect(r)}
                className="cursor-pointer border-t border-base-800 hover:bg-base-800/60"
              >
                <td className="px-2 py-1.5 font-medium text-slate-200">
                  {r.process}
                  <span className="ml-1.5 font-mono text-[10px] text-slate-500">
                    [{r.pid}]
                  </span>
                  {r.track === "churn" && (
                    <span
                      className="ml-1.5 rounded border border-neon-amber/30 bg-neon-amber/10 px-1 text-[9px] text-neon-amber"
                      title="Short-lived connection: byte tracking unavailable; activity inferred from connection churn"
                    >
                      churn
                    </span>
                  )}
                </td>
                <td className="px-2 py-1.5 font-mono text-slate-400">
                  {r.proto}
                </td>
                <td className="px-2 py-1.5 text-slate-400">{r.state}</td>
                <td className="px-2 py-1.5 font-mono text-slate-500">
                  {formatAddr(r.local_addr, r.local_port)}
                </td>
                <td className="px-2 py-1.5 font-mono text-slate-300">
                  {r.remote_addr
                    ? formatAddr(r.remote_addr, r.remote_port)
                    : "—"}
                  {r.remote_host && r.remote_host !== r.remote_addr && (
                    <span className="ml-1.5 text-slate-500">
                      ({r.remote_host})
                    </span>
                  )}
                </td>
                <td className={`px-2 py-1.5 ${ZONE_COLOR[r.zone]}`}>
                  {ZONE_LABEL[r.zone]}
                </td>
                <td className="px-2 py-1.5 text-right font-mono text-neon-emerald">
                  {r.bytes_out > 0 ? formatRate(r.bytes_out) : "—"}
                </td>
                <td className="px-2 py-1.5 text-right font-mono text-neon-cyan">
                  {r.bytes_in > 0 ? formatRate(r.bytes_in) : "—"}
                </td>
                <td className="px-2 py-1.5">
                  {r.remote_addr && r.zone !== "local" && (
                    <button
                      title="Isolate / block"
                      onClick={(e) => {
                        e.stopPropagation();
                        onBlock(r);
                      }}
                      className="rounded p-1 text-slate-500 hover:bg-neon-red/10 hover:text-neon-red"
                    >
                      <Ban className="h-3.5 w-3.5" />
                    </button>
                  )}
                  <ChevronRight className="ml-1 inline h-3.5 w-3.5 text-slate-600" />
                </td>
              </tr>
            ))}
            {filtered.length === 0 && (
              <tr>
                <td
                  colSpan={9}
                  className="px-2 py-8 text-center text-slate-600"
                >
                  No connections match.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      <div className="flex items-center justify-between border-t border-base-800 px-1 pt-1.5 text-[10px] uppercase tracking-wide text-slate-600">
        <span>{rows.length} tracked connections</span>
        <span>{formatBytes(sum(rows, "in") + sum(rows, "out"))} total this tick</span>
      </div>
    </div>
  );
}

function sum(rows: ConnectionRow[], key: "in" | "out"): number {
  return rows.reduce((acc, r) => acc + (key === "in" ? r.bytes_in : r.bytes_out), 0);
}
