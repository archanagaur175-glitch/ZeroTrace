import { useEffect, useState } from "react";
import { Download, Save } from "lucide-react";
import {
  exportJournal,
  getSettings,
  isElevated,
  setSettings,
} from "../../api/commands";
import type { AppSettings } from "../../api/types";

export function SettingsScreen() {
  const [settings, setSettingsState] = useState<AppSettings | null>(null);
  const [elevated, setElevated] = useState<boolean | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    void getSettings().then(setSettingsState).catch(() => undefined);
    void isElevated().then(setElevated).catch(() => undefined);
  }, []);

  const save = async () => {
    if (!settings) return;
    try {
      await setSettings(settings);
      setNotice("Settings saved.");
    } catch (e) {
      setNotice(`Error: ${String(e)}`);
    }
  };

  return (
    <div className="h-full overflow-auto p-4">
      <header className="mb-4">
        <h1 className="text-lg font-semibold text-slate-100">Settings</h1>
        <p className="text-xs text-slate-500">
          All settings are stored locally. Nothing is synced anywhere.
        </p>
      </header>

      <div className="grid max-w-2xl gap-4">
        <div className="rounded-xl border border-base-800 bg-base-900 p-4">
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wider text-slate-400">
            Radar
          </h2>
          {settings && (
            <div className="space-y-4">
              <label className="block">
                <span className="text-xs text-slate-300">
                  Poll interval (ms)
                </span>
                <input
                  type="number"
                  min={500}
                  max={10000}
                  step={250}
                  value={settings.poll_interval_ms}
                  onChange={(e) =>
                    setSettingsState({
                      ...settings,
                      poll_interval_ms: Number(e.target.value),
                    })
                  }
                  className="mt-1 w-40 rounded-md border border-base-700 bg-base-850 px-3 py-1.5 font-mono text-xs text-slate-200 focus:border-neon-cyan/50 focus:outline-none"
                />
                <span className="ml-2 text-[11px] text-slate-500">
                  lower = more precise, slightly more CPU
                </span>
              </label>
              <label className="flex items-center gap-2 text-xs text-slate-300">
                <input
                  type="checkbox"
                  checked={settings.reverse_dns_enabled}
                  onChange={(e) =>
                    setSettingsState({
                      ...settings,
                      reverse_dns_enabled: e.target.checked,
                    })
                  }
                  className="h-4 w-4 accent-[#10ffb0]"
                />
                Resolve hostnames (local resolver only, cached)
              </label>
            </div>
          )}
          <div className="mt-4">
            <button
              onClick={() => void save()}
              className="inline-flex items-center gap-1.5 rounded-md bg-neon-emerald px-3 py-1.5 text-xs font-semibold text-black hover:brightness-110"
            >
              <Save className="h-3.5 w-3.5" />
              Save settings
            </button>
          </div>
        </div>

        <div className="rounded-xl border border-base-800 bg-base-900 p-4">
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wider text-slate-400">
            Privacy
          </h2>
          <p className="text-xs text-slate-300">
            {elevated === null
              ? "Checking elevation state…"
              : elevated
                ? "This instance is running elevated (not recommended). ZeroTrace runs unprivileged by default and elevates only the scoped helper for each admin action."
                : "Running unprivileged. Admin actions prompt UAC per operation via the isolated ZeroTrace helper."}
          </p>
        </div>

        <div className="rounded-xl border border-base-800 bg-base-900 p-4">
          <h2 className="mb-3 text-xs font-semibold uppercase tracking-wider text-slate-400">
            Data
          </h2>
          <p className="mb-3 text-xs text-slate-300">
            The Undo center journal is stored as local JSONL at{" "}
            <code className="text-slate-400">
              %LOCALAPPDATA%\ZeroTrace\journal.jsonl
            </code>
            .
          </p>
          <button
            onClick={() =>
              void exportJournal()
                .then((p) => setNotice(`Journal exported to ${p}`))
                .catch((e) => setNotice(`Error: ${String(e)}`))
            }
            className="inline-flex items-center gap-1.5 rounded-md border border-base-600 px-3 py-1.5 text-xs text-slate-300 hover:bg-base-800"
          >
            <Download className="h-3.5 w-3.5" />
            Export journal
          </button>
        </div>
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
    </div>
  );
}
