import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import type { ReactNode } from "react";
import { useForm, type UseFormReturn } from "react-hook-form";
import { z } from "zod";

import { calculateZakatSnapshot, type ZakatSnapshot } from "@/adapters";
import { useSettingsContext } from "@/lib/settings-provider";
import { Page, PageContent, PageHeader } from "@mizan/ui";
import { Button } from "@mizan/ui/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { Input } from "@mizan/ui/components/ui/input";
import { Label } from "@mizan/ui/components/ui/label";
import { Textarea } from "@mizan/ui/components/ui/textarea";

const decimalAmount = z
  .string()
  .trim()
  .regex(/^(\d+)(\.\d+)?$/, "Use a positive decimal amount");

const optionalDecimalAmount = z
  .string()
  .trim()
  .refine((value) => value.length === 0 || /^(\d+)(\.\d+)?$/.test(value), {
    message: "Use a positive decimal amount",
  });

const zakatSchema = z
  .object({
    snapshotDate: z.string().trim().min(1, "Snapshot date is required"),
    baseCurrency: z.string().trim().length(3, "Use a 3-letter currency code"),
    nisabValue: decimalAmount,
    notes: z.string().trim(),
    assetId: z.string().trim(),
    category: z.string().trim().min(1, "Category is required"),
    amount: optionalDecimalAmount,
    included: z.boolean(),
    explanation: z.string().trim(),
    sourceCitationId: z.string().trim(),
  })
  .refine((value) => value.assetId.length > 0 || value.amount.length > 0, {
    path: ["amount"],
    message: "Enter an amount or an asset ID with a stored valuation",
  });

type ZakatFormValues = z.infer<typeof zakatSchema>;
type WizardStep = "lines" | "nisab" | "summary";

export default function ZakatCalculatorPage() {
  const { settings } = useSettingsContext();
  const enabled = settings?.shariahModeEnabled === true;
  const [step, setStep] = useState<WizardStep>("lines");
  const [snapshot, setSnapshot] = useState<ZakatSnapshot | null>(null);

  const form = useForm<ZakatFormValues>({
    resolver: zodResolver(zakatSchema),
    defaultValues: {
      snapshotDate: new Date().toISOString().slice(0, 10),
      baseCurrency: settings?.baseCurrency ?? "USD",
      nisabValue: "",
      notes: "",
      assetId: "",
      category: "short_term_asset",
      amount: "",
      included: true,
      explanation: "",
      sourceCitationId: "",
    },
  });

  const mutation = useMutation({
    mutationFn: (values: ZakatFormValues) =>
      calculateZakatSnapshot({
        snapshotDate: values.snapshotDate,
        baseCurrency: values.baseCurrency.trim().toUpperCase(),
        nisabValue: values.nisabValue,
        notes: emptyToNull(values.notes),
        lines: [
          {
            assetId: emptyToNull(values.assetId),
            category: values.category,
            amount: emptyToNull(values.amount),
            included: values.included,
            explanation: emptyToNull(values.explanation),
            sourceCitationId: emptyToNull(values.sourceCitationId),
          },
        ],
      }),
    onSuccess: (nextSnapshot) => {
      setSnapshot(nextSnapshot);
      setStep("summary");
    },
  });

  if (!enabled) {
    return (
      <Page>
        <PageHeader heading="Zakat" text="Enable optional Islamic finance tools in Settings." />
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
      <PageHeader heading="Zakat" text="Prepare a user-reviewed Zakat snapshot." />
      <PageContent className="space-y-4">
        <div className="flex flex-wrap gap-2">
          <StepButton active={step === "lines"} onClick={() => setStep("lines")}>
            Lines
          </StepButton>
          <StepButton active={step === "nisab"} onClick={() => setStep("nisab")}>
            Nisab
          </StepButton>
          <StepButton active={step === "summary"} onClick={() => setStep("summary")}>
            Summary
          </StepButton>
        </div>

        <form
          className="space-y-4"
          onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
        >
          {step === "lines" ? <LinesStep form={form} onNext={() => setStep("nisab")} /> : null}
          {step === "nisab" ? (
            <NisabStep form={form} isSaving={mutation.isPending} error={mutation.error?.message} />
          ) : null}
        </form>

        {step === "summary" ? <SummaryStep snapshot={snapshot} /> : null}
      </PageContent>
    </Page>
  );
}

function LinesStep({
  form,
  onNext,
}: {
  form: UseFormReturn<ZakatFormValues>;
  onNext: () => void;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Review Lines</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4">
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Asset ID" error={form.formState.errors.assetId?.message}>
            <Input {...form.register("assetId")} placeholder="asset id with stored valuation" />
          </Field>
          <Field label="Category" error={form.formState.errors.category?.message}>
            <select
              className="border-input bg-background h-10 rounded-md border px-3 text-sm"
              {...form.register("category")}
            >
              <option value="short_term_asset">Short-term asset</option>
              <option value="cash">Cash</option>
              <option value="investment">Investment</option>
              <option value="liability">Deductible liability</option>
              <option value="excluded">Excluded item</option>
            </select>
          </Field>
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Manual amount" error={form.formState.errors.amount?.message}>
            <Input {...form.register("amount")} inputMode="decimal" placeholder="leave blank for valuation" />
          </Field>
          <Field label="Source citation ID" error={form.formState.errors.sourceCitationId?.message}>
            <Input {...form.register("sourceCitationId")} placeholder="optional citation id" />
          </Field>
        </div>
        <Field label="Line explanation" error={form.formState.errors.explanation?.message}>
          <Input {...form.register("explanation")} placeholder="how this line was determined" />
        </Field>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" {...form.register("included")} />
          Included in Zakat calculation
        </label>
        <div>
          <Button type="button" onClick={onNext}>
            Continue
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function NisabStep({
  form,
  isSaving,
  error,
}: {
  form: UseFormReturn<ZakatFormValues>;
  isSaving: boolean;
  error?: string;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Manual Nisab</CardTitle>
      </CardHeader>
      <CardContent className="grid gap-4">
        <div className="grid gap-3 md:grid-cols-3">
          <Field label="Snapshot date" error={form.formState.errors.snapshotDate?.message}>
            <Input {...form.register("snapshotDate")} type="date" />
          </Field>
          <Field label="Base currency" error={form.formState.errors.baseCurrency?.message}>
            <Input {...form.register("baseCurrency")} maxLength={3} />
          </Field>
          <Field label="Manual nisab value" error={form.formState.errors.nisabValue?.message}>
            <Input {...form.register("nisabValue")} inputMode="decimal" placeholder="required" />
          </Field>
        </div>
        <Field label="Notes" error={form.formState.errors.notes?.message}>
          <Textarea {...form.register("notes")} placeholder="snapshot notes" />
        </Field>
        <p className="text-muted-foreground text-xs">
          Enter the nisab value you want to use. Mizan does not provide religious advice or a
          fabricated nisab.
        </p>
        {error ? <p className="text-destructive text-sm">{error}</p> : null}
        <div>
          <Button type="submit" disabled={isSaving}>
            Calculate Snapshot
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

function SummaryStep({ snapshot }: { snapshot: ZakatSnapshot | null }) {
  if (!snapshot) {
    return (
      <Card>
        <CardContent className="text-muted-foreground p-6 text-sm">
          No Zakat snapshot has been calculated yet.
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Final Summary</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4 text-sm">
        <div className="grid gap-2 sm:grid-cols-4">
          <Metric label="Assets" value={snapshot.totalZakatableAssets} />
          <Metric label="Liabilities" value={snapshot.deductibleLiabilities} />
          <Metric label="Net wealth" value={snapshot.netZakatableWealth} />
          <Metric label="Zakat due" value={snapshot.zakatDue} />
        </div>
        <div className="space-y-2">
          <div className="font-medium">Included and excluded lines</div>
          <ul className="space-y-2">
            {snapshot.lines.map((line) => (
              <li key={line.id} className="rounded-md border p-3">
                <div className="font-medium">
                  {line.category}: {line.amount} {snapshot.baseCurrency}
                </div>
                <div className="text-muted-foreground">
                  {line.included ? "Included" : "Excluded"} - {line.explanation}
                </div>
                {line.sourceCitationId ? (
                  <div className="text-muted-foreground">Citation: {line.sourceCitationId}</div>
                ) : null}
              </li>
            ))}
          </ul>
        </div>
        <p className="text-muted-foreground text-xs">
          This snapshot is a calculation aid. Review the lines and consult a qualified advisor for
          final religious or legal decisions.
        </p>
      </CardContent>
    </Card>
  );
}

function StepButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: string;
}) {
  return (
    <Button type="button" variant={active ? "default" : "outline"} onClick={onClick}>
      {children}
    </Button>
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
