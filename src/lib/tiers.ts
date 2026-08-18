import type { Tier } from "../api/types";

export interface TierStyle {
  label: string;
  text: string;
  bg: string;
  border: string;
  dot: string;
}

export const TIER_STYLES: Record<Tier, TierStyle> = {
  recommended: {
    label: "Recommended",
    text: "text-neon-emerald",
    bg: "bg-neon-emerald/10",
    border: "border-neon-emerald/30",
    dot: "bg-neon-emerald",
  },
  limited: {
    label: "Limited",
    text: "text-neon-amber",
    bg: "bg-neon-amber/10",
    border: "border-neon-amber/30",
    dot: "bg-neon-amber",
  },
  notrecommended: {
    label: "Not Recommended",
    text: "text-neon-red",
    bg: "bg-neon-red/10",
    border: "border-neon-red/30",
    dot: "bg-neon-red",
  },
  info: {
    label: "Informational",
    text: "text-slate-400",
    bg: "bg-slate-500/10",
    border: "border-slate-500/30",
    dot: "bg-slate-400",
  },
};

export const CATEGORY_LABELS: Record<string, string> = {
  telemetry: "Telemetry",
  privacy: "Privacy & Activity History",
  cortana: "Search & Cortana",
  location: "Location",
  app_privacy: "App Privacy",
  ai: "AI & Copilot",
  ads: "Advertising & IDs",
  system: "System Reporting & Bloat Tasks",
};

export const CATEGORY_ORDER = [
  "telemetry",
  "privacy",
  "ads",
  "ai",
  "cortana",
  "location",
  "app_privacy",
  "system",
];
