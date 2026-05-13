import { useHealthStatus } from "@/hooks/use-health";
import type { HealthIssue, HealthSeverity } from "@/lib/types";
import { Card, CardContent, CardHeader } from "@mizan/ui/components/ui/card";
import { Icons } from "@mizan/ui";
import { cn } from "@mizan/ui/lib/utils";
import { Link } from "react-router-dom";

// Dashboard preview of the Wealth Inbox, specified in
// docs/mizan-smart-plan/PLAN.md Prompt 3 (Inbox Preview module).
//
// This iteration sources data from the existing health checks. As Phase 1
// Prompts 8–9 land (smart alerts, full inbox aggregator), this preview will
// consume the unified normalized inbox view model. No fake rows are rendered.

const SEVERITY_DOT: Record<HealthSeverity, string> = {
  CRITICAL: "bg-destructive",
  ERROR: "bg-destructive",
  WARNING: "bg-warning",
  INFO: "bg-muted-foreground",
};

const SEVERITY_RANK: Record<HealthSeverity, number> = {
  CRITICAL: 0,
  ERROR: 1,
  WARNING: 2,
  INFO: 3,
};

const PREVIEW_LIMIT = 3;

function InboxPreviewRow({ issue }: { issue: HealthIssue }) {
  return (
    <li
      data-testid="inbox-preview-item"
      data-severity={issue.severity}
      className="flex items-start gap-2 py-2"
    >
      <span
        aria-hidden="true"
        className={cn("mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full", SEVERITY_DOT[issue.severity])}
      />
      <span className="min-w-0 flex-1 text-sm">
        <span className="block truncate font-medium">{issue.title}</span>
        {issue.message && (
          <span className="text-muted-foreground block truncate text-xs">{issue.message}</span>
        )}
      </span>
    </li>
  );
}

export function InboxPreview() {
  const status = useHealthStatus();
  const issues = (status.data?.issues ?? [])
    .slice()
    .sort((a, b) => SEVERITY_RANK[a.severity] - SEVERITY_RANK[b.severity]);
  const visible = issues.slice(0, PREVIEW_LIMIT);
  const hiddenCount = Math.max(issues.length - visible.length, 0);

  return (
    <Card data-testid="inbox-preview">
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <h2 className="text-base font-semibold">Needs attention</h2>
        <Link
          to="/inbox"
          data-testid="inbox-preview-open"
          className="text-muted-foreground hover:text-foreground text-xs"
        >
          Open inbox
        </Link>
      </CardHeader>
      <CardContent className="pb-3">
        {status.isLoading && !status.data ? (
          <p
            data-testid="inbox-preview-loading"
            className="text-muted-foreground text-sm"
          >
            Checking for new items…
          </p>
        ) : visible.length === 0 ? (
          <div
            data-testid="inbox-preview-empty"
            className="text-muted-foreground flex items-center gap-2 text-sm"
          >
            <Icons.CheckCircle className="size-4" aria-hidden="true" />
            Nothing needs attention.
          </div>
        ) : (
          <>
            <ul className="divide-border/40 divide-y" data-testid="inbox-preview-list">
              {visible.map((issue) => (
                <InboxPreviewRow key={issue.id} issue={issue} />
              ))}
            </ul>
            {hiddenCount > 0 && (
              <p className="text-muted-foreground pt-2 text-xs">
                + {hiddenCount} more in inbox
              </p>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}

export default InboxPreview;
