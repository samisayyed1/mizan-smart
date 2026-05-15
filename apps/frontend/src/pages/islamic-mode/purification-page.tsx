import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";

import {
  getPurificationPeriodSummary,
  markPurificationPaid,
  upsertPurificationEntry,
  type PurificationEntry,
} from "@/adapters";
import { useSettingsContext } from "@/lib/settings-provider";
import { Page, PageContent, PageHeader } from "@mizan/ui";
import { Button } from "@mizan/ui/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { Input } from "@mizan/ui/components/ui/input";
import { Label } from "@mizan/ui/components/ui/label";
import { Textarea } from "@mizan/ui/components/ui/textarea";

const optionalDecimal = z
  .string()
  .trim()
  .refine((value) => value.length === 0 || /^(\d+)(\.\d+)?$/.test(value), {
    message: "Use a positive decimal amount",
  });

const purificationSchema = z.object({
  assetId: z.string().trim().min(1, "Asset ID is required"),
  periodStart: z.string().trim().min(1, "Period start is required"),
  periodEnd: z.string().trim().min(1, "Period end is required"),
  totalImpureIncome: optionalDecimal,
  outstandingShares: optionalDecimal,
  userShares: optionalDecimal,
  dividendReceived: optionalDecimal,
  impureIncomeRatio: optionalDecimal,
  sourceCitationId: z.string().trim(),
  notes: z.string().trim(),
});

type PurificationFormValues = z.infer<typeof purificationSchema>;

export default function PurificationPage() {
  const { settings } = useSettingsContext();
  const enabled = settings?.shariahModeEnabled === true;
  const queryClient = useQueryClient();
  const form = useForm<PurificationFormValues>({
    resolver: zodResolver(purificationSchema),
    defaultValues: {
      assetId: "",
      periodStart: "2026-01-01",
      periodEnd: "2026-12-31",
      totalImpureIncome: "",
      outstandingShares: "",
      userShares: "",
      dividendReceived: "",
      impureIncomeRatio: "",
      sourceCitationId: "",
      notes: "",
    },
  });
  const periodStart = form.watch("periodStart");
  const periodEnd = form.watch("periodEnd");

  const summaryQuery = useQuery({
    queryKey: ["purification-summary", periodStart, periodEnd],
    queryFn: () => getPurificationPeriodSummary(periodStart, periodEnd),
    enabled,
  });

  const upsertMutation = useMutation({
    mutationFn: (values: PurificationFormValues) =>
      upsertPurificationEntry({
        assetId: values.assetId,
        periodStart: values.periodStart,
        periodEnd: values.periodEnd,
        totalImpureIncome: emptyToNull(values.totalImpureIncome),
        outstandingShares: emptyToNull(values.outstandingShares),
        userShares: emptyToNull(values.userShares),
        dividendReceived: emptyToNull(values.dividendReceived),
        impureIncomeRatio: emptyToNull(values.impureIncomeRatio),
        status: null,
        sourceCitationId: emptyToNull(values.sourceCitationId),
        notes: emptyToNull(values.notes),
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["purification-summary"] });
    },
  });

  const paidMutation = useMutation({
    mutationFn: (entryId: string) => markPurificationPaid(entryId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["purification-summary"] });
    },
  });

  if (!enabled) {
    return (
      <Page>
        <PageHeader heading="Purification" text="Enable optional Islamic finance tools in Settings." />
        <PageContent>
          <Card>
            <CardContent className="text-muted-foreground p-6 text-sm">
              Islamic finance tools are disabled for this profile.
            </CardContent>
          </Card>
        </PageContent>
      </Page>
    );
  }

  return (
    <Page>
      <PageHeader heading="Purification" text="Track optional dividend purification reviews." />
      <PageContent className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Add Review Entry</CardTitle>
          </CardHeader>
          <CardContent>
            <form
              className="grid gap-4"
              onSubmit={form.handleSubmit((values) => upsertMutation.mutate(values))}
            >
              <div className="grid gap-3 md:grid-cols-3">
                <Field label="Asset ID" error={form.formState.errors.assetId?.message}>
                  <Input {...form.register("assetId")} placeholder="asset id" />
                </Field>
                <Field label="Period start" error={form.formState.errors.periodStart?.message}>
                  <Input {...form.register("periodStart")} type="date" />
                </Field>
                <Field label="Period end" error={form.formState.errors.periodEnd?.message}>
                  <Input {...form.register("periodEnd")} type="date" />
                </Field>
              </div>
              <div className="grid gap-3 md:grid-cols-3">
                <Field label="Total impure income" error={form.formState.errors.totalImpureIncome?.message}>
                  <Input
                    {...form.register("totalImpureIncome")}
                    inputMode="decimal"
                    placeholder="total impure income"
                  />
                </Field>
                <Field label="Outstanding shares" error={form.formState.errors.outstandingShares?.message}>
                  <Input
                    {...form.register("outstandingShares")}
                    inputMode="decimal"
                    placeholder="outstanding shares"
                  />
                </Field>
                <Field label="User shares" error={form.formState.errors.userShares?.message}>
                  <Input {...form.register("userShares")} inputMode="decimal" placeholder="user shares" />
                </Field>
              </div>
              <div className="grid gap-3 md:grid-cols-2">
                <Field label="Dividend received" error={form.formState.errors.dividendReceived?.message}>
                  <Input {...form.register("dividendReceived")} inputMode="decimal" />
                </Field>
                <Field label="Impure income ratio" error={form.formState.errors.impureIncomeRatio?.message}>
                  <Input {...form.register("impureIncomeRatio")} inputMode="decimal" placeholder="0.05" />
                </Field>
              </div>
              <Field label="Source citation ID" error={form.formState.errors.sourceCitationId?.message}>
                <Input {...form.register("sourceCitationId")} placeholder="optional citation id" />
              </Field>
              <Field label="Notes" error={form.formState.errors.notes?.message}>
                <Textarea {...form.register("notes")} placeholder="review notes" />
              </Field>
              <p className="text-muted-foreground text-xs">
                Leave missing inputs blank. Mizan will mark the entry as needs review instead of
                inventing ratios.
              </p>
              {upsertMutation.error ? (
                <p className="text-destructive text-sm">{upsertMutation.error.message}</p>
              ) : null}
              <Button type="submit" disabled={upsertMutation.isPending}>
                Save Entry
              </Button>
            </form>
          </CardContent>
        </Card>
        <PurificationTable
          entries={summaryQuery.data?.entries ?? []}
          totalCalculated={summaryQuery.data?.totalCalculated ?? "0"}
          totalPaid={summaryQuery.data?.totalPaid ?? "0"}
          onMarkPaid={(entryId) => paidMutation.mutate(entryId)}
        />
      </PageContent>
    </Page>
  );
}

function PurificationTable({
  entries,
  totalCalculated,
  totalPaid,
  onMarkPaid,
}: {
  entries: PurificationEntry[];
  totalCalculated: string;
  totalPaid: string;
  onMarkPaid: (entryId: string) => void;
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between gap-3">
        <CardTitle className="text-lg">Purification Summary</CardTitle>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => exportSummary({ entries, totalCalculated, totalPaid })}
        >
          Export summary
        </Button>
      </CardHeader>
      <CardContent className="space-y-4 text-sm">
        <div className="grid gap-2 sm:grid-cols-2">
          <Metric label="Total calculated" value={totalCalculated} />
          <Metric label="Total paid" value={totalPaid} />
        </div>
        {entries.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead className="text-muted-foreground">
                <tr>
                  <th className="py-2 pr-3">Asset</th>
                  <th className="py-2 pr-3">Method</th>
                  <th className="py-2 pr-3">Amount</th>
                  <th className="py-2 pr-3">Status</th>
                  <th className="py-2 pr-3">Action</th>
                </tr>
              </thead>
              <tbody>
                {entries.map((entry) => (
                  <tr key={entry.id} className="border-t">
                    <td className="py-2 pr-3">{entry.assetId}</td>
                    <td className="py-2 pr-3">{formatMethod(entry.calculationMethod)}</td>
                    <td className="py-2 pr-3">{entry.purificationAmount}</td>
                    <td className="py-2 pr-3">{entry.status}</td>
                    <td className="py-2 pr-3">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={entry.status === "paid"}
                        onClick={() => onMarkPaid(entry.id)}
                      >
                        Mark paid
                      </Button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-muted-foreground">No purification entries for this period.</p>
        )}
      </CardContent>
    </Card>
  );
}

function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <Label>{label}</Label>
      {children}
      {error ? <div className="text-destructive text-xs">{error}</div> : null}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border p-3">
      <div className="text-muted-foreground text-xs">{label}</div>
      <div className="text-base font-medium">{value}</div>
    </div>
  );
}

function emptyToNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function formatMethod(method: PurificationEntry["calculationMethod"]): string {
  return method.replace(/_/g, " ");
}

function exportSummary({
  entries,
  totalCalculated,
  totalPaid,
}: {
  entries: PurificationEntry[];
  totalCalculated: string;
  totalPaid: string;
}) {
  const blob = new Blob([JSON.stringify({ totalCalculated, totalPaid, entries }, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "purification-summary.json";
  anchor.click();
  URL.revokeObjectURL(url);
}
