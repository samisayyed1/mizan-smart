import { Card, CardContent, CardHeader } from "@mizan/ui/components/ui/card";
import { Icons } from "@mizan/ui";
import { Link } from "react-router-dom";

// Senior-friendly Quick Actions card on the Home dashboard.
// Specified in docs/mizan-smart-plan/PLAN.md Prompt 3.
//
// Every action routes to a route that exists today. As later phases land
// (universal Add Asset, bulk valuations, document upload, report builder),
// the same hrefs will resolve to richer surfaces — no UI rewiring required.

type QuickActionIcon = "Plus" | "RefreshCw" | "FileText" | "Inbox" | "PieChart";

interface QuickAction {
  title: string;
  description: string;
  href: string;
  icon: QuickActionIcon;
}

const ACTIONS: QuickAction[] = [
  {
    title: "Add asset",
    description: "Stock, fund, bond, property, gold, crypto, or other.",
    href: "/holdings/new",
    icon: "Plus",
  },
  {
    title: "Update values",
    description: "Refresh manual valuations for property and private assets.",
    href: "/holdings/update-values",
    icon: "RefreshCw",
  },
  {
    title: "Upload document",
    description: "Statements, factsheets, and source files.",
    href: "/documents",
    icon: "FileText",
  },
  {
    title: "Review inbox",
    description: "Anything that needs your attention.",
    href: "/inbox",
    icon: "Inbox",
  },
  {
    title: "Generate report",
    description: "Performance, income, breakdowns, and data health.",
    href: "/reports",
    icon: "PieChart",
  },
];

export function QuickActions() {
  return (
    <Card data-testid="quick-actions">
      <CardHeader className="pb-3">
        <h2 className="text-base font-semibold">Quick actions</h2>
      </CardHeader>
      <CardContent className="space-y-2">
        {ACTIONS.map((action) => {
          const Icon = Icons[action.icon];
          return (
            <Link
              key={action.title}
              to={action.href}
              data-testid={`quick-action-${action.icon.toLowerCase()}`}
              className="hover:bg-muted/60 flex items-center gap-3 rounded-md px-2 py-2 transition-colors"
            >
              <span className="bg-muted text-foreground/80 flex size-9 shrink-0 items-center justify-center rounded-md">
                <Icon className="size-4" aria-hidden="true" />
              </span>
              <span className="min-w-0 flex-1">
                <span className="block text-sm font-medium">{action.title}</span>
                <span className="text-muted-foreground block truncate text-xs">
                  {action.description}
                </span>
              </span>
            </Link>
          );
        })}
      </CardContent>
    </Card>
  );
}

export default QuickActions;
