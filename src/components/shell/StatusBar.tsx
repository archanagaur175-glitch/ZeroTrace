import { Radio } from "lucide-react";

export function StatusBar() {
  return (
    <footer className="flex h-7 items-center gap-3 border-t border-base-700 bg-base-900 px-3 text-[10px] uppercase tracking-wider text-slate-500">
      <span className="flex items-center gap-1.5">
        <Radio className="h-3 w-3 animate-pulse-dot text-neon-emerald" />
        ZeroTrace
      </span>
      <span className="text-slate-600">·</span>
      <span>100% local · zero outbound traffic</span>
      <span className="ml-auto flex items-center gap-1.5">
        <span className="h-1.5 w-1.5 rounded-full bg-neon-emerald" />
        offline-first
      </span>
    </footer>
  );
}
