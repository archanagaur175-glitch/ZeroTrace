import { useEffect, useMemo, useState } from "react";
import { getSnapshot } from "../../api/commands";
import { subscribeSnapshots } from "../../api/events";
import type { ConnectionRow } from "../../api/types";
import { ingestSnapshot, useHistory, useSnapshot } from "../../state/radarStore";
import { BandwidthGraph } from "./BandwidthGraph";
import { ConnectionTable } from "./ConnectionTable";
import { ProcessDrawer } from "./ProcessDrawer";
import { BlockDialog } from "./BlockDialog";

export function RadarScreen() {
  const snapshot = useSnapshot();
  const history = useHistory();
  const [selected, setSelected] = useState<ConnectionRow | null>(null);
  const [block, setBlock] = useState<{
    row: ConnectionRow;
    target: "process" | "host";
  } | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    getSnapshot()
      .then((s) => {
        if (alive) ingestSnapshot(s);
      })
      .catch(() => undefined);
    const unsub = subscribeSnapshots((s) => ingestSnapshot(s));
    return () => {
      alive = false;
      unsub();
    };
  }, []);

  const sortedAgg = useMemo(
    () =>
      (snapshot?.aggregates ?? [])
        .sort((a, b) => b.bytes_in + b.bytes_out - (a.bytes_in + a.bytes_out))
        .slice(0, 8),
    [snapshot],
  );
  const topAgg = sortedAgg[0];

  return (
    <div className="flex h-full flex-col p-4">
      <header className="mb-3 flex items-end justify-between">
        <div>
          <h1 className="text-lg font-semibold text-slate-100">
            Network Radar
          </h1>
          <p className="text-xs text-slate-500">
            Live per-process traffic — read locally, never transmitted.
          </p>
        </div>
        {snapshot && (
          <div className="flex gap-4 font-mono text-xs">
            <div className="text-right">
              <div className="text-[10px] uppercase tracking-wider text-slate-500">
                Total up
              </div>
              <div className="text-neon-emerald">
                {(snapshot.total_out / 1024).toFixed(1)} KB/s
              </div>
            </div>
            <div className="text-right">
              <div className="text-[10px] uppercase tracking-wider text-slate-500">
                Total down
              </div>
              <div className="text-neon-cyan">
                {(snapshot.total_in / 1024).toFixed(1)} KB/s
              </div>
            </div>
            <div className="text-right">
              <div className="text-[10px] uppercase tracking-wider text-slate-500">
                Connections
              </div>
              <div className="text-slate-200">{snapshot.connections.length}</div>
            </div>
          </div>
        )}
      </header>

      <div className="mb-3 rounded-xl border border-base-800 bg-base-900 p-3">
        <BandwidthGraph points={history} height={150} />
        <div className="mt-1 flex items-center justify-between text-[10px] uppercase tracking-wider text-slate-600">
          <span>live bandwidth (bytes/s)</span>
          {topAgg && (
            <span>
              top process:{" "}
              <span className="text-slate-300">{topAgg.process}</span>
            </span>
          )}
        </div>
      </div>

      <div className="flex min-h-0 flex-1 gap-3">
        <div className="min-w-0 flex-1 rounded-xl border border-base-800 bg-base-900 p-3">
          <ConnectionTable
            rows={snapshot?.connections ?? []}
            onSelect={setSelected}
            onBlock={(r) =>
              setBlock({
                row: r,
                target: r.process_path ? "process" : "host",
              })
            }
          />
        </div>
        {selected && (
          <ProcessDrawer
            row={selected}
            allRows={snapshot?.connections ?? []}
            onClose={() => setSelected(null)}
            onTriggerBlock={(t) => setBlock({ row: selected, target: t })}
          />
        )}
      </div>

      {notice && (
        <div className="fixed bottom-9 left-1/2 z-50 -translate-x-1/2 rounded-lg border border-base-600 bg-base-900 px-4 py-2 text-xs text-slate-200 shadow-xl">
          {notice}
          <button
            className="ml-3 text-slate-500 hover:text-slate-300"
            onClick={() => setNotice(null)}
          >
            ✕
          </button>
        </div>
      )}

      {block && (
        <BlockDialog
          row={block.row}
          target={block.target}
          onClose={() => setBlock(null)}
          onDone={(ok, detail) => {
            setBlock(null);
            setNotice(ok ? `Rule applied: ${detail}` : `Failed: ${detail}`);
          }}
        />
      )}
    </div>
  );
}
