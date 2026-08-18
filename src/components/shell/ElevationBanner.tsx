import { ShieldCheck } from "lucide-react";

export function ElevationBanner({
  reason,
}: {
  reason: string;
}) {
  return (
    <div className="flex items-start gap-2 rounded-lg border border-neon-cyan/30 bg-neon-cyan/5 px-3 py-2 text-xs text-slate-300">
      <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-neon-cyan" />
      <p>
        <span className="font-semibold text-neon-cyan">Admin required.</span>{" "}
        {reason} A Windows UAC prompt will appear; this action is performed by
        the isolated ZeroTrace helper and is never sent anywhere.
      </p>
    </div>
  );
}
