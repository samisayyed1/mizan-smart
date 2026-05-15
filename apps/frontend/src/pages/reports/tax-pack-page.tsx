import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";

import { generateTaxPack, type TaxJurisdiction, type TaxPack, type TaxPackLine } from "@/adapters";
import { Button, Page, PageContent, PageHeader } from "@mizan/ui";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { Input } from "@mizan/ui/components/ui/input";
import { Label } from "@mizan/ui/components/ui/label";

const jurisdictions = ["US", "UK", "Singapore", "GCC", "General"] as const;

const taxPackSchema = z.object({
  taxYear: z.coerce.number().int().min(1900).max(9999),
  jurisdiction: z.enum(jurisdictions),
  baseCurrency: z
    .string()
    .trim()
    .length(3, "Use a three-letter currency code")
    .transform((value) => value.toUpperCase()),
});

type TaxPackFormValues = z.input<typeof taxPackSchema>;

export default function TaxPackPage() {
  const form = useForm<TaxPackFormValues>({
    resolver: zodResolver(taxPackSchema),
    defaultValues: {
      taxYear: new Date().getFullYear() - 1,
      jurisdiction: "General",
      baseCurrency: "USD",
    },
  });
  const mutation = useMutation({
    mutationFn: (values: TaxPackFormValues) => {
      const parsed = taxPackSchema.parse(values);
      return generateTaxPack({
        taxYear: parsed.taxYear,
        jurisdiction: parsed.jurisdiction,
        baseCurrency: parsed.baseCurrency,
      });
    },
  });

  return (
    <Page>
      <PageHeader heading="Tax Pack" text="CPA-ready data preparation, not tax advice." />
      <PageContent className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Generate Draft</CardTitle>
          </CardHeader>
          <CardContent>
            <form
              className="grid gap-4"
              onSubmit={form.handleSubmit((values) => mutation.mutate(values))}
            >
              <div className="grid gap-3 md:grid-cols-3">
                <Field label="Tax year" htmlFor="tax-year" error={form.formState.errors.taxYear?.message}>
                  <Input id="tax-year" {...form.register("taxYear")} inputMode="numeric" />
                </Field>
                <Field
                  label="Jurisdiction"
                  htmlFor="tax-jurisdiction"
                  error={form.formState.errors.jurisdiction?.message}
                >
                  <select
                    id="tax-jurisdiction"
                    {...form.register("jurisdiction")}
                    className="border-input bg-background h-10 w-full rounded-md border px-3 text-sm"
                  >
                    {jurisdictions.map((jurisdiction) => (
                      <option key={jurisdiction} value={jurisdiction}>
                        {jurisdiction}
                      </option>
                    ))}
                  </select>
                </Field>
                <Field
                  label="Base currency"
                  htmlFor="base-currency"
                  error={form.formState.errors.baseCurrency?.message}
                >
                  <Input id="base-currency" {...form.register("baseCurrency")} maxLength={3} />
                </Field>
              </div>
              <p className="text-muted-foreground text-sm">
                Mizan prepares ledger-backed summaries only. It does not infer filing treatment or
                jurisdiction-specific tax classifications.
              </p>
              {mutation.error ? (
                <p className="text-destructive text-sm">{mutation.error.message}</p>
              ) : null}
              <Button type="submit" disabled={mutation.isPending}>
                Generate Tax Pack
              </Button>
            </form>
          </CardContent>
        </Card>

        {mutation.data ? <TaxPackResult pack={mutation.data} /> : null}
      </PageContent>
    </Page>
  );
}

function TaxPackResult({ pack }: { pack: TaxPack }) {
  const totals = pack.lines.reduce<Record<string, string>>((acc, line) => {
    const key = `${formatCategory(line.category)} ${line.currency}`;
    const next = Number(acc[key] ?? 0) + Number(line.amount);
    acc[key] = next.toString();
    return acc;
  }, {});

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Draft Summary</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 text-sm">
          <p className="text-muted-foreground">{pack.disclaimer}</p>
          <div className="grid gap-2 sm:grid-cols-3">
            <Metric label="Status" value={pack.status} />
            <Metric label="Lines" value={String(pack.lines.length)} />
            <Metric label="Checklist" value={String(pack.missingDataChecklist.length)} />
          </div>
          {Object.keys(totals).length > 0 ? (
            <div className="grid gap-2 sm:grid-cols-2">
              {Object.entries(totals).map(([label, value]) => (
                <Metric key={label} label={label} value={value} />
              ))}
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Missing Data Checklist</CardTitle>
        </CardHeader>
        <CardContent>
          {pack.missingDataChecklist.length > 0 ? (
            <ul className="space-y-2 text-sm">
              {pack.missingDataChecklist.map((item) => (
                <li key={item.id} className="rounded-md border p-3">
                  <span className="font-medium capitalize">{item.severity}</span>
                  <span className="text-muted-foreground ml-2">{item.message}</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-muted-foreground text-sm">No checklist items for this draft.</p>
          )}
        </CardContent>
      </Card>

      <TaxPackLines lines={pack.lines} />
    </div>
  );
}

function TaxPackLines({ lines }: { lines: TaxPackLine[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Prepared Lines</CardTitle>
      </CardHeader>
      <CardContent>
        {lines.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead className="text-muted-foreground">
                <tr>
                  <th className="py-2 pr-3">Date</th>
                  <th className="py-2 pr-3">Category</th>
                  <th className="py-2 pr-3">Asset</th>
                  <th className="py-2 pr-3">Amount</th>
                  <th className="py-2 pr-3">Source</th>
                </tr>
              </thead>
              <tbody>
                {lines.map((line) => (
                  <tr key={line.id} className="border-t">
                    <td className="py-2 pr-3">{line.taxableDate}</td>
                    <td className="py-2 pr-3">{formatCategory(line.category)}</td>
                    <td className="py-2 pr-3">{line.assetId ?? "Cash"}</td>
                    <td className="py-2 pr-3">
                      {line.amount} {line.currency}
                    </td>
                    <td className="py-2 pr-3">{line.sourceCitationId ?? line.activityId ?? "Manual review"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-muted-foreground text-sm">No tax pack lines were generated.</p>
        )}
      </CardContent>
    </Card>
  );
}

function Field({
  label,
  htmlFor,
  error,
  children,
}: {
  label: string;
  htmlFor: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <Label htmlFor={htmlFor}>{label}</Label>
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

function formatCategory(category: TaxPackLine["category"]): string {
  return category.replace(/_/g, " ");
}

export type { TaxJurisdiction };
