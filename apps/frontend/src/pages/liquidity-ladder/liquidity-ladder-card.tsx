import { getLiquidityLadder } from "@/adapters";
import { Icons } from "@mizan/ui";
import { Card, CardContent, CardHeader } from "@mizan/ui/components/ui/card";
import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { datedItemCount, findView, formatDecimalMoney } from "./liquidity-format";

export function LiquidityLadderCard() {
  const query = useQuery({
    queryKey: ["liquidity-ladder", "dashboard-card"],
    queryFn: () => getLiquidityLadder(),
    staleTime: 60_000,
  });
  const view = findView(query.data?.views, "next_30_days");
  const groups = view?.currencyGroups ?? [];
  const itemCount = groups.reduce((total, group) => total + datedItemCount(group), 0);
  const primaryGroup = groups[0] ?? null;

  return (
    <Card data-testid="liquidity-ladder-card">
      <CardHeader className="flex flex-row items-center justify-between pb-2">
        <h2 className="text-base font-semibold">Liquidity ladder</h2>
        <Link
          to="/liquidity-ladder"
          data-testid="liquidity-ladder-open"
          className="text-muted-foreground hover:text-foreground inline-flex items-center gap-1 text-xs"
        >
          Open
          <Icons.ArrowRight className="size-3" aria-hidden="true" />
        </Link>
      </CardHeader>
      <CardContent className="space-y-3 pb-3">
        {query.isLoading && !query.data ? (
          <p className="text-muted-foreground text-sm">Reading scheduled cashflows...</p>
        ) : !view || groups.length === 0 ? (
          <p data-testid="liquidity-ladder-empty" className="text-muted-foreground text-sm">
            No cash balances or dated cashflows are recorded for the next 30 days.
          </p>
        ) : (
          <>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div>
                <p className="text-muted-foreground text-xs">Dated items</p>
                <p className="font-semibold">{itemCount}</p>
              </div>
              <div>
                <p className="text-muted-foreground text-xs">Currencies</p>
                <p className="font-semibold">{groups.length}</p>
              </div>
            </div>
            {primaryGroup ? (
              <div className="rounded-md border p-3">
                <p className="text-muted-foreground text-xs">{primaryGroup.currency}</p>
                <p className="text-sm font-medium">
                  Confirmed runway {formatDecimalMoney(primaryGroup.netConfirmed, primaryGroup.currency)}
                </p>
                <p className="text-muted-foreground text-xs">
                  Expected after scheduled items{" "}
                  {formatDecimalMoney(primaryGroup.netExpected, primaryGroup.currency)}
                </p>
              </div>
            ) : null}
            <p className="text-muted-foreground text-xs">
              Expected and confirmed cashflows are labeled separately.
            </p>
          </>
        )}
      </CardContent>
    </Card>
  );
}

export default LiquidityLadderCard;
