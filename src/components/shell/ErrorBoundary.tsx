import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
  info: ErrorInfo | null;
}

/** Renders a diagnosable panel on crash instead of a silent blank window. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, info: null };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error("[zerotrace] render error", error, info);
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex h-full w-full flex-col items-center justify-center gap-3 bg-base-950 p-8 text-center">
          <h1 className="text-lg font-semibold text-neon-red">
            ZeroTrace hit a rendering error
          </h1>
          <pre className="max-w-xl overflow-auto whitespace-pre-wrap rounded-lg border border-base-800 bg-base-900 p-3 text-left text-xs text-slate-300">
            {this.state.error.stack ?? this.state.error.message}
          </pre>
          <button
            className="rounded-md border border-neon-emerald/40 px-3 py-1 text-xs text-neon-emerald hover:bg-neon-emerald/10"
            onClick={() => window.location.reload()}
          >
            Reload
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}