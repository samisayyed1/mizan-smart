import { Button } from "@mizan/ui/components/ui/button";
import { Icons } from "@mizan/ui/components/ui/icons";
import { Tooltip, TooltipContent, TooltipTrigger } from "@mizan/ui/components/ui/tooltip";
import { formatDistanceToNow } from "date-fns";
import { useMizanConnect } from "../providers/mizan-connect-provider";
import { useAggregatedSyncStatus, useSyncBrokerData } from "../hooks";
import type { AggregatedSyncStatus } from "../types";

interface SyncButtonProps {
  /** Optional class name for the button */
  className?: string;
  /** Show label text alongside icon */
  showLabel?: boolean;
  /** Button size */
  size?: "default" | "sm" | "icon";
}

const statusColors: Record<AggregatedSyncStatus, string> = {
  not_connected: "text-muted-foreground",
  idle: "text-green-500",
  running: "text-blue-500",
  needs_review: "text-yellow-500",
  failed: "text-red-500",
};

/**
 * Contextual sync button that shows sync status and triggers sync.
 * Only visible when Connect is enabled and user has an active subscription.
 */
export function SyncButton({ className, showLabel = false, size = "icon" }: SyncButtonProps) {
  const { isEnabled, isConnected } = useMizanConnect();
  const { status, lastSyncTime } = useAggregatedSyncStatus();
  const { mutate: syncBrokerData, isPending: isSyncing } = useSyncBrokerData();

  // TODO(chunk-4): restore plan-tier gating once /api/v1/user/me returns
  // team.plan. For Chunk 3 the broker UI shows whenever the user is
  // signed in to Mizan Connect.
  if (!isEnabled || !isConnected) {
    return null;
  }

  const isRunning = status === "running" || isSyncing;
  const colorClass = statusColors[status];

  const tooltipContent = lastSyncTime
    ? `Last synced ${formatDistanceToNow(new Date(lastSyncTime), { addSuffix: true })}`
    : "Never synced";

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size={size}
          onClick={() => syncBrokerData()}
          disabled={isRunning}
          className={className}
        >
          {isRunning ? (
            <Icons.Spinner className="h-4 w-4 animate-spin" />
          ) : (
            <Icons.RefreshCw className={`h-4 w-4 ${colorClass}`} />
          )}
          {showLabel && <span className="ml-2">{isRunning ? "Syncing..." : "Sync"}</span>}
        </Button>
      </TooltipTrigger>
      <TooltipContent>
        <p>{isRunning ? "Syncing..." : tooltipContent}</p>
      </TooltipContent>
    </Tooltip>
  );
}
