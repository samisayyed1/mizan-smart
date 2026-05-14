import type {
  DataLineageEntityType,
  DataLineageMetricType,
  DataLineageResponse,
} from "@/adapters";
import { getDataLineage } from "@/adapters";
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Icons,
} from "@mizan/ui";
import { useState } from "react";
import { Link } from "react-router-dom";

interface ExplainableNumberProps {
  entityType: DataLineageEntityType;
  entityId: string;
  metricType: DataLineageMetricType;
  label?: string;
}

export function ExplainableNumber({
  entityType,
  entityId,
  metricType,
  label = "Explain this number",
}: ExplainableNumberProps) {
  const [open, setOpen] = useState(false);
  const [lineage, setLineage] = useState<DataLineageResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadLineage = async () => {
    setOpen(true);
    if (lineage || isLoading) return;
    setIsLoading(true);
    setError(null);
    try {
      const response = await getDataLineage({ entityType, entityId, metricType });
      setLineage(response);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Unable to load lineage");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        aria-label={label}
        title={label}
        onClick={() => {
          void loadLineage();
        }}
      >
        <Icons.Info className="h-4 w-4" />
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-2xl">
          <DialogHeader>
            <DialogTitle>Explain This Number</DialogTitle>
            <DialogDescription>
              {lineage ? `${lineage.formulaName}: ${lineage.displayedValue}` : label}
            </DialogDescription>
          </DialogHeader>

          {isLoading ? <p className="text-muted-foreground text-sm">Loading lineage...</p> : null}
          {error ? (
            <p role="alert" className="text-destructive text-sm">
              {error}
            </p>
          ) : null}
          {lineage ? <LineageDetails lineage={lineage} /> : null}
        </DialogContent>
      </Dialog>
    </>
  );
}

function LineageDetails({ lineage }: { lineage: DataLineageResponse }) {
  return (
    <div className="space-y-5 text-sm">
      <section className="space-y-2">
        <h3 className="font-medium">Formula</h3>
        <p className="text-muted-foreground">{lineage.formulaDescription}</p>
        <dl className="grid gap-2 sm:grid-cols-2">
          <Detail label="Currency" value={lineage.currency ?? "No currency"} />
          <Detail label="Confidence" value={lineage.confidence ?? "Not stated"} />
          <Detail label="Freshness" value={lineage.freshness ?? "Not stated"} />
          <Detail label="Last updated" value={lineage.lastUpdated ?? "Not recorded"} />
          <Detail label="Rounding" value={lineage.roundingPolicy} />
        </dl>
      </section>

      <section className="space-y-2">
        <h3 className="font-medium">Inputs</h3>
        {lineage.inputRows.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs">
              <thead className="text-muted-foreground">
                <tr>
                  <th className="py-2 pr-3 font-medium">Source</th>
                  <th className="py-2 pr-3 font-medium">Label</th>
                  <th className="py-2 pr-3 font-medium">Value</th>
                  <th className="py-2 pr-3 font-medium">Date</th>
                </tr>
              </thead>
              <tbody>
                {lineage.inputRows.map((row) => (
                  <tr key={`${row.sourceTable}-${row.sourceId}`} className="border-border border-t">
                    <td className="py-2 pr-3">{row.sourceTable}</td>
                    <td className="py-2 pr-3">{row.label}</td>
                    <td className="py-2 pr-3">
                      {row.value}
                      {row.currency ? ` ${row.currency}` : ""}
                    </td>
                    <td className="py-2 pr-3">{row.asOfDate ?? "No date"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-muted-foreground">No input rows recorded.</p>
        )}
      </section>

      <section className="space-y-2">
        <h3 className="font-medium">Citations</h3>
        {lineage.sourceCitations.length > 0 ? (
          <ul className="space-y-1">
            {lineage.sourceCitations.map((citation) => (
              <li key={citation.id}>
                {citation.label}
                {citation.pageNumber ? `, page ${citation.pageNumber}` : ""}
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-muted-foreground">No source document linked yet</p>
        )}
      </section>

      <section className="space-y-2">
        <h3 className="font-medium">Source Documents</h3>
        {lineage.sourceDocuments.length > 0 ? (
          <ul className="space-y-1">
            {lineage.sourceDocuments.map((document) => (
              <li key={document.id}>
                <Link
                  className="text-primary underline-offset-4 hover:underline"
                  to={`/documents/review-queue?documentId=${encodeURIComponent(document.id)}`}
                >
                  {document.name}
                </Link>
                {document.pageNumber ? `, page ${document.pageNumber}` : ""}
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-muted-foreground">No source document linked yet</p>
        )}
      </section>

      {lineage.fxRatesUsed.length > 0 ? (
        <section className="space-y-2">
          <h3 className="font-medium">FX Rates Used</h3>
          <ul className="space-y-1">
            {lineage.fxRatesUsed.map((rate) => (
              <li key={`${rate.fromCurrency}-${rate.toCurrency}-${rate.asOfDate ?? "latest"}`}>
                {rate.fromCurrency} to {rate.toCurrency}: {rate.rate}
                {rate.asOfDate ? ` on ${rate.asOfDate}` : ""}
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <section className="space-y-2">
        <h3 className="font-medium">Warnings</h3>
        {lineage.warnings.length > 0 ? (
          <ul className="space-y-1">
            {lineage.warnings.map((warning) => (
              <li key={warning}>{warning}</li>
            ))}
          </ul>
        ) : (
          <p className="text-muted-foreground">No stale or missing data warnings.</p>
        )}
      </section>
    </div>
  );
}

function Detail({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-muted-foreground text-xs">{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

