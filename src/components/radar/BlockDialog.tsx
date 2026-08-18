import { useState } from "react";
import { Ban, X } from "lucide-react";
import { blockHost, blockProcessOutbound } from "../../api/commands";
import type { ConnectionRow } from "../../api/types";
import { ElevationBanner } from "../shell/ElevationBanner";

export function BlockDialog({
  row,
  target,
  onClose,
  onDone,
}: {
  row: ConnectionRow;
  target: "process" | "host";
  onClose: () => void;
  onDone: (ok: boolean, detail: string) => void;
}) {
  const [busy, setBusy] = useState(false);

  const isProcess = target === "process";

  const execute = async () => {
    setBusy(true);
    try {
      const outcome = isProcess
        ? await blockProcessOutbound(
            row.process_path ?? "",
            `${row.process}.exe`,
          )
        : await blockHost(row.remote_addr ?? row.remote_host ?? "");
      onDone(outcome.ok, outcome.detail);
    } catch (e) {
      onDone(false, String(e));
    }
  };

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-xl border border-base-600 bg-base-900 p-5 shadow-2xl">
        <div className="mb-3 flex items-center gap-2">
          <Ban className="h-5 w-5 text-neon-red" />
          <h2 className="text-base font-semibold text-slate-100">
            {isProcess ? "Isolate process" : "Block remote host"}
          </h2>
          <button
            className="ml-auto text-slate-500 hover:text-slate-200"
            onClick={onClose}
            aria-label="Close"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {isProcess ? (
          <div className="mb-4 rounded-lg border border-base-700 bg-base-850 p-3">
            <div className="text-[10px] uppercase tracking-wider text-slate-500">
              Process
            </div>
            <div className="mt-0.5 break-all font-mono text-xs text-slate-200">
              {row.process}
            </div>
            <div className="mt-1 text-[10px] uppercase tracking-wider text-slate-500">
              Executable
            </div>
            <div className="mt-0.5 break-all font-mono text-xs text-slate-400">
              {row.process_path ?? "unknown"}
            </div>
            <p className="mt-2 text-[11px] text-slate-400">
              Creates a Windows Firewall outbound block rule for this
              executable. The process itself is not touched.
            </p>
          </div>
        ) : (
          <div className="mb-4 rounded-lg border border-base-700 bg-base-850 p-3">
            <div className="text-[10px] uppercase tracking-wider text-slate-500">
              Remote host
            </div>
            <div className="mt-0.5 font-mono text-xs text-neon-cyan">
              {row.remote_addr ?? row.remote_host}
              {row.remote_host && row.remote_host !== row.remote_addr
                ? ` (${row.remote_host})`
                : ""}
            </div>
            <p className="mt-2 text-[11px] text-slate-400">
              Blocks all outbound traffic to this address via a Windows
              Firewall rule.
            </p>
          </div>
        )}

        <div className="mb-4">
          <ElevationBanner
            reason="Adding a Windows Firewall rule requires administrator rights."
          />
        </div>

        <div className="flex justify-end gap-2">
          <button
            onClick={onClose}
            disabled={busy}
            className="rounded-md border border-base-600 px-3 py-1.5 text-sm text-slate-300 hover:bg-base-800"
          >
            Cancel
          </button>
          <button
            onClick={() => void execute()}
            disabled={busy}
            className="rounded-md bg-neon-red px-3 py-1.5 text-sm font-semibold text-black hover:brightness-110 disabled:opacity-50"
          >
            {busy ? "Waiting for UAC…" : "Block outbound"}
          </button>
        </div>
      </div>
    </div>
  );
}
