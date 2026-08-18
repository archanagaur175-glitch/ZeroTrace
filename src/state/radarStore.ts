import { createStore, useStore } from "./store";
import { RingBuffer, type ChartPoint } from "../lib/chart";
import type { NetSnapshot } from "../api/types";

interface RadarState {
  snapshot: NetSnapshot | null;
  history: RingBuffer;
}

const store = createStore<RadarState>({
  snapshot: null,
  history: new RingBuffer(150),
});

export function ingestSnapshot(snap: NetSnapshot): void {
  store.set((s) => {
    const history = s.history;
    history.push({
      t: snap.ts,
      up: snap.total_out,
      down: snap.total_in,
    } satisfies ChartPoint);
    return { snapshot: snap, history };
  });
}

export function useSnapshot(): NetSnapshot | null {
  return useStore(store, (s) => s.snapshot);
}

export function useHistory(): ChartPoint[] {
  return useStore(store, (s) => s.history.snapshot());
}
