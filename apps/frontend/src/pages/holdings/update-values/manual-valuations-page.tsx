import { useCallback, useEffect, useMemo, useState } from "react";
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
} from "@tanstack/react-table";

import {
  bulkUpdateValuations,
  getManualValuationHistory,
  listManualValuationAssets,
  type ManualValuationAsset,
  type ManualValuationHistoryRow,
  type ManualValuationStaleness,
  type ManualValuationUpdateRow,
  type RowValidationError,
} from "@/adapters";
import { Button, Icons, Input, Label, Page, PageContent, PageHeader } from "@mizan/ui";
import { Card, CardContent, CardHeader } from "@mizan/ui/components/ui/card";

import { manualValuationBatchSchema, todayIso } from "./manual-valuations-schema";

interface DraftRow extends ManualValuationUpdateRow {
  name: string;
  classification: string;
  staleness: ManualValuationStaleness;
  historyCount: number;
}

function toDraft(row: ManualValuationAsset): DraftRow {
  return {
    assetId: row.assetId,
    name: row.name,
    classification: row.classification,
    currentValue: row.currentValue ?? "",
    valuationDate: row.valuationDate ?? todayIso(),
    currency: row.currency,
    notes: row.notes ?? "",
    staleness: row.staleness,
    historyCount: row.historyCount,
  };
}

function errorKey(rowIndex: number, field: string): string {
  return `${rowIndex}:${field}`;
}

function buildErrorMap(errors: RowValidationError[]): Map<string, string> {
  return new Map(errors.map((error) => [errorKey(error.rowIndex, error.field), error.message]));
}

export default function ManualValuationsPage() {
  const [rows, setRows] = useState<DraftRow[]>([]);
  const [errors, setErrors] = useState<Map<string, string>>(new Map());
  const [history, setHistory] = useState<ManualValuationHistoryRow[]>([]);
  const [historyAssetName, setHistoryAssetName] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [status, setStatus] = useState<string | null>(null);

  const loadRows = useCallback(async () => {
    setIsLoading(true);
    const assets = await listManualValuationAssets();
    setRows(assets.map(toDraft));
    setIsLoading(false);
  }, []);

  useEffect(() => {
    void loadRows().catch((error: unknown) => {
      setStatus(error instanceof Error ? error.message : String(error));
      setIsLoading(false);
    });
  }, [loadRows]);

  const updateRow = useCallback((rowIndex: number, patch: Partial<ManualValuationUpdateRow>) => {
    setRows((current) =>
      current.map((row, index) => (index === rowIndex ? { ...row, ...patch } : row)),
    );
  }, []);

  const markUnchanged = useCallback((rowIndex: number) => {
    updateRow(rowIndex, { valuationDate: todayIso() });
  }, [updateRow]);

  const showHistory = useCallback(async (row: DraftRow) => {
    const loaded = await getManualValuationHistory(row.assetId);
    setHistory(loaded);
    setHistoryAssetName(row.name);
  }, []);

  async function saveAll() {
    setIsSaving(true);
    setStatus(null);
    const request = {
      rows: rows.map((row) => ({
        assetId: row.assetId,
        currentValue: row.currentValue.trim(),
        valuationDate: row.valuationDate,
        currency: row.currency.trim().toUpperCase(),
        notes: row.notes?.trim() ? row.notes.trim() : null,
      })),
    };
    const parsed = manualValuationBatchSchema.safeParse(request);
    if (!parsed.success) {
      const rowErrors: RowValidationError[] = parsed.error.issues.map((issue) => ({
        rowIndex: typeof issue.path[1] === "number" ? issue.path[1] : 0,
        field: typeof issue.path[2] === "string" ? issue.path[2] : "currentValue",
        message: issue.message,
      }));
      setErrors(buildErrorMap(rowErrors));
      setIsSaving(false);
      return;
    }

    const result = await bulkUpdateValuations(parsed.data);
    if (result.errors.length > 0) {
      setErrors(buildErrorMap(result.errors));
      setIsSaving(false);
      return;
    }

    setErrors(new Map());
    setStatus(`${result.updatedCount} values saved`);
    await loadRows();
    setIsSaving(false);
  }

  const columns = useMemo<ColumnDef<DraftRow>[]>(
    () => [
      {
        accessorKey: "name",
        header: "Asset",
        cell: ({ row }) => (
          <div className="min-w-48">
            <div className="font-medium">{row.original.name}</div>
            <div className="text-muted-foreground text-sm">
              {classificationLabel(row.original.classification)}
            </div>
            <StaleBadge staleness={row.original.staleness} />
          </div>
        ),
      },
      {
        accessorKey: "currentValue",
        header: "Current value",
        cell: ({ row }) => (
          <EditableCell
            label="Current value"
            value={row.original.currentValue}
            error={errors.get(errorKey(row.index, "currentValue"))}
            onChange={(currentValue) => updateRow(row.index, { currentValue })}
          />
        ),
      },
      {
        accessorKey: "valuationDate",
        header: "Valuation date",
        cell: ({ row }) => (
          <EditableCell
            label="Valuation date"
            type="date"
            value={row.original.valuationDate}
            error={errors.get(errorKey(row.index, "valuationDate"))}
            onChange={(valuationDate) => updateRow(row.index, { valuationDate })}
          />
        ),
      },
      {
        accessorKey: "currency",
        header: "Currency",
        cell: ({ row }) => (
          <EditableCell
            label="Currency"
            value={row.original.currency}
            error={errors.get(errorKey(row.index, "currency"))}
            onChange={(currency) => updateRow(row.index, { currency: currency.toUpperCase() })}
          />
        ),
      },
      {
        accessorKey: "notes",
        header: "Notes",
        cell: ({ row }) => (
          <EditableCell
            label="Notes"
            value={row.original.notes ?? ""}
            error={errors.get(errorKey(row.index, "notes"))}
            onChange={(notes) => updateRow(row.index, { notes })}
          />
        ),
      },
      {
        id: "actions",
        header: "Actions",
        cell: ({ row }) => (
          <div className="flex min-w-48 flex-wrap gap-2">
            <Button type="button" variant="outline" onClick={() => markUnchanged(row.index)}>
              <Icons.Check className="size-4" aria-hidden="true" />
              Mark unchanged
            </Button>
            <Button type="button" variant="ghost" disabled title="Document Vault arrives in Phase 2">
              <Icons.Upload className="size-4" aria-hidden="true" />
              Upload document
            </Button>
            <Button type="button" variant="ghost" disabled title="Web Evidence arrives in Phase 5">
              <Icons.Search className="size-4" aria-hidden="true" />
              Find evidence
            </Button>
            <Button type="button" variant="ghost" onClick={() => void showHistory(row.original)}>
              <Icons.History className="size-4" aria-hidden="true" />
              View history
            </Button>
          </div>
        ),
      },
    ],
    [errors, markUnchanged, showHistory, updateRow],
  );

  const table = useReactTable({
    data: rows,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <Page>
      <PageHeader heading="Update Values" />
      <PageContent className="space-y-6 pb-28">
        {status && <div className="bg-muted rounded-md px-4 py-3 text-sm">{status}</div>}

        {isLoading ? (
          <div className="text-muted-foreground py-12 text-center">Loading manually valued assets...</div>
        ) : rows.length === 0 ? (
          <Card>
            <CardHeader>
              <h2 className="text-lg font-semibold">No manually valued assets</h2>
            </CardHeader>
            <CardContent className="text-muted-foreground">
              Add a property, private investment, commodity, business, insurance, collectible, or
              custom asset before updating values here.
            </CardContent>
          </Card>
        ) : (
          <div className="overflow-x-auto rounded-md border">
            <table className="w-full min-w-[1120px] border-collapse">
              <thead className="bg-muted/60">
                {table.getHeaderGroups().map((headerGroup) => (
                  <tr key={headerGroup.id}>
                    {headerGroup.headers.map((header) => (
                      <th key={header.id} className="px-4 py-3 text-left text-sm font-semibold">
                        {flexRender(header.column.columnDef.header, header.getContext())}
                      </th>
                    ))}
                  </tr>
                ))}
              </thead>
              <tbody>
                {table.getRowModel().rows.map((row) => (
                  <tr key={row.id} className="min-h-20 border-t align-top">
                    {row.getVisibleCells().map((cell) => (
                      <td key={cell.id} className="px-4 py-4">
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {historyAssetName && (
          <Card data-testid="valuation-history">
            <CardHeader>
              <h2 className="text-lg font-semibold">History: {historyAssetName}</h2>
            </CardHeader>
            <CardContent>
              {history.length === 0 ? (
                <div className="text-muted-foreground">No valuation history yet.</div>
              ) : (
                <div className="space-y-2">
                  {history.map((item) => (
                    <div key={item.id} className="flex justify-between gap-4 border-b py-2 text-sm">
                      <span>{item.valuationDate}</span>
                      <span className="font-medium">
                        {item.valueNative} {item.currency}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        )}
      </PageContent>

      <div className="bg-background/95 fixed inset-x-0 bottom-0 border-t p-4 backdrop-blur">
        <div className="mx-auto flex max-w-7xl items-center justify-between gap-4">
          <div className="text-muted-foreground text-sm">
            {rows.length} manually valued {rows.length === 1 ? "asset" : "assets"}
          </div>
          <Button type="button" onClick={() => void saveAll()} disabled={isSaving || isLoading}>
            {isSaving ? <Icons.Spinner className="size-4 animate-spin" aria-hidden="true" /> : null}
            Save values
          </Button>
        </div>
      </div>
    </Page>
  );
}

function EditableCell({
  label,
  value,
  error,
  type = "text",
  onChange,
}: {
  label: string;
  value: string;
  error?: string;
  type?: string;
  onChange: (value: string) => void;
}) {
  return (
    <div className="min-w-40 space-y-1">
      <Label className="sr-only">{label}</Label>
      <Input value={value} type={type} onChange={(event) => onChange(event.target.value)} />
      {error && <div className="text-destructive text-sm">{error}</div>}
    </div>
  );
}

function StaleBadge({ staleness }: { staleness: ManualValuationStaleness }) {
  if (staleness === "critical") {
    return <div className="mt-2 text-sm font-medium text-red-700">Critical: over 90 days old</div>;
  }
  if (staleness === "warning") {
    return <div className="mt-2 text-sm font-medium text-amber-700">Warning: over 45 days old</div>;
  }
  return <div className="mt-2 text-sm text-emerald-700">Current</div>;
}

function classificationLabel(value: string): string {
  return value.replaceAll("_", " ");
}
