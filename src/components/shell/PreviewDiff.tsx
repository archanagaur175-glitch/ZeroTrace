import { FileDiff } from "lucide-react";

export interface DiffLine {
  target: string;
  action: string;
  from: string;
  to: string;
}

export function PreviewDiff({
  diffs,
  note,
}: {
  diffs: DiffLine[];
  note?: string;
}) {
  return (
    <div className="overflow-hidden rounded-lg border border-base-700">
      <div className="flex items-center gap-2 border-b border-base-700 bg-base-800 px-3 py-2 text-xs font-semibold uppercase tracking-wide text-slate-400">
        <FileDiff className="h-3.5 w-3.5 text-neon-cyan" />
        Preview of changes
      </div>
      <table className="w-full text-left text-xs">
        <thead>
          <tr className="text-[10px] uppercase tracking-wide text-slate-500">
            <th className="px-3 py-1.5">Target</th>
            <th className="px-3 py-1.5">Action</th>
            <th className="px-3 py-1.5">Before</th>
            <th className="px-3 py-1.5">After</th>
          </tr>
        </thead>
        <tbody>
          {diffs.map((d, i) => (
            <tr
              key={i}
              className="border-t border-base-800 font-mono text-[11px] text-slate-300"
            >
              <td className="px-3 py-2 break-all">{d.target}</td>
              <td className="px-3 py-2">{d.action}</td>
              <td className="px-3 py-2 text-slate-500">{d.from}</td>
              <td className="px-3 py-2 text-neon-emerald">{d.to}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {note && (
        <p className="border-t border-base-700 bg-base-800 px-3 py-2 text-[11px] text-slate-400">
          {note}
        </p>
      )}
    </div>
  );
}
