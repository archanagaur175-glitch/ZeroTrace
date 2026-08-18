import {
  Activity,
  Boxes,
  ListChecks,
  ScanEye,
  Settings,
  ShieldCheck,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

export type Screen =
  | "radar"
  | "telemetry"
  | "startup"
  | "trust"
  | "settings";

const ITEMS: { id: Screen; label: string; icon: LucideIcon }[] = [
  { id: "radar", label: "Radar", icon: Activity },
  { id: "telemetry", label: "Telemetry", icon: ListChecks },
  { id: "startup", label: "Startup", icon: Boxes },
  { id: "trust", label: "Trust", icon: ShieldCheck },
  { id: "settings", label: "Settings", icon: Settings },
];

export function IconRail({
  active,
  onNavigate,
}: {
  active: Screen;
  onNavigate: (s: Screen) => void;
}) {
  return (
    <nav className="flex w-16 flex-col items-center gap-2 border-r border-base-700 bg-base-900 py-4">
      <div className="mb-2 flex h-9 w-9 items-center justify-center">
        <ScanEye className="h-6 w-6 text-neon-emerald" strokeWidth={1.8} />
      </div>
      {ITEMS.map((item) => {
        const Icon = item.icon;
        const isActive = active === item.id;
        return (
          <button
            key={item.id}
            title={item.label}
            aria-label={item.label}
            onClick={() => onNavigate(item.id)}
            className={`relative flex h-11 w-11 items-center justify-center rounded-lg transition-colors ${
              isActive
                ? "bg-neon-emerald/10 text-neon-emerald shadow-glow"
                : "text-slate-400 hover:bg-base-800 hover:text-slate-200"
            }`}
          >
            {isActive && (
              <span className="absolute -left-0 top-1/2 h-6 w-0.5 -translate-y-1/2 rounded bg-neon-emerald" />
            )}
            <Icon className="h-5 w-5" strokeWidth={1.7} />
          </button>
        );
      })}
    </nav>
  );
}
