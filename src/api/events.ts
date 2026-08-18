import { listen } from "@tauri-apps/api/event";
import type { NetSnapshot } from "./types";

export type SnapshotListener = (snap: NetSnapshot) => void;

let subscribed = false;
const listeners = new Set<SnapshotListener>();

export function subscribeSnapshots(fn: SnapshotListener): () => void {
  listeners.add(fn);
  void ensure();
  return () => {
    listeners.delete(fn);
  };
}

async function ensure(): Promise<void> {
  if (subscribed) return;
  subscribed = true;
  try {
    await listen<NetSnapshot>("net::snapshot", (e) => {
      for (const l of listeners) l(e.payload);
    });
  } catch (err) {
    console.error("failed to subscribe to net::snapshot", err);
    subscribed = false;
  }
}
