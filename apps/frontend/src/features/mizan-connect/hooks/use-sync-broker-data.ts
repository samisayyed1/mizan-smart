import { useMutation } from "@tanstack/react-query";
import { syncBrokerData } from "../services/broker-service";
import { toast } from "@mizan/ui/components/ui/use-toast";

/**
 * Tauri's `invoke` rejects with the inner `String` for `Result<_, String>`
 * commands — i.e. the rejection is a plain JS string, NOT an `Error`
 * instance. This helper extracts a human-readable message regardless of
 * whether the caller threw an `Error`, a string, or some other shape.
 */
function extractErrorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  if (err && typeof err === "object") {
    const maybe = err as { message?: unknown };
    if (typeof maybe.message === "string") return maybe.message;
    try {
      return JSON.stringify(err);
    } catch {
      // fall through
    }
  }
  return "Unknown error";
}

/**
 * Hook to trigger broker data sync.
 * The actual sync runs in the background and results are handled via
 * global event listeners (SSE events trigger toasts and query invalidation).
 */
export function useSyncBrokerData() {
  return useMutation({
    mutationFn: syncBrokerData,
    onSuccess: () => {
      toast.loading("Syncing broker data...", { id: "broker-sync-start" });
    },
    onError: (error) => {
      toast.error(`Failed to start sync: ${extractErrorMessage(error)}`);
    },
  });
}
