import { ShieldAlert, X } from "lucide-react";

export interface ConfirmSpec {
  title: string;
  body: React.ReactNode;
  confirmLabel: string;
  cancelLabel?: string;
  danger?: boolean;
  onConfirm: () => void;
}

export function ConfirmModal({
  spec,
  onClose,
}: {
  spec: ConfirmSpec;
  onClose: () => void;
}) {
  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg rounded-xl border border-base-600 bg-base-900 p-5 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-3 flex items-center gap-2">
          <ShieldAlert
            className={`h-5 w-5 ${spec.danger ? "text-neon-red" : "text-neon-amber"}`}
          />
          <h2 className="text-base font-semibold text-slate-100">
            {spec.title}
          </h2>
          <button
            className="ml-auto text-slate-500 hover:text-slate-200"
            onClick={onClose}
            aria-label="Close"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
        <div className="mb-5 max-h-72 overflow-y-auto text-sm text-slate-300">
          {spec.body}
        </div>
        <div className="flex justify-end gap-2">
          <button
            onClick={onClose}
            className="rounded-md border border-base-600 px-3 py-1.5 text-sm text-slate-300 hover:bg-base-800"
          >
            {spec.cancelLabel ?? "Cancel"}
          </button>
          <button
            onClick={() => {
              spec.onConfirm();
              onClose();
            }}
            className={`rounded-md px-3 py-1.5 text-sm font-semibold ${
              spec.danger
                ? "bg-neon-red/90 text-black hover:bg-neon-red"
                : "bg-neon-emerald text-black hover:brightness-110"
            }`}
          >
            {spec.confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
