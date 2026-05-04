import { openUrlInBrowser } from "@/adapters";
import { QueryKeys } from "@/lib/query-keys";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "@mizan/ui/components/ui/use-toast";
import { useCallback, useEffect, useRef, useState } from "react";
import { createBrokerLoginPortal } from "../services/broker-service";

const POLL_INTERVAL_MS = 5_000;
const POLL_DURATION_MS = 60_000;

/**
 * Mutation that mints a SnapTrade Connection Portal URL and opens it in
 * the user's default browser.
 *
 * After a successful mint, callers can opt in to a 60-second polling
 * window via {@link usePollConnectionsAfterPortal}: while polling, the
 * `BROKER_CONNECTIONS` query is invalidated every 5 seconds so the new
 * authorization shows up as soon as SnapTrade's callback finishes.
 *
 * Failure modes (raised as toasts):
 * - 429 → "too many login-portal requests; try again later"
 * - any 5xx upstream → "broker temporarily unavailable"
 * - network error → "couldn't reach Mizan Connect"
 */
export function useCreateBrokerLoginPortal() {
  return useMutation({
    mutationFn: async (broker?: string) => createBrokerLoginPortal(broker),
    onSuccess: ({ url }) => {
      void openUrlInBrowser(url);
      toast.success("Opened SnapTrade portal in your browser", {
        id: "broker-portal-opened",
      });
    },
    onError: (error) => {
      const msg = error instanceof Error ? error.message : "Unknown error";
      toast.error(`Couldn't start broker connection: ${msg}`, {
        id: "broker-portal-error",
      });
    },
  });
}

/**
 * Drives a 60-second polling window that refetches broker connections
 * every 5 seconds. Returns `[isPolling, startPolling]`. Idempotent: a
 * second `startPolling()` call within the window resets the timer.
 *
 * Used right after `useCreateBrokerLoginPortal` succeeds so the
 * connections list updates as soon as the user completes the SnapTrade
 * flow in their browser.
 */
export function usePollConnectionsAfterPortal(): [boolean, () => void] {
  const queryClient = useQueryClient();
  const [isPolling, setIsPolling] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const stopRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const stop = useCallback(() => {
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    if (stopRef.current !== null) {
      clearTimeout(stopRef.current);
      stopRef.current = null;
    }
    setIsPolling(false);
  }, []);

  const start = useCallback(() => {
    stop();
    setIsPolling(true);
    intervalRef.current = setInterval(() => {
      queryClient.invalidateQueries({ queryKey: [QueryKeys.BROKER_CONNECTIONS] });
      queryClient.invalidateQueries({ queryKey: [QueryKeys.BROKER_ACCOUNTS] });
    }, POLL_INTERVAL_MS);
    stopRef.current = setTimeout(stop, POLL_DURATION_MS);
  }, [queryClient, stop]);

  useEffect(() => stop, [stop]);

  return [isPolling, start];
}
