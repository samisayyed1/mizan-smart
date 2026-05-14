import { zodResolver } from "@hookform/resolvers/zod";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useForm, type UseFormRegisterReturn, type UseFormReturn } from "react-hook-form";
import { Link, useParams } from "react-router-dom";
import { z } from "zod";

import {
  addCapitalCall,
  addPrivateDistribution,
  addPrivateInvestmentValuation,
  getPrivateInvestmentDetail,
  updateCapitalCallStatus,
  type CapitalCall,
  type CapitalCallStatus,
  type PrivateInvestmentDetail,
  type PrivateInvestmentJCurvePoint,
} from "@/adapters";
import { Button, Icons, Input, Label, Page, PageContent, PageHeader } from "@mizan/ui";
import { Card, CardContent, CardHeader } from "@mizan/ui/components/ui/card";

const moneyString = z
  .string()
  .trim()
  .min(1, "Amount is required")
  .regex(/^\d+(\.\d+)?$/, "Use a non-negative decimal amount");
const dateString = z.string().min(1, "Date is required");
const currencyString = z
  .string()
  .trim()
  .length(3, "Use a 3-letter currency")
  .regex(/^[A-Z]{3}$/, "Use uppercase ISO currency");

const capitalCallSchema = z.object({
  noticeDate: dateString,
  dueDate: dateString,
  amount: moneyString,
  currency: currencyString,
  status: z.enum(["expected", "due", "paid", "cancelled"]),
  notes: z.string().optional(),
});

const valuationSchema = z.object({
  valuationDate: dateString,
  nav: moneyString,
  currency: currencyString,
});

const distributionSchema = z.object({
  distributionDate: dateString,
  amount: moneyString,
  currency: currencyString,
  recallable: z.boolean(),
  notes: z.string().optional(),
});

type CapitalCallFormValues = z.infer<typeof capitalCallSchema>;
type ValuationFormValues = z.infer<typeof valuationSchema>;
type DistributionFormValues = z.infer<typeof distributionSchema>;

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

function nullableText(value: string | undefined): string | null {
  const trimmed = value?.trim() ?? "";
  return trimmed.length > 0 ? trimmed : null;
}

export default function PrivateInvestmentDetailPage() {
  const { assetId } = useParams<{ assetId: string }>();
  const [detail, setDetail] = useState<PrivateInvestmentDetail | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);

  const capitalCallForm = useForm<CapitalCallFormValues>({
    resolver: zodResolver(capitalCallSchema),
    defaultValues: {
      noticeDate: todayIso(),
      dueDate: todayIso(),
      amount: "",
      currency: "USD",
      status: "due",
      notes: "",
    },
  });
  const valuationForm = useForm<ValuationFormValues>({
    resolver: zodResolver(valuationSchema),
    defaultValues: { valuationDate: todayIso(), nav: "", currency: "USD" },
  });
  const distributionForm = useForm<DistributionFormValues>({
    resolver: zodResolver(distributionSchema),
    defaultValues: {
      distributionDate: todayIso(),
      amount: "",
      currency: "USD",
      recallable: false,
      notes: "",
    },
  });

  const loadDetail = useCallback(async () => {
    if (!assetId) {
      setError("Missing private investment asset id.");
      setIsLoading(false);
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      setDetail(await getPrivateInvestmentDetail(assetId));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not load private investment.");
    } finally {
      setIsLoading(false);
    }
  }, [assetId]);

  useEffect(() => {
    void loadDetail();
  }, [loadDetail]);

  const currency = detail?.summary.investment.commitmentCurrency ?? "USD";

  const upcomingCalls = useMemo(
    () => detail?.upcomingCapitalCalls.filter((call) => call.status !== "cancelled") ?? [],
    [detail],
  );

  async function submitCapitalCall(values: CapitalCallFormValues) {
    if (!assetId) return;
    setBusyAction("call");
    setStatusMessage(null);
    setError(null);
    try {
      await addCapitalCall({
        assetId,
        noticeDate: values.noticeDate,
        dueDate: values.dueDate,
        amount: values.amount.trim(),
        currency: values.currency.trim().toUpperCase(),
        status: values.status,
        sourceCitationId: null,
        notes: nullableText(values.notes),
      });
      capitalCallForm.reset({
        noticeDate: todayIso(),
        dueDate: todayIso(),
        amount: "",
        currency,
        status: "due",
        notes: "",
      });
      setStatusMessage("Capital call saved.");
      await loadDetail();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not add capital call.");
    } finally {
      setBusyAction(null);
    }
  }

  async function submitValuation(values: ValuationFormValues) {
    if (!assetId) return;
    setBusyAction("valuation");
    setStatusMessage(null);
    setError(null);
    try {
      await addPrivateInvestmentValuation({
        assetId,
        valuationDate: values.valuationDate,
        nav: values.nav.trim(),
        currency: values.currency.trim().toUpperCase(),
        sourceCitationId: null,
      });
      valuationForm.reset({ valuationDate: todayIso(), nav: "", currency });
      setStatusMessage("NAV updated.");
      await loadDetail();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not update NAV.");
    } finally {
      setBusyAction(null);
    }
  }

  async function submitDistribution(values: DistributionFormValues) {
    if (!assetId) return;
    setBusyAction("distribution");
    setStatusMessage(null);
    setError(null);
    try {
      await addPrivateDistribution({
        assetId,
        distributionDate: values.distributionDate,
        amount: values.amount.trim(),
        currency: values.currency.trim().toUpperCase(),
        recallable: values.recallable,
        sourceCitationId: null,
        notes: nullableText(values.notes),
      });
      distributionForm.reset({
        distributionDate: todayIso(),
        amount: "",
        currency,
        recallable: false,
        notes: "",
      });
      setStatusMessage("Distribution saved.");
      await loadDetail();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not add distribution.");
    } finally {
      setBusyAction(null);
    }
  }

  async function markCallPaid(call: CapitalCall) {
    setBusyAction(call.id);
    setStatusMessage(null);
    setError(null);
    try {
      await updateCapitalCallStatus({ id: call.id, status: "paid" });
      setStatusMessage("Capital call marked paid.");
      await loadDetail();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not mark call paid.");
    } finally {
      setBusyAction(null);
    }
  }

  if (isLoading) {
    return (
      <Page>
        <PageHeader heading="Private Investment" />
        <PageContent className="py-12 text-center text-sm text-muted-foreground">
          Loading private investment...
        </PageContent>
      </Page>
    );
  }

  if (!detail) {
    return (
      <Page>
        <PageHeader heading="Private Investment" />
        <PageContent className="space-y-4">
          {error ? <p className="text-sm text-destructive">{error}</p> : null}
          <Card>
            <CardHeader>
              <h2 className="text-lg font-semibold">No private investment record</h2>
            </CardHeader>
            <CardContent className="space-y-3 text-sm text-muted-foreground">
              <p>This asset has no private investment terms, calls, distributions, or NAVs yet.</p>
              <Button asChild variant="outline">
                <Link to="/holdings/new">
                  <Icons.Plus className="size-4" aria-hidden="true" />
                  Add private investment
                </Link>
              </Button>
            </CardContent>
          </Card>
        </PageContent>
      </Page>
    );
  }

  const { summary } = detail;

  return (
    <Page>
      <PageHeader heading={summary.investment.manager} />
      <PageContent className="space-y-6 pb-28">
        <div>
          <p className="text-sm text-muted-foreground">
            {summary.investment.strategy}
            {summary.investment.vintageYear ? ` · Vintage ${summary.investment.vintageYear}` : ""}
          </p>
          {summary.investment.notes ? <p className="mt-2 text-sm">{summary.investment.notes}</p> : null}
        </div>

        {error ? <p className="rounded-md bg-destructive/10 px-4 py-3 text-sm text-destructive">{error}</p> : null}
        {statusMessage ? <p className="rounded-md bg-muted px-4 py-3 text-sm">{statusMessage}</p> : null}

        <section className="grid gap-3 md:grid-cols-3 xl:grid-cols-5">
          <MetricCard label="Commitment" value={formatMoney(summary.commitment, currency)} />
          <MetricCard label="Paid-in" value={formatMoney(summary.paidInCapital, currency)} />
          <MetricCard label="Unfunded" value={formatMoney(summary.unfundedCommitment, currency)} />
          <MetricCard label="NAV" value={formatMoney(summary.currentNav, currency)} />
          <MetricCard label="Distributions" value={formatMoney(summary.totalDistributions, currency)} />
          <MetricCard label="DPI" value={formatRatio(summary.dpi)} />
          <MetricCard label="RVPI" value={formatRatio(summary.rvpi)} />
          <MetricCard label="TVPI" value={formatRatio(summary.tvpi)} />
          <MetricCard label="MOIC" value={formatRatio(summary.moic)} />
          <MetricCard label="Recallable" value={formatMoney(summary.recallableDistributions, currency)} />
        </section>

        {summary.warnings.length > 0 ? (
          <section className="rounded-md border border-amber-300 bg-amber-50 p-4 text-sm text-amber-950">
            <h2 className="font-medium">Data checks</h2>
            <ul className="mt-2 list-disc space-y-1 pl-5">
              {summary.warnings.map((warning) => (
                <li key={warning}>{warning}</li>
              ))}
            </ul>
          </section>
        ) : null}

        <section className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
          <Card>
            <CardHeader>
              <h2 className="text-lg font-semibold">J-curve</h2>
            </CardHeader>
            <CardContent>
              <JCurveChart points={detail.jCurve} currency={currency} />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <h2 className="text-lg font-semibold">Upcoming capital calls</h2>
            </CardHeader>
            <CardContent>
              {upcomingCalls.length === 0 ? (
                <p className="text-sm text-muted-foreground">No expected or due capital calls.</p>
              ) : (
                <div className="space-y-3">
                  {upcomingCalls.map((call) => (
                    <CallRow
                      key={call.id}
                      call={call}
                      busy={busyAction === call.id}
                      onMarkPaid={() => void markCallPaid(call)}
                    />
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </section>

        <section className="grid gap-6 xl:grid-cols-3">
          <CapitalCallForm
            form={capitalCallForm}
            busy={busyAction === "call"}
            onSubmit={(values) => void submitCapitalCall(values)}
          />
          <ValuationForm
            form={valuationForm}
            busy={busyAction === "valuation"}
            onSubmit={(values) => void submitValuation(values)}
          />
          <DistributionForm
            form={distributionForm}
            busy={busyAction === "distribution"}
            onSubmit={(values) => void submitDistribution(values)}
          />
        </section>

        <section className="grid gap-6 xl:grid-cols-3">
          <EventsCard title="Capital calls" rows={detail.capitalCalls.map(formatCapitalCall)} />
          <EventsCard title="Distributions" rows={detail.distributions.map(formatDistribution)} />
          <EventsCard title="Linked documents" rows={detail.sourceCitationIds.map(formatCitation)} />
        </section>
      </PageContent>
    </Page>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <Card>
      <CardContent className="p-4">
        <div className="text-xs font-medium uppercase text-muted-foreground">{label}</div>
        <div className="mt-2 text-xl font-semibold">{value}</div>
      </CardContent>
    </Card>
  );
}

function JCurveChart({
  points,
  currency,
}: {
  points: PrivateInvestmentJCurvePoint[];
  currency: string;
}) {
  if (points.length === 0) {
    return (
      <div className="flex h-64 items-center justify-center rounded-md border text-sm text-muted-foreground">
        No cashflow or NAV data yet.
      </div>
    );
  }

  const values = points.flatMap((point) => [
    decimalToNumber(point.cumulativeNetCashflow),
    ...(point.nav ? [decimalToNumber(point.nav)] : []),
  ]);
  const min = Math.min(...values, 0);
  const max = Math.max(...values, 0);
  const range = max - min || 1;
  const xFor = (index: number) => 40 + (index * 560) / Math.max(points.length - 1, 1);
  const yFor = (value: number) => 190 - ((value - min) * 160) / range;
  const cashflowPolyline = points
    .map((point, index) => `${xFor(index)},${yFor(decimalToNumber(point.cumulativeNetCashflow))}`)
    .join(" ");
  const navPolyline = points
    .map((point, index) => (point.nav ? `${xFor(index)},${yFor(decimalToNumber(point.nav))}` : null))
    .filter((value): value is string => value !== null)
    .join(" ");

  return (
    <div>
      <svg role="img" aria-label="Private investment J-curve" viewBox="0 0 640 220" className="h-64 w-full">
        <line x1="40" y1={yFor(0)} x2="600" y2={yFor(0)} stroke="currentColor" opacity="0.25" />
        <polyline points={cashflowPolyline} fill="none" stroke="var(--chart-1)" strokeWidth="4" />
        {navPolyline ? (
          <polyline points={navPolyline} fill="none" stroke="var(--chart-2)" strokeWidth="3" />
        ) : null}
      </svg>
      <div className="flex flex-wrap gap-4 text-xs text-muted-foreground">
        <span>Cumulative net cashflow</span>
        <span>NAV overlay</span>
        <span>{currency}</span>
      </div>
    </div>
  );
}

function CapitalCallForm({
  form,
  busy,
  onSubmit,
}: {
  form: UseFormReturn<CapitalCallFormValues>;
  busy: boolean;
  onSubmit: (values: CapitalCallFormValues) => void;
}) {
  return (
    <Card>
      <CardHeader>
        <h2 className="text-lg font-semibold">Add capital call</h2>
      </CardHeader>
      <CardContent>
        <form className="space-y-3" onSubmit={form.handleSubmit(onSubmit)}>
          <TextField label="Notice date" type="date" registration={form.register("noticeDate")} error={form.formState.errors.noticeDate?.message} />
          <TextField label="Due date" type="date" registration={form.register("dueDate")} error={form.formState.errors.dueDate?.message} />
          <TextField label="Call amount" registration={form.register("amount")} error={form.formState.errors.amount?.message} />
          <TextField label="Currency" registration={form.register("currency")} error={form.formState.errors.currency?.message} />
          <label className="grid gap-1 text-sm">
            Status
            <select className="border-input bg-background rounded-md border px-3 py-2" {...form.register("status")}>
              {(["expected", "due", "paid", "cancelled"] satisfies CapitalCallStatus[]).map((status) => (
                <option key={status} value={status}>
                  {status}
                </option>
              ))}
            </select>
          </label>
          <TextField label="Notes" registration={form.register("notes")} />
          <Button type="submit" disabled={busy}>
            {busy ? <Icons.Spinner className="size-4 animate-spin" aria-hidden="true" /> : <Icons.Plus className="size-4" aria-hidden="true" />}
            Save call
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

function ValuationForm({
  form,
  busy,
  onSubmit,
}: {
  form: UseFormReturn<ValuationFormValues>;
  busy: boolean;
  onSubmit: (values: ValuationFormValues) => void;
}) {
  return (
    <Card>
      <CardHeader>
        <h2 className="text-lg font-semibold">Update NAV</h2>
      </CardHeader>
      <CardContent>
        <form className="space-y-3" onSubmit={form.handleSubmit(onSubmit)}>
          <TextField label="Valuation date" type="date" registration={form.register("valuationDate")} error={form.formState.errors.valuationDate?.message} />
          <TextField label="NAV" registration={form.register("nav")} error={form.formState.errors.nav?.message} />
          <TextField label="Currency" registration={form.register("currency")} error={form.formState.errors.currency?.message} />
          <Button type="submit" disabled={busy}>
            {busy ? <Icons.Spinner className="size-4 animate-spin" aria-hidden="true" /> : <Icons.Save className="size-4" aria-hidden="true" />}
            Save NAV
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

function DistributionForm({
  form,
  busy,
  onSubmit,
}: {
  form: UseFormReturn<DistributionFormValues>;
  busy: boolean;
  onSubmit: (values: DistributionFormValues) => void;
}) {
  return (
    <Card>
      <CardHeader>
        <h2 className="text-lg font-semibold">Add distribution</h2>
      </CardHeader>
      <CardContent>
        <form className="space-y-3" onSubmit={form.handleSubmit(onSubmit)}>
          <TextField label="Distribution date" type="date" registration={form.register("distributionDate")} error={form.formState.errors.distributionDate?.message} />
          <TextField label="Distribution amount" registration={form.register("amount")} error={form.formState.errors.amount?.message} />
          <TextField label="Currency" registration={form.register("currency")} error={form.formState.errors.currency?.message} />
          <label className="flex items-center gap-2 text-sm">
            <input type="checkbox" {...form.register("recallable")} />
            Recallable
          </label>
          <TextField label="Notes" registration={form.register("notes")} />
          <Button type="submit" disabled={busy}>
            {busy ? <Icons.Spinner className="size-4 animate-spin" aria-hidden="true" /> : <Icons.Plus className="size-4" aria-hidden="true" />}
            Save distribution
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}

function TextField({
  label,
  type = "text",
  registration,
  error,
}: {
  label: string;
  type?: string;
  registration: UseFormRegisterReturn;
  error?: string;
}) {
  return (
    <Label className="grid gap-1 text-sm">
      {label}
      <Input type={type} {...registration} />
      {error ? <span className="text-xs text-destructive">{error}</span> : null}
    </Label>
  );
}

function CallRow({
  call,
  busy,
  onMarkPaid,
}: {
  call: CapitalCall;
  busy: boolean;
  onMarkPaid: () => void;
}) {
  return (
    <div className="rounded-md border p-3 text-sm">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <div className="font-medium">{formatMoney(call.amount, call.currency)}</div>
          <div className="text-muted-foreground">Due {call.dueDate}</div>
          {call.sourceCitationId ? (
            <div className="text-muted-foreground">Source citation: {call.sourceCitationId}</div>
          ) : null}
        </div>
        {call.status !== "paid" ? (
          <Button size="sm" variant="outline" disabled={busy} onClick={onMarkPaid}>
            {busy ? <Icons.Spinner className="size-4 animate-spin" aria-hidden="true" /> : <Icons.Check className="size-4" aria-hidden="true" />}
            Mark paid
          </Button>
        ) : null}
      </div>
    </div>
  );
}

function EventsCard({ title, rows }: { title: string; rows: string[] }) {
  return (
    <Card>
      <CardHeader>
        <h2 className="text-lg font-semibold">{title}</h2>
      </CardHeader>
      <CardContent>
        {rows.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {title === "Linked documents" ? "No source citations linked yet." : "No records yet."}
          </p>
        ) : (
          <ul className="space-y-2 text-sm">
            {rows.map((row) => (
              <li key={row} className="rounded-md border px-3 py-2">
                {row}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}

function formatCapitalCall(call: CapitalCall): string {
  return `${call.dueDate}: ${formatMoney(call.amount, call.currency)} (${call.status})${call.sourceCitationId ? ` - citation ${call.sourceCitationId}` : ""}`;
}

function formatDistribution(distribution: { distributionDate: string; amount: string; currency: string; recallable: boolean; sourceCitationId?: string | null }): string {
  return `${distribution.distributionDate}: ${formatMoney(distribution.amount, distribution.currency)}${distribution.recallable ? " recallable" : ""}${distribution.sourceCitationId ? ` - citation ${distribution.sourceCitationId}` : ""}`;
}

function formatCitation(citationId: string): string {
  return `Source citation: ${citationId}`;
}

function formatMoney(value: string, currency: string): string {
  const numeric = decimalToNumber(value);
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency,
    maximumFractionDigits: 2,
  }).format(numeric);
}

function formatRatio(value?: string | null): string {
  if (!value) return "Not available";
  return `${decimalToNumber(value).toFixed(2)}x`;
}

function decimalToNumber(value: string): number {
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : 0;
}
