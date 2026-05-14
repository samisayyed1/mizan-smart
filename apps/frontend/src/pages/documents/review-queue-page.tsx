import {
  approveExtractedFact,
  deferExtractedFact,
  getParsedDocument,
  linkExtractedFactToEntity,
  listPendingExtractedFacts,
  rejectExtractedFact,
  updateExtractedFactBeforeApproval,
} from "@/adapters";
import type {
  ExtractedFact,
  ExtractedFactLinkEntityType,
  ParsedDocument,
  ParsedTextBlock,
} from "@/adapters";
import { Button, Icons, Page, PageContent, PageHeader } from "@mizan/ui";
import { useEffect, useMemo, useState } from "react";

function confidenceLabel(confidence: number | null): string {
  return confidence === null ? "No confidence" : `${Math.round(confidence * 100)}%`;
}

function pageLabel(pageNumber: number | null): string {
  return pageNumber === null ? "No page" : `Page ${pageNumber}`;
}

function isDecimal(value: string): boolean {
  return /^-?\d+(\.\d+)?$/.test(value.trim());
}

function blocksForFact(parsed: ParsedDocument | null, fact: ExtractedFact | null): ParsedTextBlock[] {
  if (!parsed || !fact?.pageNumber) return [];
  return parsed.textBlocks.filter((block) => block.pageNumber === fact.pageNumber);
}

export default function DocumentReviewQueuePage() {
  const [facts, setFacts] = useState<ExtractedFact[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [parsedDocument, setParsedDocument] = useState<ParsedDocument | null>(null);
  const [normalizedValue, setNormalizedValue] = useState("");
  const [currency, setCurrency] = useState("");
  const [dateValue, setDateValue] = useState("");
  const [notes, setNotes] = useState("");
  const [entityType, setEntityType] = useState<ExtractedFactLinkEntityType>("asset");
  const [entityId, setEntityId] = useState("");
  const [reviewedFact, setReviewedFact] = useState<ExtractedFact | null>(null);
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const selectedFact = facts.find((fact) => fact.id === selectedId) ?? facts[0] ?? reviewedFact;
  const textBlocks = useMemo(
    () => blocksForFact(parsedDocument, selectedFact ?? null),
    [parsedDocument, selectedFact],
  );

  async function refreshFacts(): Promise<void> {
    setLoading(true);
    try {
      const loaded = await listPendingExtractedFacts();
      setFacts(loaded);
      setSelectedId((current) => current ?? loaded[0]?.id ?? null);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not load extracted facts.");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refreshFacts();
  }, []);

  useEffect(() => {
    if (!selectedFact) {
      setParsedDocument(null);
      return;
    }
    setNormalizedValue(selectedFact.normalizedValue ?? "");
    setCurrency(selectedFact.currency ?? "");
    setDateValue(selectedFact.dateValue ?? "");
    setNotes(selectedFact.reviewNotes ?? "");
    let cancelled = false;
    getParsedDocument(selectedFact.documentId)
      .then((parsed) => {
        if (!cancelled) setParsedDocument(parsed);
      })
      .catch(() => {
        if (!cancelled) setParsedDocument(null);
      });
    return () => {
      cancelled = true;
    };
  }, [selectedFact]);

  function removePendingFact(factId: string, nextFact: ExtractedFact): void {
    setFacts((current) => current.filter((fact) => fact.id !== factId));
    setSelectedId(null);
    setReviewedFact(nextFact);
  }

  async function handleApprove(): Promise<void> {
    if (!selectedFact) return;
    setWorking(true);
    setError(null);
    try {
      const approved = await approveExtractedFact(selectedFact.id, {
        reviewNotes: notes || null,
      });
      removePendingFact(selectedFact.id, approved);
      setMessage("Fact approved");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not approve fact.");
    } finally {
      setWorking(false);
    }
  }

  async function handleEditAndApprove(): Promise<void> {
    if (!selectedFact) return;
    if (currency.trim() && normalizedValue.trim() && !isDecimal(normalizedValue)) {
      setError("Normalized value must be a decimal amount before approval.");
      return;
    }
    setWorking(true);
    setError(null);
    try {
      await updateExtractedFactBeforeApproval(selectedFact.id, {
        normalizedValue: normalizedValue || null,
        currency: currency || null,
        dateValue: dateValue || null,
        reviewNotes: notes || null,
      });
      const approved = await approveExtractedFact(selectedFact.id, {
        reviewNotes: notes || null,
      });
      removePendingFact(selectedFact.id, approved);
      setMessage("Fact edited and approved");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not edit and approve fact.");
    } finally {
      setWorking(false);
    }
  }

  async function handleReject(): Promise<void> {
    if (!selectedFact) return;
    setWorking(true);
    setError(null);
    try {
      const rejected = await rejectExtractedFact(selectedFact.id, {
        reviewNotes: notes || null,
      });
      removePendingFact(selectedFact.id, rejected);
      setMessage("Fact rejected");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not reject fact.");
    } finally {
      setWorking(false);
    }
  }

  async function handleLink(): Promise<void> {
    if (!selectedFact) return;
    setWorking(true);
    setError(null);
    try {
      await linkExtractedFactToEntity(selectedFact.id, {
        entityType,
        entityId,
        reviewNotes: notes || null,
      });
      setMessage(`Linked to ${entityType}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not link fact.");
    } finally {
      setWorking(false);
    }
  }

  async function handleDefer(): Promise<void> {
    if (!selectedFact) return;
    setWorking(true);
    setError(null);
    try {
      const deferred = await deferExtractedFact(selectedFact.id, {
        reviewNotes: notes || null,
      });
      setReviewedFact(deferred);
      setMessage("Fact deferred");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not defer fact.");
    } finally {
      setWorking(false);
    }
  }

  return (
    <Page>
      <PageHeader heading="Document Review Queue" text="Review extracted facts before using them." />
      <PageContent>
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_420px]">
          <section className="border-border min-h-96 rounded-lg border p-4">
            <div className="flex items-center justify-between gap-3">
              <h2 className="text-base font-semibold">Document text</h2>
              {selectedFact ? (
                <span className="text-muted-foreground text-sm">{pageLabel(selectedFact.pageNumber)}</span>
              ) : null}
            </div>
            {!selectedFact ? (
              <p className="text-muted-foreground mt-6 text-sm">No fact selected</p>
            ) : textBlocks.length === 0 ? (
              <p className="text-muted-foreground mt-6 text-sm">
                Text blocks unavailable for this document page
              </p>
            ) : (
              <div className="mt-4 space-y-3">
                {textBlocks.map((block) => (
                  <p
                    key={`${block.pageNumber}-${block.blockOrder}`}
                    className="border-border bg-muted/20 rounded-md border px-3 py-2 text-sm"
                  >
                    {block.text}
                  </p>
                ))}
              </div>
            )}
          </section>

          <section className="border-border rounded-lg border">
            <div className="border-border flex items-center justify-between border-b px-4 py-3">
              <h2 className="text-base font-semibold">Extracted facts</h2>
              {loading ? <Icons.Loader className="text-muted-foreground size-4 animate-spin" /> : null}
            </div>
            {error ? (
              <div role="alert" className="text-destructive px-4 py-3 text-sm">
                {error}
              </div>
            ) : null}
            {message ? <p className="text-muted-foreground px-4 pt-3 text-sm">{message}</p> : null}
            {reviewedFact ? (
              <p className="px-4 pt-3 text-sm">
                Last reviewed status: <span className="font-medium">{reviewedFact.status}</span>
              </p>
            ) : null}
            {facts.length === 0 && !loading ? (
              <p className="text-muted-foreground px-4 py-8 text-sm">No pending extracted facts</p>
            ) : (
              <div className="divide-border divide-y">
                {facts.map((fact) => (
                  <button
                    key={fact.id}
                    type="button"
                    className="hover:bg-muted/30 block w-full px-4 py-3 text-left"
                    onClick={() => setSelectedId(fact.id)}
                  >
                    <span className="block text-sm font-medium">{fact.factType}</span>
                    <span className="text-muted-foreground block text-xs">
                      {fact.rawValue} · {pageLabel(fact.pageNumber)} · {confidenceLabel(fact.confidenceScore)}
                    </span>
                  </button>
                ))}
              </div>
            )}

            {selectedFact ? (
              <div className="border-border space-y-3 border-t p-4">
                <dl className="grid grid-cols-2 gap-3 text-sm">
                  <div>
                    <dt className="text-muted-foreground">Raw value</dt>
                    <dd>{selectedFact.rawValue}</dd>
                  </div>
                  <div>
                    <dt className="text-muted-foreground">Normalized</dt>
                    <dd>{selectedFact.normalizedValue ?? "None"}</dd>
                  </div>
                  <div>
                    <dt className="text-muted-foreground">Confidence</dt>
                    <dd>{confidenceLabel(selectedFact.confidenceScore)}</dd>
                  </div>
                  <div>
                    <dt className="text-muted-foreground">Source highlight</dt>
                    <dd>{selectedFact.boundingBox ? "Bounding box available" : "No bounding box"}</dd>
                  </div>
                  <div className="col-span-2">
                    <dt className="text-muted-foreground">Suggested target mapping</dt>
                    <dd>No suggested target mapping</dd>
                  </div>
                </dl>

                <label className="block text-sm">
                  <span className="text-muted-foreground">Normalized value</span>
                  <input
                    className="border-input mt-1 w-full rounded-md border px-3 py-2"
                    value={normalizedValue}
                    onChange={(event) => setNormalizedValue(event.currentTarget.value)}
                  />
                </label>
                <div className="grid grid-cols-2 gap-3">
                  <label className="block text-sm">
                    <span className="text-muted-foreground">Currency</span>
                    <input
                      className="border-input mt-1 w-full rounded-md border px-3 py-2"
                      value={currency}
                      onChange={(event) => setCurrency(event.currentTarget.value)}
                    />
                  </label>
                  <label className="block text-sm">
                    <span className="text-muted-foreground">Date</span>
                    <input
                      className="border-input mt-1 w-full rounded-md border px-3 py-2"
                      value={dateValue}
                      onChange={(event) => setDateValue(event.currentTarget.value)}
                    />
                  </label>
                </div>
                <label className="block text-sm">
                  <span className="text-muted-foreground">Review notes</span>
                  <textarea
                    className="border-input mt-1 w-full rounded-md border px-3 py-2"
                    value={notes}
                    onChange={(event) => setNotes(event.currentTarget.value)}
                  />
                </label>
                <div className="grid grid-cols-[120px_minmax(0,1fr)_auto] gap-2">
                  <select
                    aria-label="Link entity type"
                    className="border-input rounded-md border px-2"
                    value={entityType}
                    onChange={(event) => setEntityType(event.currentTarget.value as ExtractedFactLinkEntityType)}
                  >
                    <option value="asset">Asset</option>
                    <option value="account">Account</option>
                  </select>
                  <input
                    aria-label="Link entity id"
                    className="border-input rounded-md border px-3 py-2"
                    value={entityId}
                    onChange={(event) => setEntityId(event.currentTarget.value)}
                  />
                  <Button type="button" variant="secondary" disabled={working || !entityId} onClick={() => void handleLink()}>
                    Link
                  </Button>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button type="button" disabled={working} onClick={() => void handleApprove()}>
                    Approve
                  </Button>
                  <Button type="button" variant="secondary" disabled={working} onClick={() => void handleEditAndApprove()}>
                    Edit and approve
                  </Button>
                  <Button type="button" variant="secondary" disabled={working} onClick={() => void handleDefer()}>
                    Defer
                  </Button>
                  <Button type="button" variant="destructive" disabled={working} onClick={() => void handleReject()}>
                    Reject
                  </Button>
                </div>
              </div>
            ) : null}
          </section>
        </div>
      </PageContent>
    </Page>
  );
}
