import { useSyncExternalStore } from "react";

export interface Store<T> {
  get(): T;
  set(fn: (s: T) => T): void;
  subscribe(l: () => void): () => void;
}

export function createStore<T>(initial: T): Store<T> {
  let state = initial;
  const listeners = new Set<() => void>();
  return {
    get: () => state,
    set: (fn) => {
      state = fn(state);
      for (const l of listeners) l();
    },
    subscribe: (l) => {
      listeners.add(l);
      return () => {
        listeners.delete(l);
      };
    },
  };
}

export function useStore<T, R>(store: Store<T>, selector: (s: T) => R): R {
  return useSyncExternalStore(
    (l) => store.subscribe(l),
    () => selector(store.get()),
    () => selector(store.get()),
  );
}
