import { useEffect, useMemo, useState } from "react";
import {
  applyTelemetryChanges,
  getTelemetryCatalog,
  getUndoCenter,
  undoChange,
} from "../../api/commands";
import type { CatalogItem, JournalEntry } from "../../api/types";
import { CATEGORY_ORDER, CATEGORY_LABELS } from "../../lib/tiers";
import { ConfirmModal, type ConfirmSpec } from "../shell/ConfirmModal";
import { ElevationBanner } from "../shell/ElevationBanner";
import { PreviewDiff } from "../shell/PreviewDiff";
import { TierChip } from "../shell/TierChip";

export function TelemetryScreen() {
  const [items, setItems] = useState<CatalogItem[]>([]);
  const [journal, setJournal] = useState<JournalEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<string>("all");
  const [confirm, setConfirm] = useState<ConfirmSpec | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = async () => {
    const [catalog, entries] = await Promise.all([
      getTelemetryCatalog(),
      getUndoCenter(),
    ]);
    setItems(catalog);
    setJournal(entries);
    setLoading(false);
  };

  useEffect(() => {
    void refresh().catch(() => setLoading(false));
  }, []);

  const grouped = useMemo(() => {
    const map = new Map<string, CatalogItem[]>();
    for (const item of items) {
      if (filter !== "all" && item.tier !== filter) continue;
      const arr = map.get(item.category) ?? [];
      arr.push(item);
      map.set(item.category, arr);
    }
    const order = CATEGORY_ORDER;
    return order
      .map((c) => ({ category: c, items: map.get(c) ?? [] }))
      .filter((g) => g.items.length > 0);
  }, [items, filter]);

  const toggle = (item: CatalogItem) => {
    const diff = buildDiff(item);
    setConfirm({
      title: item.applied ? "Revert change" : "Apply privacy change",
      danger: !item.applied,
      confirmLabel: item.applied ? "Revert" : "Apply",
      body: (
        <div className="space-y-3">
          <p className="text-slate-300">{item.description}</p>
          <PreviewDiff diffs={diff.diffs} note={diff.note} />
          <ElevationBanner
            reason="Registry and scheduled-task changes require administrator rights."
          />
        </div>
      ),
      onConfirm: async () => {
        setBusy(true);
        try {
          if (item.applied) {
            const last = journal
              .filter((j) => j.item_id === item.id && !j.restored)
              .sort((a, b) => (a.ts < b.ts ? 1 : -1))[0];
            const res = last
              ? await undoChange(last.id)
              : { ok: true, detail: "No recorded change to revert." };
            setNotice(res.ok ? `Reverted: ${res.detail}` : `Failed: ${res.detail}`);
          } else {
            const res = await applyTelemetryChanges([item.id]);
            setNotice(res.ok ? "Applied. A restore point was attempted first." : `Failed: ${res.detail}`);
          }
          await refresh();
        } catch (e) {
          setNotice(`Error: ${String(e)}`);
        } finally {
          setBusy(false);
        }
      },
    });
  };

  const filters = [
    { id: "all", label: "All" },
    { id: "recommended", label: "Recommended" },
    { id: "limited", label: "Limited" },
    { id: "notrecommended", label: "Not Rec." },
    { id: "info", label: "Info only" },
  ];

  return (
    <div className="flex h-full flex-col overflow-hidden p-4">
      <header className="mb-3 flex items-end justify-between">
        <div>
          <h1 className="text-lg font-semibold text-slate-100">
            Telemetry & Bloatware Cleaner
          </h1>
          <p className="text-xs text-slate-500">
            Windows collects data through scattered registry keys and scheduled
            tasks. Every change is previewed, restore-pointed, and reversible.
          </p>
        </div>
        <div className="flex gap-1">
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id)}
              className={`rounded-md px-2.5 py-1 text-[11px] font-medium ${
                filter === f.id
                  ? "bg-neon-emerald/10 text-neon-emerald"
                  : "text-slate-500 hover:text-slate-300"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </header>

      <div className="min-h-0 flex-1 overflow-auto pb-3">
        {loading ? (
          <div className="py-16 text-center text-sm text-slate-500">
            Reading local configuration…
          </div>
        ) : (
          <div className="space-y-5">
            {grouped.map((g) => (
              <section key={g.category}>
                <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-neon-cyan">
                  {CATEGORY_LABELS[g.category]}
                </h2>
                <div className="overflow-hidden rounded-xl border border-base-800 bg-base-900">
                  {g.items.map((item, i) => (
                    <ToggleRow
                      key={item.id}
                      item={item}
                      last={i === g.items.length - 1}
                      onToggle={() => toggle(item)}
                    />
                  ))}
                </div>
              </section>
            ))}
            {grouped.length === 0 && (
              <div className="py-16 text-center text-sm text-slate-500">
                No items match this filter.
              </div>
            )}
          </div>
        )}
      </div>

      <UndoCenter
        entries={journal}
        onUndo={async (id) => {
          setBusy(true);
          try {
            const res = await undoChange(id);
            setNotice(res.ok ? `Reverted: ${res.detail}` : `Failed: ${res.detail}`);
            const entries = await getUndoCenter();
            setJournal(entries);
          } catch (e) {
            setNotice(`Error: ${String(e)}`);
          } finally {
            setBusy(false);
          }
        }}
        busy={busy}
      />

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

      {confirm && <ConfirmModal spec={confirm} onClose={() => setConfirm(null)} />}
    </div>
  );
}

function buildDiff(item: CatalogItem): {
  diffs: { target: string; action: string; from: string; to: string }[];
  note: string;
} {
  const to = item.applied ? "restore previous value" : item.desired_display;
  const action =
    item.kind === "delete"
      ? item.applied
        ? "Restore value"
        : "Delete value"
      : item.kind === "task"
        ? item.applied
          ? "Re-enable task"
          : "Disable task"
        : item.applied
          ? "Restore previous value"
          : "Set value";
  return {
    diffs: [
      {
        target: item.target_display,
        action,
        from: item.applied ? item.desired_display : item.current_display,
        to,
      },
    ],
    note: "A System Restore Point is created before any change. Every change is logged to the Undo center below and can be reverted.",
  };
}

function ToggleRow({
  item,
  last,
  onToggle,
}: {
  item: CatalogItem;
  last: boolean;
  onToggle: () => void;
}) {
  const disabled = item.guarded || item.kind === "info";
  return (
    <div
      className={`flex items-start gap-3 px-4 py-3 hover:bg-base-850/60 ${last ? "" : "border-b border-base-800"}`}
    >
      <button
        onClick={onToggle}
        disabled={disabled}
        aria-label={item.applied ? "Revert" : "Apply"}
        className={`mt-0.5 flex h-5 w-9 shrink-0 items-center rounded-full p-0.5 transition-colors ${
          item.applied
            ? "bg-neon-emerald"
            : "bg-base-700"
        } ${disabled ? "cursor-not-allowed opacity-30" : "cursor-pointer"}`}
      >
        <span
          className={`h-4 w-4 rounded-full bg-white transition-transform ${
            item.applied ? "translate-x-4" : ""
          }`}
        />
      </button>
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-medium text-slate-100">
            {item.title}
          </span>
          <TierChip tier={item.tier} />
          {item.guarded && (
            <span className="rounded-full border border-slate-600 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-slate-400">
              guarded
            </span>
          )}
        </div>
        <p className="mt-0.5 text-xs text-slate-400">{item.description}</p>
        <div className="mt-1.5 flex items-center gap-2 font-mono text-[10px] text-slate-500">
          <span className="break-all">{item.target_display}</span>
          <span className="text-slate-600">→</span>
          <span className="break-all text-slate-400">{item.desired_display}</span>
        </div>
        <div className="mt-0.5 text-[10px] text-slate-600">
          current: {item.current_display}
        </div>
      </div>
    </div>
  );
}

function UndoCenter({
  entries,
  onUndo,
  busy,
}: {
  entries: JournalEntry[];
  onUndo: (id: string) => void;
  busy: boolean;
}) {
  const [open, setOpen] = useState(false);
  const recent = [...entries].sort((a, b) => (a.ts < b.ts ? 1 : -1)).slice(0, 8);

  return (
    <div className="shrink-0 rounded-xl border border-base-800 bg-base-900">
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex w-full items-center justify-between px-4 py-2.5 text-left"
      >
        <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
          Undo center ({entries.length})
        </span>
        <span className="text-slate-500">{open ? "▾" : "▸"}</span>
      </button>
      {open && (
        <div className="max-h-52 overflow-auto border-t border-base-800">
          {recent.length === 0 ? (
            <div className="px-4 py-6 text-center text-xs text-slate-600">
              No changes recorded yet. Everything ZeroTrace does lands here.
            </div>
          ) : (
            <table className="w-full text-left text-[11px]">
              <tbody>
                {recent.map((e) => (
                  <tr key={e.id} className="border-t border-base-800">
                    <td className="px-4 py-2 align-top">
                      <div className="font-mono text-slate-400">{e.kind}</div>
                      <div className="text-slate-600">
                        {new Date(e.ts).toLocaleString("en-GB", {
                          hour12: false,
                        })}
                      </div>
                    </td>
                    <td className="px-2 py-2 align-top text-slate-300">
                      <div className="break-all">{e.target}</div>
                      <div className="mt-0.5 font-mono text-[10px] text-slate-500">
                        {e.old_display} → {e.new_display}
                      </div>
                    </td>
                    <td className="px-4 py-2 align-top text-right">
                      {e.restored ? (
                        <span className="text-neon-emerald">reverted</span>
                      ) : (
                        <button
                          disabled={busy}
                          onClick={() => onUndo(e.id)}
                          className="rounded border border-neon-cyan/30 px-2 py-0.5 text-[10px] text-neon-cyan hover:bg-neon-cyan/10 disabled:opacity-40"
                        >
                          Revert
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  );
}
