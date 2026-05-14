import { getLiquidityLadder, type LiquidityLadderView, type LiquidityLadderWindow } from "@/adapters";
import { Button, Icons, Page, PageContent, PageHeader } from "@mizan/ui";
import { Card, CardContent, CardHeader } from "@mizan/ui/components/ui/card";
import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import {
  confidenceLabel,
  directionLabel,
  findView,
  formatDecimalMoney,
  WINDOW_LABELS,
} from "./liquidity-format";

const WINDOWS: LiquidityLadderWindow[] = ["next_30_days", "next_90_days", "next_12_months"];

export default function LiquidityLadderPage() {
  const [selectedWindow, setSelectedWindow] = useState<LiquidityLadderWindow>("next_30_days");
  const query = useQuery({
    queryKey: ["liquidity-ladder", "detail"],
    queryFn: () => getLiquidityLadder(),
    staleTime: 60_000,
  });
  const view = findView(query.data?.views, selectedWindow);
  const allItems = useMemo(
    () => view?.currencyGroups.flatMap((group) => group.items) ?? [],
    [view],
  );

  return (
    <Page>
      <PageHeader heading="Liquidity Ladder" />
      <PageContent className="space-y-6">
        <div className="flex flex-wrap gap-2">
          {WINDOWS.map((window) => (
            <Button
              key={window}
              type="button"
              variant={window === selectedWindow ? "default" : "outline"}
              size="sm"
              onClick={() => setSelectedWindow(window)}
            >
              {WINDOW_LABELS[window]}
            </Button>
          ))}
        </div>

        {query.isLoading && !query.data ? (
          <Card>
            <CardContent className="py-8 text-sm text-muted-foreground">
              Reading cash balances and scheduled cashflows...
            </CardContent>
          </Card>
        ) : !view || view.currencyGroups.length === 0 ? (
          <Card>
            <CardHeader>
              <h2 className="text-lg font-semibold">No scheduled cashflows</h2>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-muted-foreground">
              <p data-testid="liquidity-ladder-empty">
                Mizan found no cash balances or dated cashflows in this window.
              </p>
              <p>
                Future dividends, taxes, and insurance premiums are not estimated unless they are
                already recorded with dates.
              </p>
            </CardContent>
          </Card>
        ) : (
          <>
            <section className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
              {view.currencyGroups.map((group) => (
                <Card key={group.currency} data-testid="liquidity-currency-group">
                  <CardHeader className="pb-2">
                    <h2 className="text-base font-semibold">{group.currency}</h2>
                  </CardHeader>
                  <CardContent className="space-y-2 text-sm">
                    <Metric
                      label="Available cash"
                      value={formatDecimalMoney(group.availableCash, group.currency)}
                    />
                    <Metric
                      label="Confirmed net"
                      value={formatDecimalMoney(group.netConfirmed, group.currency)}
                    />
                    <Metric
                      label="Expected net"
                      value={formatDecimalMoney(group.netExpected, group.currency)}
                    />
                    <div className="grid grid-cols-2 gap-2 pt-1 text-xs text-muted-foreground">
                      <span>Confirmed in {formatDecimalMoney(group.confirmedIncoming, group.currency)}</span>
                      <span>Expected in {formatDecimalMoney(group.expectedIncoming, group.currency)}</span>
                      <span>Confirmed out {formatDecimalMoney(group.confirmedOutgoing, group.currency)}</span>
                      <span>Expected out {formatDecimalMoney(group.expectedOutgoing, group.currency)}</span>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </section>

            <Timeline view={view} />

            <Card>
              <CardHeader>
                <h2 className="text-lg font-semibold">Cashflow table</h2>
              </CardHeader>
              <CardContent>
                <div className="overflow-x-auto">
                  <table className="w-full min-w-[720px] text-left text-sm">
                    <thead className="text-muted-foreground border-b text-xs uppercase">
                      <tr>
                        <th className="py-2 pr-3">Date</th>
                        <th className="py-2 pr-3">Item</th>
                        <th className="py-2 pr-3">Direction</th>
                        <th className="py-2 pr-3">Status</th>
                        <th className="py-2 pr-3 text-right">Amount</th>
                      </tr>
                    </thead>
                    <tbody>
                      {allItems.map((item) => (
                        <tr key={item.id} className="border-b last:border-b-0">
                          <td className="py-2 pr-3">{item.date}</td>
                          <td className="py-2 pr-3">
                            <span className="font-medium">{item.label}</span>
                            {item.notes ? (
                              <span className="text-muted-foreground block text-xs">
                                {item.notes}
                              </span>
                            ) : null}
                          </td>
                          <td className="py-2 pr-3">{directionLabel(item.direction)}</td>
                          <td className="py-2 pr-3">
                            <span data-testid={`confidence-${item.confidence}`}>
                              {confidenceLabel(item.confidence)}
                            </span>
                          </td>
                          <td className="py-2 pr-3 text-right">
                            {formatDecimalMoney(item.amount, item.currency)}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </CardContent>
            </Card>

            <div className="space-y-1 text-sm text-muted-foreground">
              {view.warnings.map((warning) => (
                <p key={warning}>{warning}</p>
              ))}
            </div>
          </>
        )}
      </PageContent>
    </Page>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-medium">{value}</span>
    </div>
  );
}

function Timeline({ view }: { view: LiquidityLadderView }) {
  const items = view.currencyGroups
    .flatMap((group) => group.items)
    .filter((item) => item.itemType !== "cash_balance")
    .slice(0, 12);
  return (
    <Card>
      <CardHeader>
        <h2 className="text-lg font-semibold">Timeline</h2>
      </CardHeader>
      <CardContent>
        {items.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            Only current cash balances are available in this window.
          </p>
        ) : (
          <ol className="space-y-3">
            {items.map((item) => (
              <li key={item.id} className="flex items-start gap-3">
                <span className="bg-muted flex size-8 shrink-0 items-center justify-center rounded-md">
                  <Icons.Calendar className="size-4" aria-hidden="true" />
                </span>
                <span className="min-w-0 flex-1">
                  <span className="flex flex-wrap items-center gap-x-2 gap-y-1">
                    <span className="font-medium">{item.label}</span>
                    <span className="text-muted-foreground text-xs">
                      {confidenceLabel(item.confidence)}
                    </span>
                  </span>
                  <span className="text-muted-foreground block text-sm">
                    {item.date} · {directionLabel(item.direction)} ·{" "}
                    {formatDecimalMoney(item.amount, item.currency)}
                  </span>
                </span>
              </li>
            ))}
          </ol>
        )}
      </CardContent>
    </Card>
  );
}
