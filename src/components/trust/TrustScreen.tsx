import { useEffect, useState } from "react";
import {
  ExternalLink,
  FileCheck2,
  GitCommitHorizontal,
  Lock,
  ShieldCheck,
  WifiOff,
} from "lucide-react";
import { getBuildInfo, getDependencyAudit, openRepo } from "../../api/commands";
import type { BuildInfo, DepAudit } from "../../api/types";
import { shortHash } from "../../lib/format";

export function TrustScreen() {
  const [build, setBuild] = useState<BuildInfo | null>(null);
  const [audit, setAudit] = useState<DepAudit | null>(null);

  useEffect(() => {
    void getBuildInfo().then(setBuild).catch(() => undefined);
    void getDependencyAudit().then(setAudit).catch(() => undefined);
  }, []);

  return (
    <div className="h-full overflow-auto p-4">
      <header className="mb-4">
        <h1 className="text-lg font-semibold text-slate-100">
          Trust & Verification
        </h1>
        <p className="text-xs text-slate-500">
          Radical transparency: verify that ZeroTrace does exactly what it
          claims.
        </p>
      </header>

      <div className="grid max-w-3xl gap-4">
        <ZeroTelemetryBanner />

        <div className="rounded-xl border border-base-800 bg-base-900 p-4">
          <div className="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-slate-400">
            <GitCommitHorizontal className="h-4 w-4 text-neon-cyan" />
            Build provenance
          </div>
          <dl className="grid grid-cols-2 gap-3 text-xs">
            <Field label="Version" value={build?.version ?? "—"} mono />
            <Field label="Profile" value={build?.profile ?? "—"} mono />
            <Field label="Commit" value={shortHash(build?.commit ?? "—")} mono />
            <Field label="Built at" value={build?.built_at ?? "—"} />
            <Field label="Source" value={build?.repo ?? "—"} mono />
          </dl>
          <p className="mt-3 text-[11px] text-slate-500">
            This binary was built from commit{" "}
            <code className="text-neon-cyan">{shortHash(build?.commit ?? "—")}</code>.
            Rebuild it yourself from source and compare the artifact hash to
            verify provenance.
          </p>
          <button
            onClick={() => void openRepo().catch(() => undefined)}
            className="mt-3 inline-flex items-center gap-1.5 rounded-md border border-base-600 px-3 py-1.5 text-xs text-neon-cyan hover:bg-neon-cyan/10"
          >
            <ExternalLink className="h-3.5 w-3.5" />
            View source on GitHub
            <span className="text-[10px] text-slate-500">
              (opens your browser — your choice)
            </span>
          </button>
        </div>

        <DependencyAudit audit={audit} />

        <div className="rounded-xl border border-base-800 bg-base-900 p-4">
          <div className="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-slate-400">
            <FileCheck2 className="h-4 w-4 text-neon-emerald" />
            License
          </div>
          <p className="text-xs leading-relaxed text-slate-300">
            ZeroTrace is licensed under the{" "}
            <span className="font-semibold text-slate-100">
              GNU General Public License v3 (GPL-3.0)
            </span>
            , the same license used by Portmaster — the closest architectural
            predecessor to this tool. Copyleft keeps this category of
            system-privilege privacy software open for everyone.
          </p>
        </div>

        <div className="rounded-xl border border-base-800 bg-base-900 p-4">
          <div className="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-slate-400">
            <Lock className="h-4 w-4 text-neon-emerald" />
            Local-only operation
          </div>
          <ul className="list-inside list-disc space-y-1 text-xs text-slate-300">
            <li>All reads (sockets, registry, tasks) are local to this PC.</li>
            <li>No account, no cloud, no data leaves the machine.</li>
            <li>Reverse-DNS uses only your configured local resolver.</li>
            <li>
              Elevation is scoped: a helper binary performs each admin action
              on demand, then exits.
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
}

function Field({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div>
      <dt className="text-[10px] uppercase tracking-wider text-slate-500">
        {label}
      </dt>
      <dd
        className={`mt-0.5 break-all ${mono ? "font-mono text-slate-200" : "text-slate-200"}`}
      >
        {value}
      </dd>
    </div>
  );
}

function ZeroTelemetryBanner() {
  return (
    <div className="flex items-start gap-3 rounded-xl border border-neon-emerald/30 bg-neon-emerald/5 p-4">
      <ShieldCheck className="mt-0.5 h-6 w-6 shrink-0 text-neon-emerald" />
      <div>
        <div className="text-sm font-semibold text-slate-100">
          ZeroTrace makes zero outbound network calls. Ever.
        </div>
        <p className="mt-1 text-xs leading-relaxed text-slate-300">
          No analytics SDKs, no crash reporters, no update-checker, no font or
          CDN fetches. This guarantee is <em>enforced</em>: the WebView runs
          under a Content Security Policy with <code>connect-src 'none'</code>,
          and the installer is CI-verified to contain no network code. The
          GitHub Actions pipeline <em>does</em> fetch build dependencies at
          build time — that is build infrastructure, not runtime telemetry, and
          never ships in this app.
        </p>
      </div>
    </div>
  );
}

function DependencyAudit({ audit }: { audit: DepAudit | null }) {
  return (
    <div className="rounded-xl border border-base-800 bg-base-900 p-4">
      <div className="mb-3 flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-slate-400">
        <WifiOff className="h-4 w-4 text-neon-cyan" />
        Dependency audit
      </div>
      {audit ? (
        <>
          <p className="text-xs text-slate-300">
            {audit.total_deps} dependencies scanned across{" "}
            {audit.files.map((f) => f.file).join(", ")} against a deny-list of
            analytics/tracking packages.
          </p>
          <div className="mt-3">
            {audit.zero_telemetry ? (
              <div className="flex items-center gap-2 rounded-lg border border-neon-emerald/30 bg-neon-emerald/10 px-3 py-2 text-xs font-semibold text-neon-emerald">
                <ShieldCheck className="h-4 w-4" />
                PASS — no telemetry, analytics, or tracking packages found
              </div>
            ) : (
              <div className="rounded-lg border border-neon-red/30 bg-neon-red/10 px-3 py-2 text-xs font-semibold text-neon-red">
                FAIL — review the hits below
              </div>
            )}
          </div>
          {audit.hits.length > 0 && (
            <ul className="mt-3 space-y-1 text-xs text-neon-red">
              {audit.hits.map((h, i) => (
                <li key={i}>
                  <code>{h.name}</code> in {h.file}: {h.reason}
                </li>
              ))}
            </ul>
          )}
          <p className="mt-3 text-[11px] text-slate-500">
            Audit generated at {audit.generated_at} by the release pipeline and
            embedded into this binary.
          </p>
        </>
      ) : (
        <div className="text-xs text-slate-500">Loading audit report…</div>
      )}
    </div>
  );
}
