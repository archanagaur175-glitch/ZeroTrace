import { useEffect, useState } from "react";
import {
  disableStartupEntry,
  enableStartupEntry,
  getServiceXRay,
  getStartupEntries,
} from "../../api/commands";
import type { ServiceRow, StartupEntry } from "../../api/types";
import { ConfirmModal, type ConfirmSpec } from "../shell/ConfirmModal";
import { ElevationBanner } from "../shell/ElevationBanner";

export function StartupScreen() {
  const [entries, setEntries] = useState<StartupEntry[]>([]);
  const [services, setServices] = useState<ServiceRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [confirm, setConfirm] = useState<ConfirmSpec | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = async () => {
    const [e, s] = await Promise.all([
      getStartupEntries(),
      getServiceXRay(),
    ]);
    setEntries(e);
    setServices(s);
    setLoading(false);
  };

  useEffect(() => {
    void refresh().catch(() => setLoading(false));
  }, []);

  const toggle = (entry: StartupEntry) => {
    const disable = entry.enabled;
    setConfirm({
      title: disable ? "Disable startup entry" : "Enable startup entry",
      danger: disable,
      confirmLabel: disable ? "Disable" : "Enable",
      body: (
        <div className="space-y-3">
          <div className="rounded-lg border border-base-700 bg-base-850 p-3 font-mono text-xs">
            <div className="text-slate-400">{entry.name}</div>
            <div className="mt-1 break-all text-slate-500">{entry.command}</div>
            <div className="mt-1 text-slate-600">{entry.source}</div>
          </div>
          <ElevationBanner reason="Modifying startup entries requires administrator rights." />
        </div>
      ),
      onConfirm: async () => {
        try {
          const res = disable
            ? await disableStartupEntry(entry.id)
            : await enableStartupEntry(entry.id);
          setNotice(
            res.ok ? `Done: ${res.detail}` : `Failed: ${res.detail}`,
          );
          await refresh();
        } catch (e) {
          setNotice(`Error: ${String(e)}`);
        }
      },
    });
  };

  const riskStyle = (r: StartupEntry["risk"]) =>
    r === "high"
      ? "bg-neon-red/10 text-neon-red border-neon-red/30"
      : r === "medium"
        ? "bg-neon-amber/10 text-neon-amber border-neon-amber/30"
        : "bg-neon-emerald/10 text-neon-emerald border-neon-emerald/30";

  return (
    <div className="flex h-full flex-col overflow-hidden p-4">
      <header className="mb-3">
        <h1 className="text-lg font-semibold text-slate-100">
          Startup & Background X-Ray
        </h1>
        <p className="text-xs text-slate-500">
          What starts automatically, from where, and whether it can be trusted.
        </p>
      </header>

      <div className="min-h-0 flex-1 overflow-auto pb-3">
        {loading ? (
          <div className="py-16 text-center text-sm text-slate-500">
            Scanning startup sources…
          </div>
        ) : (
          <div className="space-y-5">
            <section>
              <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-neon-cyan">
                Startup entries ({entries.length})
              </h2>
              <div className="overflow-hidden rounded-xl border border-base-800 bg-base-900">
                {entries.length === 0 ? (
                  <div className="px-4 py-6 text-center text-xs text-slate-600">
                    No startup entries found.
                  </div>
                ) : (
                  <table className="w-full text-left text-xs">
                    <thead className="bg-base-950 text-[10px] uppercase tracking-wider text-slate-500">
                      <tr>
                        <th className="px-3 py-2">Entry</th>
                        <th className="px-3 py-2">Source</th>
                        <th className="px-3 py-2">Signer</th>
                        <th className="px-3 py-2">Risk</th>
                        <th className="px-3 py-2" />
                      </tr>
                    </thead>
                    <tbody>
                      {entries.map((e) => (
                        <tr
                          key={e.id}
                          className="border-t border-base-800 align-top"
                        >
                          <td className="px-3 py-2">
                            <div className="font-medium text-slate-200">
                              {e.name}
                            </div>
                            <div className="mt-0.5 break-all font-mono text-[10px] text-slate-500">
                              {e.command}
                            </div>
                            {e.path && (
                              <div className="mt-0.5 break-all font-mono text-[10px] text-slate-600">
                                {e.path}
                              </div>
                            )}
                          </td>
                          <td className="px-3 py-2 text-slate-400">
                            {e.source}
                            <div className="text-[10px] text-slate-600">
                              {e.source_detail}
                            </div>
                          </td>
                          <td className="px-3 py-2 text-slate-400">
                            {e.signed === null
                              ? "—"
                              : e.signed
                                ? e.publisher ?? "Signed"
                                : "Unsigned"}
                          </td>
                          <td className="px-3 py-2">
                            <span
                              className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase ${riskStyle(e.risk)}`}
                              title={e.risk_reasons.join("; ")}
                            >
                              {e.risk}
                            </span>
                            {e.risk_reasons.length > 0 && (
                              <ul className="mt-1 list-inside list-disc text-[10px] text-slate-500">
                                {e.risk_reasons.slice(0, 2).map((r, i) => (
                                  <li key={i}>{r}</li>
                                ))}
                              </ul>
                            )}
                          </td>
                          <td className="px-3 py-2 text-right">
                            <button
                              onClick={() => toggle(e)}
                              disabled={e.source.includes("RunOnce")}
                              className={`rounded-md border px-2.5 py-1 text-[10px] font-semibold ${
                                e.enabled
                                  ? "border-neon-red/40 text-neon-red hover:bg-neon-red/10"
                                  : "border-neon-emerald/40 text-neon-emerald hover:bg-neon-emerald/10"
                              } disabled:cursor-not-allowed disabled:opacity-30`}
                            >
                              {e.enabled ? "Disable" : "Enable"}
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            </section>

            <section>
              <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-neon-cyan">
                Background services ({services.length})
              </h2>
              <div className="overflow-hidden rounded-xl border border-base-800 bg-base-900">
                <table className="w-full text-left text-xs">
                  <thead className="bg-base-950 text-[10px] uppercase tracking-wider text-slate-500">
                    <tr>
                      <th className="px-3 py-2">Service</th>
                      <th className="px-3 py-2">State</th>
                      <th className="px-3 py-2">Signer</th>
                      <th className="px-3 py-2">Risk</th>
                    </tr>
                  </thead>
                  <tbody>
                    {services.map((s) => (
                      <tr
                        key={s.name}
                        className="border-t border-base-800 align-top"
                      >
                        <td className="px-3 py-2">
                          <div className="text-slate-200">{s.display_name}</div>
                          <div className="font-mono text-[10px] text-slate-500">
                            {s.name}
                          </div>
                          <div className="mt-0.5 break-all font-mono text-[10px] text-slate-600">
                            {s.path}
                          </div>
                        </td>
                        <td className="px-3 py-2 text-slate-400">{s.state}</td>
                        <td className="px-3 py-2 text-slate-400">
                          {s.signed === null
                            ? "—"
                            : s.signed
                              ? s.publisher ?? "Signed"
                              : "Unsigned"}
                        </td>
                        <td className="px-3 py-2">
                          <span
                            className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase ${riskStyle(s.risk)}`}
                            title={s.risk_reasons.join("; ")}
                          >
                            {s.risk}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          </div>
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

      {confirm && (
        <ConfirmModal spec={confirm} onClose={() => setConfirm(null)} />
      )}
    </div>
  );
}
