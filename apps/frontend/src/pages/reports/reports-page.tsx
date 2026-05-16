import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";

import {
  addManualFeeEntry,
  exportReport,
  generateReport,
  type EstateBinderSection,
  type FeeCategory,
  type ReportRun,
  type ReportType,
} from "@/adapters";
import { Button, Icons, Page, PageContent, PageHeader } from "@mizan/ui";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { Input } from "@mizan/ui/components/ui/input";
import { Label } from "@mizan/ui/components/ui/label";

interface ReportOption {
  type: ReportType;
  title: string;
  description: string;
  icon: keyof typeof Icons;
}

const REPORT_OPTIONS: ReportOption[] = [
  {
    type: "fee_report",
    title: "Fee Report",
    description: "Fee totals, source rows, and deterministic spike warnings.",
    icon: "HandCoins",
  },
  {
    type: "estate_binder",
    title: "Estate Binder",
    description: "Organized legacy binder with selected sections and no legal advice.",
    icon: "FileText",
  },
  {
    type: "monthly_wealth_letter",
    title: "Monthly Wealth Letter",
    description: "Premium deterministic monthly summary without AI commentary.",
    icon: "FileText",
  },
  {
    type: "net_worth",
    title: "Net Worth Report",
    description: "Deterministic net worth lines when source rows are available.",
    icon: "TrendingUp",
  },
  {
    type: "portfolio_summary",
    title: "Portfolio Summary",
    description: "Portfolio summary foundation with honest empty-state handling.",
    icon: "PieChart",
  },
  {
    type: "income",
    title: "Income Report",
    description: "Income report foundation for deterministic future sections.",
    icon: "HandCoins",
  },
  {
    type: "data_quality",
    title: "Data Quality Report",
    description: "Data quality report foundation without invented rows.",
    icon: "ShieldCheck",
  },
  {
    type: "tax_pack",
    title: "Tax Pack Report",
    description: "Preview latest tax pack lines with source citation status.",
    icon: "ShieldCheck",
  },
];

const ESTATE_BINDER_OPTIONS: { section: EstateBinderSection; label: string }[] = [
  { section: "accounts", label: "Accounts" },
  { section: "assets", label: "Assets" },
  { section: "liabilities", label: "Liabilities" },
  { section: "property", label: "Property" },
  { section: "insurance", label: "Insurance / ULIP" },
  { section: "pensions", label: "Pensions" },
  { section: "private_investments", label: "Private investments" },
  { section: "documents_manifest", label: "Documents manifest" },
  { section: "entity_ownership", label: "Entity ownership summary" },
  { section: "islamic_notes", label: "Zakat / waqf / charity notes" },
];

const FEE_CATEGORY_OPTIONS: { category: FeeCategory; label: string }[] = [
  { category: "broker_fees", label: "Broker fees" },
  { category: "transaction_fees", label: "Transaction fees" },
  { category: "platform_fees", label: "Platform fees" },
  { category: "advisory_fees", label: "Advisory fees" },
  { category: "fund_expense_ratio_manual", label: "Fund expense ratio manual" },
  { category: "insurance_ulip_charges", label: "Insurance / ULIP charges" },
  { category: "fx_fees", label: "FX fees" },
  { category: "private_fund_fees", label: "Private fund fees" },
  { category: "custody_admin_fees", label: "Custody / admin fees" },
  { category: "other", label: "Other" },
];

export default function ReportsPage() {
  const [reportType, setReportType] = useState<ReportType>("tax_pack");
  const [baseCurrency, setBaseCurrency] = useState("USD");
  const [periodMonth, setPeriodMonth] = useState(() => new Date().toISOString().slice(0, 7));
  const [manualFeeDate, setManualFeeDate] = useState(() => new Date().toISOString().slice(0, 10));
  const [manualFeeCategory, setManualFeeCategory] = useState<FeeCategory>("transaction_fees");
  const [manualFeeAmount, setManualFeeAmount] = useState("");
  const [manualFeeNotes, setManualFeeNotes] = useState("");
  const [estateSections, setEstateSections] = useState<EstateBinderSection[]>([
    "accounts",
    "assets",
    "liabilities",
    "property",
    "insurance",
    "pensions",
    "private_investments",
    "documents_manifest",
  ]);

  const generateMutation = useMutation({
    mutationFn: () => {
      const request = {
        reportType,
        baseCurrency: baseCurrency.toUpperCase(),
      };
      if (reportType === "monthly_wealth_letter" || reportType === "fee_report") {
        return generateReport({ ...request, periodMonth });
      }
      if (reportType === "estate_binder") {
        return generateReport({ ...request, includedSections: estateSections });
      }
      return generateReport(request);
    },
  });

  const exportMutation = useMutation({
    mutationFn: (reportRunId: string) => exportReport(reportRunId),
    onSuccess: (bundle) => {
      downloadBundle(bundle.fileName, bundle.mimeType, bundle.bytes);
    },
  });

  const manualFeeMutation = useMutation({
    mutationFn: () =>
      addManualFeeEntry({
        feeDate: manualFeeDate,
        category: manualFeeCategory,
        amount: manualFeeAmount,
        currency: baseCurrency.toUpperCase(),
        notes: manualFeeNotes.trim() || null,
      }),
    onSuccess: () => {
      setManualFeeAmount("");
      setManualFeeNotes("");
    },
  });

  const selected = REPORT_OPTIONS.find((option) => option.type === reportType) ?? REPORT_OPTIONS[0];

  return (
    <Page>
      <PageHeader
        heading="Reports"
        text="Build deterministic report previews from local source data."
      />
      <PageContent>
        {/* Tax Pack has its own dedicated builder for jurisdiction-specific
            exports and the full CPA bundle (Phase 4 P28/P29). Surface the
            link here so the standalone /reports/tax-pack page isn't orphaned. */}
        <div className="mb-4 flex flex-wrap items-center gap-3 rounded-lg border bg-muted/20 px-4 py-3 text-sm">
          <Icons.ShieldCheck className="text-muted-foreground size-4" aria-hidden="true" />
          <span className="text-muted-foreground flex-1">
            Need year + jurisdiction + CPA export bundle?
          </span>
          <Button asChild variant="secondary" size="sm">
            <Link to="/reports/tax-pack">Open Tax Pack builder</Link>
          </Button>
        </div>
        <div className="grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Report Builder</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="report-type">Report type</Label>
                <select
                  id="report-type"
                  className="border-input bg-background h-10 w-full rounded-md border px-3 text-sm"
                  value={reportType}
                  onChange={(event) => setReportType(event.target.value as ReportType)}
                >
                  {REPORT_OPTIONS.map((option) => (
                    <option key={option.type} value={option.type}>
                      {option.title}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="base-currency">Base currency</Label>
                <Input
                  id="base-currency"
                  value={baseCurrency}
                  maxLength={3}
                  onChange={(event) => setBaseCurrency(event.target.value.toUpperCase())}
                />
              </div>
              {reportType === "monthly_wealth_letter" || reportType === "fee_report" ? (
                <div className="space-y-2">
                  <Label htmlFor="period-month">Month</Label>
                  <Input
                    id="period-month"
                    type="month"
                    value={periodMonth}
                    onChange={(event) => setPeriodMonth(event.target.value)}
                  />
                </div>
              ) : null}
              {reportType === "fee_report" ? (
                <div className="space-y-3 rounded-md border p-3">
                  <p className="text-sm font-medium">Manual fee entry</p>
                  <div className="grid gap-2">
                    <Label htmlFor="manual-fee-date">Date</Label>
                    <Input
                      id="manual-fee-date"
                      type="date"
                      value={manualFeeDate}
                      onChange={(event) => setManualFeeDate(event.target.value)}
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="manual-fee-category">Category</Label>
                    <select
                      id="manual-fee-category"
                      className="border-input bg-background h-10 w-full rounded-md border px-3 text-sm"
                      value={manualFeeCategory}
                      onChange={(event) => setManualFeeCategory(event.target.value as FeeCategory)}
                    >
                      {FEE_CATEGORY_OPTIONS.map((option) => (
                        <option key={option.category} value={option.category}>
                          {option.label}
                        </option>
                      ))}
                    </select>
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="manual-fee-amount">Amount</Label>
                    <Input
                      id="manual-fee-amount"
                      inputMode="decimal"
                      value={manualFeeAmount}
                      onChange={(event) => setManualFeeAmount(event.target.value)}
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="manual-fee-notes">Notes</Label>
                    <Input
                      id="manual-fee-notes"
                      value={manualFeeNotes}
                      onChange={(event) => setManualFeeNotes(event.target.value)}
                    />
                  </div>
                  <Button
                    type="button"
                    variant="outline"
                    className="w-full"
                    onClick={() => manualFeeMutation.mutate()}
                    disabled={
                      manualFeeMutation.isPending ||
                      manualFeeDate.length !== 10 ||
                      manualFeeAmount.trim().length === 0 ||
                      baseCurrency.length !== 3
                    }
                  >
                    Save Fee
                  </Button>
                  {manualFeeMutation.error ? (
                    <p className="text-destructive text-sm">{manualFeeMutation.error.message}</p>
                  ) : null}
                </div>
              ) : null}
              {reportType === "estate_binder" ? (
                <div className="space-y-2">
                  <p className="text-sm font-medium">Included sections</p>
                  <div className="grid gap-2">
                    {ESTATE_BINDER_OPTIONS.map((option) => (
                      <label key={option.section} className="flex items-center gap-2 text-sm">
                        <input
                          type="checkbox"
                          checked={estateSections.includes(option.section)}
                          onChange={(event) =>
                            setEstateSections((current) =>
                              event.target.checked
                                ? [...current, option.section]
                                : current.filter((section) => section !== option.section),
                            )
                          }
                        />
                        <span>{option.label}</span>
                      </label>
                    ))}
                  </div>
                </div>
              ) : null}
              <Button
                type="button"
                className="w-full"
                onClick={() => generateMutation.mutate()}
                disabled={
                  generateMutation.isPending ||
                  baseCurrency.length !== 3 ||
                  ((reportType === "monthly_wealth_letter" || reportType === "fee_report") &&
                    periodMonth.length !== 7) ||
                  (reportType === "estate_binder" && estateSections.length === 0)
                }
              >
                Generate Preview
              </Button>
              {generateMutation.error ? (
                <p className="text-destructive text-sm">{generateMutation.error.message}</p>
              ) : null}
            </CardContent>
          </Card>

          <div className="space-y-4">
            <ReportOptionSummary option={selected} />
            {generateMutation.data ? (
              <ReportPreview
                report={generateMutation.data}
                isExporting={exportMutation.isPending}
                onExport={() => exportMutation.mutate(generateMutation.data.id)}
              />
            ) : (
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Preview</CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-muted-foreground text-sm">
                    Generate a report to preview deterministic sections and citation status.
                  </p>
                </CardContent>
              </Card>
            )}
            {exportMutation.error ? (
              <p className="text-destructive text-sm">{exportMutation.error.message}</p>
            ) : null}
          </div>
        </div>
      </PageContent>
    </Page>
  );
}

function ReportOptionSummary({ option }: { option: ReportOption }) {
  const Icon = Icons[option.icon];
  return (
    <Card>
      <CardContent className="flex items-start gap-3 p-4">
        <Icon className="text-foreground/80 mt-0.5 size-5 shrink-0" aria-hidden="true" />
        <div>
          <p className="font-medium">{option.title}</p>
          <p className="text-muted-foreground text-sm">{option.description}</p>
        </div>
      </CardContent>
    </Card>
  );
}

function ReportPreview({
  report,
  isExporting,
  onExport,
}: {
  report: ReportRun;
  isExporting: boolean;
  onExport: () => void;
}) {
  return (
    <Card>
      <CardHeader>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <CardTitle className="text-lg">Preview</CardTitle>
          <Button type="button" variant="outline" onClick={onExport} disabled={isExporting}>
            Export HTML
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-muted-foreground text-sm">{report.disclaimer}</p>
        <div className="grid gap-2 sm:grid-cols-3">
          <Metric label="Status" value={report.status} />
          <Metric label="Sections" value={String(report.sections.length)} />
          <Metric label="Base" value={report.baseCurrency} />
        </div>
        {report.sections.map((section) => (
          <div key={section.id} className="rounded-md border">
            <div className="border-b px-3 py-2 text-sm font-medium">{section.title}</div>
            <div className="divide-y">
              {section.lines.map((line) => (
                <div
                  key={line.id}
                  className="grid gap-2 px-3 py-2 text-sm md:grid-cols-[minmax(0,1fr)_140px_160px]"
                >
                  <span>{line.label}</span>
                  <span>
                    {line.amount ?? line.valueText}
                    {line.currency ? ` ${line.currency}` : ""}
                  </span>
                  <span className="text-muted-foreground">
                    {line.sourceCitationId ?? "Missing citation"}
                  </span>
                </div>
              ))}
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border px-3 py-2">
      <p className="text-muted-foreground text-xs uppercase">{label}</p>
      <p className="font-medium">{value}</p>
    </div>
  );
}

function downloadBundle(fileName: string, mimeType: string, bytes: number[]) {
  const blob = new Blob([new Uint8Array(bytes)], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  anchor.click();
  URL.revokeObjectURL(url);
}
