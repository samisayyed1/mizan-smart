import { Button, Icons, Page, PageContent, PageHeader } from "@mizan/ui";
import { Link } from "react-router-dom";

// Reports page — landing surface for the deterministic Report Builder
// (Phase 4, prompts 30–32 of docs/mizan-smart-plan/PLAN.md). The full
// Report Builder, Monthly Wealth Letter, Tax Pack, and Estate Binder are
// implemented later.
//
// For now this page links to the real, existing report-like screens so
// users have a direct path to performance, income, breakdowns, and data
// health. No fake report rows are rendered.

type ReportIconName = "TrendingUp" | "HandCoins" | "PieChart" | "ShieldCheck";

interface ReportLink {
  title: string;
  description: string;
  href: string;
  icon: ReportIconName;
}

const EXISTING_REPORTS: ReportLink[] = [
  {
    title: "Performance",
    description: "Time-weighted return, drawdowns, and account-level performance.",
    href: "/performance",
    icon: "TrendingUp",
  },
  {
    title: "Income",
    description: "Dividends, interest, and other distributions by period.",
    href: "/income",
    icon: "HandCoins",
  },
  {
    title: "Holdings breakdown",
    description: "Allocation by sector, country, currency, and account.",
    href: "/insights",
    icon: "PieChart",
  },
  {
    title: "Data health",
    description: "Stale quotes, missing FX, classification gaps.",
    href: "/health",
    icon: "ShieldCheck",
  },
];

export default function ReportsPage() {
  return (
    <Page>
      <PageHeader
        heading="Reports"
        text="Performance, income, breakdowns, and data health."
      />
      <PageContent>
        <div
          data-testid="reports-builder-notice"
          className="rounded-lg border border-dashed bg-muted/30 px-6 py-6"
        >
          <p className="text-base font-medium">Deterministic Report Builder coming in Phase 4</p>
          <p className="text-muted-foreground mt-1 text-sm">
            The full Report Builder, Monthly Wealth Letter, Tax Pack, and Estate Binder
            are implemented in Phase 4 of this branch. The reports below are real and
            available today.
          </p>
        </div>

        <div className="mt-6 grid gap-3 md:grid-cols-2" data-testid="reports-existing-grid">
          {EXISTING_REPORTS.map((report) => {
            const Icon = Icons[report.icon];
            return (
              <Button
                key={report.href}
                asChild
                variant="outline"
                className="h-auto justify-start gap-3 px-4 py-4 text-left"
              >
                <Link to={report.href}>
                  <Icon className="text-foreground/80 size-5 shrink-0" aria-hidden="true" />
                  <span className="flex min-w-0 flex-1 flex-col items-start gap-1">
                    <span className="text-base font-semibold">{report.title}</span>
                    <span className="text-muted-foreground text-sm leading-snug">
                      {report.description}
                    </span>
                  </span>
                </Link>
              </Button>
            );
          })}
        </div>
      </PageContent>
    </Page>
  );
}
