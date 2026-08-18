import type { Tier } from "../../api/types";
import { TIER_STYLES } from "../../lib/tiers";

export function TierChip({ tier }: { tier: Tier }) {
  const s = TIER_STYLES[tier];
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${s.bg} ${s.border} ${s.text}`}
    >
      <span className={`h-1.5 w-1.5 rounded-full ${s.dot}`} />
      {s.label}
    </span>
  );
}
