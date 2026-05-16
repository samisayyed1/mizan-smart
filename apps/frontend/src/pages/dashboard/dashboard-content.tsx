import {
  getConcentrationFragilitySummary,
  getEstimatedHistoricalValuation,
  getFeeIntelligenceSummary,
} from "@/adapters";
import { HistoryChart } from "@/components/history-chart";
import { ExplainableNumber } from "@/components/explainable-number";
import { useHapticFeedback } from "@/hooks";
import { useHoldings } from "@/hooks/use-holdings";
import { useValuationHistory } from "@/hooks/use-valuation-history";
import type { AccountValuation } from "@/lib/types";
import {
  HoldingType,
  isAlternativeAssetKind,
  PORTFOLIO_ACCOUNT_ID,
  type AssetKind,
} from "@/lib/constants";
import { useSettingsContext } from "@/lib/settings-provider";
import { DateRange, TimePeriod } from "@/lib/types";
import { calculatePerformanceMetrics } from "@/lib/utils";
import { PortfolioUpdateTrigger } from "@/pages/dashboard/portfolio-update-trigger";
import type { TimePeriod as UITimePeriod } from "@mizan/ui";
import {
  GainAmount,
  GainPercent,
  getInitialIntervalData,
  IntervalSelector,
  usePersistentState,
} from "@mizan/ui";
import { Card, CardContent } from "@mizan/ui/components/ui/card";
import { Skeleton } from "@mizan/ui/components/ui/skeleton";
import { differenceInDays, format, parseISO } from "date-fns";
import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { AccountsSummary } from "./accounts-summary";
import Balance from "./balance";
import SavingGoals from "./goals";
import InboxPreview from "./inbox-preview";
import LiquidityLadderCard from "../liquidity-ladder/liquidity-ladder-card";
import QuickActions from "./quick-actions";
import TopHoldings from "./top-holdings";
import { ConcentrationRadarCard } from "./concentration-radar-card";

const DEFAULT_INTERVAL: UITimePeriod = "3M";
const INTERVAL_STORAGE_KEY = "dashboard-interval";

export function DashboardContent() {
  // Use the same persisted state as IntervalSelector for the interval code
  const [intervalCode] = usePersistentState<UITimePeriod>(INTERVAL_STORAGE_KEY, DEFAULT_INTERVAL);

  // Derive initial values from the persisted interval code
  const [dateRange, setDateRange] = useState<DateRange | undefined>(
    () => getInitialIntervalData(intervalCode).range,
  );
  const [selectedIntervalDescription, setSelectedIntervalDescription] = useState<string>(
    () => getInitialIntervalData(intervalCode).description,
  );
  const [isAllTime, setIsAllTime] = useState<boolean>(() => intervalCode === "ALL");

  const { holdings: allHoldings, isLoading: isHoldingsLoading } = useHoldings(PORTFOLIO_ACCOUNT_ID);
  const { triggerHaptic } = useHapticFeedback();

  // Filter holdings for display (exclude alternative assets and cash for TopHoldings)
  const holdings = useMemo(() => {
    if (!allHoldings) return [];
    return allHoldings.filter((h) => {
      // Exclude cash holdings from display
      if (h.holdingType === HoldingType.CASH) return false;
      // Exclude alternative assets from display
      if (h.assetKind && isAlternativeAssetKind(h.assetKind as AssetKind)) return false;
      return true;
    });
  }, [allHoldings]);

  // Total portfolio value (includes cash, excludes alternative assets)
  const totalValue = useMemo(() => {
    if (!allHoldings) return 0;
    return allHoldings
      .filter((h) => {
        return !(h.assetKind && isAlternativeAssetKind(h.assetKind as AssetKind));
      })
      .reduce((acc, holding) => acc + (holding.marketValue?.base ?? 0), 0);
  }, [allHoldings]);

  // Toggle: when ON, the chart shows an *estimated* historical curve
  // computed by pricing current holdings against historical quotes (good
  // for accounts where the broker only delivered a current snapshot).
  const [useEstimatedHistory, setUseEstimatedHistory] = useState(false);

  const { valuationHistory: realValuationHistory, isLoading: isRealHistoryLoading } =
    useValuationHistory(useEstimatedHistory ? undefined : dateRange);

  const { data: estimatedValuationHistory, isLoading: isEstimatedLoading } = useQuery<
    AccountValuation[]
  >({
    queryKey: ["estimated-historical-valuation", PORTFOLIO_ACCOUNT_ID],
    queryFn: () => getEstimatedHistoricalValuation(PORTFOLIO_ACCOUNT_ID),
    enabled: useEstimatedHistory,
    staleTime: 5 * 60 * 1000,
  });

  const valuationHistory = useEstimatedHistory ? estimatedValuationHistory : realValuationHistory;
  const isValuationHistoryLoading = useEstimatedHistory ? isEstimatedLoading : isRealHistoryLoading;

  const { settings } = useSettingsContext();
  const baseCurrency = settings?.baseCurrency ?? "USD";
  const feePeriodMonth = useMemo(() => new Date().toISOString().slice(0, 7), []);
  const feeSummary = useQuery({
    queryKey: ["fee-intelligence-summary", feePeriodMonth],
    queryFn: () => getFeeIntelligenceSummary(feePeriodMonth),
    staleTime: 5 * 60 * 1000,
  });
  const concentrationSummary = useQuery({
    queryKey: ["concentration-fragility-summary", baseCurrency],
    queryFn: () => getConcentrationFragilitySummary(baseCurrency),
    staleTime: 5 * 60 * 1000,
  });

  // Calculate gainLossAmount and simpleReturn from valuationHistory
  const { gainLossAmount, simpleReturn } = useMemo(() => {
    return calculatePerformanceMetrics(valuationHistory, isAllTime);
  }, [valuationHistory, isAllTime]);

  const currentValuation = useMemo(() => {
    return valuationHistory && valuationHistory.length > 0
      ? valuationHistory[valuationHistory.length - 1]
      : null;
  }, [valuationHistory]);

  const chartData = useMemo(() => {
    return (
      valuationHistory?.map((item) => ({
        date: item.valuationDate,
        totalValue: item.totalValue,
        netContribution: item.netContribution,
        currency: item.baseCurrency ?? baseCurrency,
      })) ?? []
    );
  }, [valuationHistory, baseCurrency]);

  const isNegative = totalValue < 0;

  // Sparse-data hint: when the user requests a wider window than they
  // actually have data for, the chart looks identical across multiple
  // intervals (1Y vs 3M etc) which feels broken. Detect that and surface
  // a subtle "data starts X" annotation so the chart's silence makes sense.
  const sparseDataHint = useMemo(() => {
    if (isAllTime || !valuationHistory?.length || !dateRange?.from) {
      return null;
    }
    const earliestRow = valuationHistory[0];
    if (!earliestRow?.valuationDate) return null;
    const earliestDataDate = parseISO(earliestRow.valuationDate);
    // Threshold: if the earliest data point is more than 7 days after the
    // requested window start, the chart is effectively clamped. Tell the
    // user with short, readable text — long hint strings overflow narrow
    // viewports and look bad in the monospace font we use here.
    const gapDays = differenceInDays(earliestDataDate, dateRange.from);
    if (gapDays <= 7) return null;
    const days = valuationHistory.length;
    return `Earliest data ${format(earliestDataDate, "MMM d, yyyy")} · ${days} day${days === 1 ? "" : "s"}`;
  }, [valuationHistory, dateRange, isAllTime]);

  // Headline interval label — when data is sparse vs the requested window,
  // override "past 5 years" / "past year" with the actual extent so the
  // gain/loss percentage isn't misleadingly labeled. e.g. requesting 5Y
  // on a 3-month-old account: instead of "-14.57% past 5 years" (which
  // sounds like a 5-year drawdown), render "-14.57% past 92 days".
  const displayedIntervalDescription = useMemo(() => {
    if (sparseDataHint && valuationHistory?.length) {
      const days = valuationHistory.length;
      if (days < 31) return `past ${days} days`;
      const months = Math.round(days / 30);
      return `past ${months} month${months === 1 ? "" : "s"}`;
    }
    return selectedIntervalDescription;
  }, [sparseDataHint, valuationHistory, selectedIntervalDescription]);

  // Callback for IntervalSelector
  const handleIntervalSelect = (
    code: TimePeriod,
    description: string,
    range: DateRange | undefined,
  ) => {
    setSelectedIntervalDescription(description);
    setDateRange(range);
    setIsAllTime(code === "ALL");
  };

  return (
    <div className="flex min-h-screen flex-col">
      {/* HEADLINE — balance, change, time range. Sits above the chart
          with quiet breathing room. The gradient backdrop is bounded
          to the chart area below, not washed across the whole page. */}
      <div className="px-4 pb-3 pt-4 md:px-6 md:pb-4 md:pt-6 lg:px-8">
        <PortfolioUpdateTrigger lastCalculatedAt={currentValuation?.calculatedAt}>
          <div className="flex flex-col gap-1">
            <Balance
              isLoading={isHoldingsLoading}
              targetValue={totalValue}
              currency={baseCurrency}
              displayCurrency={true}
            />
            {/* Gain block + Explain-This-Number icon button on one row.
                The icon button is sized as a small ghost so it reads as a
                caption affordance, not a competing CTA. */}
            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm lg:text-base">
              {isValuationHistoryLoading && !valuationHistory ? (
                <div className="flex items-center gap-3">
                  <Skeleton className="h-4 w-24" />
                  <div className="border-secondary h-4 border-r" />
                  <Skeleton className="h-4 w-16" />
                </div>
              ) : (
                <>
                  <GainAmount
                    className="text-sm font-light lg:text-base"
                    value={gainLossAmount}
                    currency={baseCurrency}
                    displayCurrency={false}
                  />
                  <div className="border-secondary h-4 border-r" />
                  <GainPercent
                    className="text-sm font-light lg:text-base"
                    value={simpleReturn}
                    animated={true}
                  />
                </>
              )}
              {displayedIntervalDescription && (
                <span className="text-muted-foreground text-sm font-light lg:text-base">
                  {displayedIntervalDescription}
                </span>
              )}
              <ExplainableNumber
                entityType="portfolio"
                entityId="total"
                metricType="net_worth"
              />
            </div>
          </div>
        </PortfolioUpdateTrigger>
      </div>

      {/* CHART REGION — the gradient is bounded HERE only, so it frames
          the chart without bleeding into the cards below. */}
      <div
        className={`bg-linear-to-t flex flex-col ${
          isNegative
            ? "from-destructive/20 via-destructive/10 to-transparent"
            : "from-success/20 via-success/10 to-transparent"
        }`}
      >
        <div className="h-[280px]">
          <HistoryChart data={chartData} isLoading={isValuationHistoryLoading} />
          {valuationHistory && chartData.length > 0 && (
            <div className="flex w-full flex-col items-center gap-3 pt-1">
              {!useEstimatedHistory && (
                <IntervalSelector
                  className="pointer-events-auto relative z-20 w-full max-w-screen-sm sm:max-w-screen-md md:max-w-2xl lg:max-w-3xl"
                  onIntervalSelect={handleIntervalSelect}
                  onHaptic={triggerHaptic}
                  isLoading={isValuationHistoryLoading}
                  storageKey={INTERVAL_STORAGE_KEY}
                  defaultValue={DEFAULT_INTERVAL}
                />
              )}
              <div className="flex w-full max-w-md flex-col items-center gap-2 px-4">
                {sparseDataHint && !useEstimatedHistory && (
                  <p
                    className="text-muted-foreground pointer-events-auto max-w-full truncate text-center text-[11px] leading-tight tracking-wide sm:text-xs"
                    title={sparseDataHint}
                  >
                    {sparseDataHint}
                  </p>
                )}
                {useEstimatedHistory && (
                  <p className="pointer-events-auto text-balance text-center text-[11px] leading-snug tracking-wide text-amber-500/85 sm:text-xs">
                    Estimated. Current holdings priced against historical quotes &mdash; past
                    trades, splits, and contributions are not reflected.
                  </p>
                )}
                <button
                  type="button"
                  onClick={() => {
                    setUseEstimatedHistory((v) => !v);
                    triggerHaptic();
                  }}
                  className="border-border/60 bg-background/60 text-muted-foreground hover:text-foreground hover:border-border pointer-events-auto cursor-pointer rounded-full border px-3 py-1 text-[11px] font-medium tracking-wide transition-colors sm:text-xs"
                >
                  {useEstimatedHistory ? "Back to actual history" : "Estimate full history"}
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* CARD GRID — left column = Accounts summary (the long one),
          right rail = action-first stack. Column gap is the standard
          24px dashboard rhythm; right rail uses uniform 24px between
          cards so the surfaces line up cleanly. */}
      <div className="grow px-4 pb-[calc(var(--mobile-nav-ui-height)+max(var(--mobile-nav-gap),env(safe-area-inset-bottom)))] pt-8 md:px-6 md:pb-6 md:pt-10 lg:px-8 lg:pb-8">
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
          <div className="lg:col-span-2">
            <AccountsSummary dateRange={dateRange} isAllTime={isAllTime} />
          </div>
          {/* Right rail hierarchy:
                1. Fee spike alert (only when triggered)
                2. Inbox preview — "what needs attention right now"
                3. Quick actions — entry points
                4. Liquidity ladder — forward cashflow
                5. Top holdings — biggest positions
                6. Concentration radar — risk lens
                7. Goals — long horizon
              Lower-frequency surfaces sit further down so the eye
              hits action-oriented content first. */}
          <div className="space-y-6 lg:col-span-1">
            {feeSummary.data?.spike ? <FeeSpikeWarning spike={feeSummary.data.spike} /> : null}
            <InboxPreview />
            <QuickActions />
            <LiquidityLadderCard />
            <TopHoldings
              holdings={holdings}
              isLoading={isHoldingsLoading}
              baseCurrency={baseCurrency}
            />
            {concentrationSummary.data ? (
              <ConcentrationRadarCard summary={concentrationSummary.data} />
            ) : null}
            <SavingGoals />
          </div>
        </div>
      </div>
    </div>
  );
}

function FeeSpikeWarning({
  spike,
}: {
  spike: {
    currency: string;
    currentPeriodTotal: string;
    priorAverage: string;
    multiple: string;
  };
}) {
  return (
    <Card data-testid="fee-spike-warning" className="border-warning/50">
      <CardContent className="space-y-1 p-4">
        <p className="text-sm font-medium">Fee spike detected</p>
        <p className="text-muted-foreground text-sm">
          Recorded fees are {spike.currentPeriodTotal} {spike.currency}; prior average{" "}
          {spike.priorAverage} {spike.currency}.
        </p>
      </CardContent>
    </Card>
  );
}

export default DashboardContent;
