import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";

import { listWealthInboxItems, type InboxItem, type InboxItemType } from "@/adapters";
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

const FILTERS = [
  { label: "All", value: "all" },
  { label: "Documents", value: "document" },
  { label: "Valuations", value: "valuation" },
  { label: "Tax", value: "tax" },
  { label: "Income", value: "income" },
  { label: "Private Investments", value: "private_investment" },
  { label: "Security", value: "security" },
  { label: "AI Suggestions", value: "ai_suggestion" },
  { label: "Web Evidence", value: "web_evidence" },
] as const satisfies readonly { label: string; value: InboxItemType | "all" }[];

type FilterValue = (typeof FILTERS)[number]["value"];
type SortValue = "critical" | "due" | "newest";

const SEVERITY_LABEL = {
  critical: "Critical",
  warning: "Review",
  info: "Info",
} as const;

const SEVERITY_DOT = {
  critical: "bg-destructive",
  warning: "bg-warning",
  info: "bg-muted-foreground",
} as const;

const SEVERITY_RANK = {
  critical: 0,
  warning: 1,
  info: 2,
} as const;

function sortItems(items: InboxItem[], sort: SortValue): InboxItem[] {
  return items.slice().sort((a, b) => {
    if (sort === "newest") {
      return b.createdAt.localeCompare(a.createdAt) || a.id.localeCompare(b.id);
    }
    if (sort === "due") {
      return compareDueDates(a.dueDate, b.dueDate) || compareSeverity(a, b) || a.id.localeCompare(b.id);
    }
    return compareSeverity(a, b) || compareDueDates(a.dueDate, b.dueDate) || b.createdAt.localeCompare(a.createdAt);
  });
}

function compareSeverity(a: InboxItem, b: InboxItem): number {
  return SEVERITY_RANK[a.severity] - SEVERITY_RANK[b.severity];
}

function compareDueDates(a: string | null, b: string | null): number {
  if (a && b) {
    return a.localeCompare(b);
  }
  if (a) {
    return -1;
  }
  if (b) {
    return 1;
  }
  return 0;
}

function formatType(type: InboxItemType): string {
  return type
    .split("_")
    .map((part) => part[0]?.toUpperCase() + part.slice(1))
    .join(" ");
}

function InboxRow({ item }: { item: InboxItem }) {
  return (
    <li
      data-testid="inbox-item"
      data-severity={item.severity}
      data-type={item.itemType}
      className="flex items-start gap-3 rounded-md border bg-card px-4 py-4"
    >
      <span
        aria-hidden="true"
        className={cn("mt-2 h-2 w-2 shrink-0 rounded-full", SEVERITY_DOT[item.severity])}
      />
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h3 className="text-base font-semibold">{item.title}</h3>
          <Badge variant="outline" className="text-xs">
            {SEVERITY_LABEL[item.severity]}
          </Badge>
          <Badge variant="secondary" className="text-xs">
            {formatType(item.itemType)}
          </Badge>
        </div>
        <p className="text-muted-foreground mt-1 text-sm">{item.description}</p>
        {item.dueDate && (
          <p className="text-muted-foreground mt-2 text-xs">Due {item.dueDate}</p>
        )}
      </div>
      <Button asChild size="sm" variant="secondary">
        <Link data-testid="inbox-action" to={item.actionRoute}>
          Open
        </Link>
      </Button>
    </li>
  );
}

export default function InboxPage() {
  const [items, setItems] = useState<InboxItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<FilterValue>("all");
  const [sort, setSort] = useState<SortValue>("critical");

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    void listWealthInboxItems()
      .then((loaded) => {
        if (!cancelled) {
          setItems(loaded);
          setError(null);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const visibleItems = useMemo(() => {
    const filtered =
      filter === "all" ? items : items.filter((item) => item.itemType === filter);
    return sortItems(filtered, sort);
  }, [filter, items, sort]);

  return (
    <Page>
      <PageHeader heading="Inbox" text="Everything that needs your attention, in one place." />
      <PageContent>
        <div className="mb-4 flex flex-wrap items-center gap-3">
          <label className="text-sm font-medium">
            Filter
            <select
              className="border-input bg-background ml-2 rounded-md border px-3 py-2 text-sm"
              value={filter}
              onChange={(event) => setFilter(event.target.value as FilterValue)}
            >
              {FILTERS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <label className="text-sm font-medium">
            Sort
            <select
              className="border-input bg-background ml-2 rounded-md border px-3 py-2 text-sm"
              value={sort}
              onChange={(event) => setSort(event.target.value as SortValue)}
            >
              <option value="critical">Critical first</option>
              <option value="due">Due soon</option>
              <option value="newest">Newest</option>
            </select>
          </label>
        </div>

        {isLoading ? (
          <div className="space-y-3" data-testid="inbox-loading">
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-20 w-full" />
          </div>
        ) : error ? (
          <div className="rounded-md border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm">
            {error}
          </div>
        ) : visibleItems.length === 0 ? (
          <div
            data-testid="inbox-empty"
            className="rounded-lg border border-dashed bg-muted/30 px-6 py-12 text-center"
          >
            <Icons.CheckCircle className="text-muted-foreground mx-auto size-10" aria-hidden="true" />
            <p className="mt-4 text-base font-medium">Nothing needs attention</p>
          </div>
        ) : (
          <ul className="space-y-3" data-testid="inbox-list">
            {visibleItems.map((item) => (
              <InboxRow key={item.id} item={item} />
            ))}
          </ul>
        )}
      </PageContent>
    </Page>
  );
}
