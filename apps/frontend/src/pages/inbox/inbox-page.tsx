import { useHealthStatus } from "@/hooks/use-health";
import type { HealthIssue, HealthSeverity } from "@/lib/types";
import {
  Badge,
  Button,
  Icons,
  Page,
  PageContent,
  PageHeader,
  Skeleton,
} from "@mizan/ui";
import { cn } from "@mizan/ui/lib/utils";
import { Link } from "react-router-dom";

// Wealth Inbox — the central action center described in
// docs/mizan-smart-plan/PLAN.md (Phase 1, Prompt 9).
//
// In this iteration the inbox aggregates only what is real and deterministic
// today: active health-check issues. As Phase 1 (alerts, valuation staleness,
// data quality) and Phase 2 (pending document reviews) land, additional
// item sources are added here. No fake rows are ever rendered.

const SEVERITY_LABEL: Record<HealthSeverity, string> = {
  CRITICAL: "Critical",
  ERROR: "Needs attention",
  WARNING: "Review",
  INFO: "Info",
};

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

function InboxRow({ issue }: { issue: HealthIssue }) {
  return (
    <li
      data-testid="inbox-item"
      data-severity={issue.severity}
      className="flex items-start gap-3 rounded-md border bg-card px-4 py-4"
    >
      <span
        aria-hidden="true"
        className={cn("mt-2 h-2 w-2 shrink-0 rounded-full", SEVERITY_DOT[issue.severity])}
      />
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="text-base font-semibold">{issue.title}</h3>
          <Badge variant="outline" className="text-xs">
            {SEVERITY_LABEL[issue.severity]}
          </Badge>
        </div>
        {issue.message && (
          <p className="text-muted-foreground mt-1 text-sm">{issue.message}</p>
        )}
      </div>
      <Button asChild size="sm" variant="secondary">
        <Link to="/health">Open</Link>
      </Button>
    </li>
  );
}

export default function InboxPage() {
  const status = useHealthStatus();
  const issues = (status.data?.issues ?? [])
    .slice()
    .sort((a, b) => SEVERITY_RANK[a.severity] - SEVERITY_RANK[b.severity]);

  return (
    <Page>
      <PageHeader
        heading="Inbox"
        text="Everything that needs your attention, in one place."
      />
      <PageContent>
        {status.isLoading ? (
          <div className="space-y-3" data-testid="inbox-loading">
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-20 w-full" />
          </div>
        ) : issues.length === 0 ? (
          <div
            data-testid="inbox-empty"
            className="rounded-lg border border-dashed bg-muted/30 px-6 py-12 text-center"
          >
            <Icons.CheckCircle className="text-muted-foreground mx-auto size-10" aria-hidden="true" />
            <p className="mt-4 text-base font-medium">Nothing needs attention</p>
            <p className="text-muted-foreground mt-1 text-sm">
              Inbox will surface alerts, stale valuations, document reviews, and upcoming
              events as they appear.
            </p>
          </div>
        ) : (
          <ul className="space-y-3" data-testid="inbox-list">
            {issues.map((issue) => (
              <InboxRow key={issue.id} issue={issue} />
            ))}
          </ul>
        )}
      </PageContent>
    </Page>
  );
}
