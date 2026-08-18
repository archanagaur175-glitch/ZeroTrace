export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "0 B";
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB", "PB"];
  let v = n;
  let u = -1;
  do {
    v /= 1024;
    u += 1;
  } while (v >= 1024 && u < units.length - 1);
  return `${v.toFixed(v >= 100 ? 0 : v >= 10 ? 1 : 2)} ${units[u]}`;
}

export function formatRate(n: number): string {
  return `${formatBytes(n)}/s`;
}

export function formatPort(port: number | null): string {
  return port === null ? "—" : String(port);
}

export function formatAddr(addr: string, port: number | null): string {
  const hasColon = addr.includes(":");
  const base = hasColon ? `[${addr}]` : addr;
  return port === null ? base : `${base}:${port}`;
}

export function formatTs(ms: number): string {
  const d = new Date(ms);
  return d.toLocaleTimeString("en-GB", { hour12: false });
}

export function formatDate(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleString("en-GB", { hour12: false });
}

export function shortHash(h: string): string {
  if (!h) return "unknown";
  return h.slice(0, 10);
}

export function clamp(v: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, v));
}
