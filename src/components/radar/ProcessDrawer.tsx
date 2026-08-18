import { useEffect, useState } from "react";
import { Ban, ShieldCheck, ShieldQuestion, X } from "lucide-react";
import { getProcessInfo, signatureStatus } from "../../api/commands";
import type { ConnectionRow, ProcessDetail } from "../../api/types";
import { formatBytes, formatRate } from "../../lib/format";

export function ProcessDrawer({
  row,
  allRows,
  onClose,
  onTriggerBlock,
}: {
  row: ConnectionRow;
  allRows: ConnectionRow[];
  onClose: () => void;
  onTriggerBlock: (target: "process" | "host") => void;
}) {
  const [detail, setDetail] = useState<ProcessDetail | null>(null);
  const [msg, setMsg] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setDetail(null);
    setMsg(null);
    getProcessInfo(row.pid)
      .then((d) => {
        if (cancelled) return;
        setDetail(d);
        if (d.path) {
          return signatureStatus(d.path).then((s) => {
            if (cancelled) return;
            setDetail({ ...d, signed: s.signed, publisher: s.publisher });
          });
        }
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [row.pid]);

  const rows = allRows.filter((r) => r.pid === row.pid);
  const up = rows.reduce((a, r) => a + r.bytes_out, 0);
  const down = rows.reduce((a, r) => a + r.bytes_in, 0);

  return (
    <aside className="flex w-80 shrink-0 flex-col border-l border-base-700 bg-base-900">
      <div className="flex items-center gap-2 border-b border-base-700 px-4 py-3">
        <h2 className="truncate text-sm font-semibold text-slate-100">
          {detail?.name ?? row.process}
        </h2>
        <span className="font-mono text-[10px] text-slate-500">[{row.pid}]</span>
        <button
          className="ml-auto text-slate-500 hover:text-slate-200"
          onClick={onClose}
          aria-label="Close"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="flex-1 space-y-4 overflow-auto px-4 py-4 text-xs">
        <div className="grid grid-cols-2 gap-2">
          <Stat label="Up" value={formatRate(up)} color="text-neon-emerald" />
          <Stat label="Down" value={formatRate(down)} color="text-neon-cyan" />
          <Stat label="Connections" value={String(rows.length)} />
          <Stat
            label="Total bytes"
            value={formatBytes(up + down)}
            color="text-slate-200"
          />
        </div>

        <div>
          <div className="mb-1 text-[10px] uppercase tracking-wider text-slate-500">
            Executable
          </div>
          <div className="break-all font-mono text-[11px] text-slate-300">
            {detail?.path ?? "—"}
          </div>
        </div>

        <div>
          <div className="mb-1 text-[10px] uppercase tracking-wider text-slate-500">
            Publisher
          </div>
          <div className="flex items-center gap-1.5 text-[11px] text-slate-300">
            {detail?.signed === null ? (
              <ShieldQuestion className="h-3.5 w-3.5 text-slate-500" />
            ) : detail?.signed ? (
              <ShieldCheck className="h-3.5 w-3.5 text-neon-emerald" />
            ) : (
              <ShieldQuestion className="h-3.5 w-3.5 text-neon-amber" />
            )}
            {detail?.publisher ?? "Unknown"}
          </div>
          <div className="mt-1 text-[11px] text-slate-500">
            {detail?.signed === null
              ? "Signature not checked"
              : detail?.signed
                ? "Digitally signed"
                : "Unsigned"}
          </div>
        </div>

        <div>
          <div className="mb-1 text-[10px] uppercase tracking-wider text-slate-500">
            Live connections
          </div>
          <div className="space-y-1">
            {rows.map((r, i) => (
              <div
                key={i}
                className="flex items-center justify-between rounded border border-base-800 bg-base-850 px-2 py-1.5 font-mono text-[10px]"
              >
                <span className="truncate text-slate-400">
                  {r.remote_addr ?? r.local_addr}:{r.remote_port ?? r.local_port}
                </span>
                <span className="ml-2 shrink-0 text-neon-cyan">
                  {r.bytes_in > 0 ? formatRate(r.bytes_in) : "—"}
                </span>
              </div>
            ))}
            {rows.length === 0 && (
              <div className="text-slate-600">No active sockets.</div>
            )}
          </div>
        </div>

        <div className="space-y-2">
          <button
            disabled={!detail?.path}
            onClick={() => onTriggerBlock("process")}
            className="flex w-full items-center justify-center gap-2 rounded-md border border-neon-red/40 bg-neon-red/10 px-3 py-2 text-xs font-semibold text-neon-red hover:bg-neon-red/20 disabled:cursor-not-allowed disabled:opacity-40"
          >
            <Ban className="h-3.5 w-3.5" />
            Isolate outbound traffic
          </button>
        </div>

        {msg && <div className="text-[11px] text-slate-400">{msg}</div>}
      </div>
    </aside>
  );
}

function Stat({
  label,
  value,
  color = "text-slate-200",
}: {
  label: string;
  value: string;
  color?: string;
}) {
  return (
    <div className="rounded-md border border-base-800 bg-base-850 px-2.5 py-2">
      <div className="text-[9px] uppercase tracking-wider text-slate-500">
        {label}
      </div>
      <div className={`mt-0.5 font-mono text-xs ${color}`}>{value}</div>
    </div>
  );
}
