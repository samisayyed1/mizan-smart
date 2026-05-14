import { useMemo, useState } from "react";

import {
  acceptReconciliationAdjustment,
  ignoreReconciliationMatch,
  manualReconciliationMatch,
  reconcileImportPreview,
  type JsonValue,
  type ReconciliationInputItem,
  type ReconciliationItem,
  type ReconciliationRunDetail,
} from "@/adapters";
import { Button, Input } from "@mizan/ui";

const EMPTY_ITEMS = "[]";

export default function ReconciliationCenterPage() {
  const [scopeId, setScopeId] = useState("import-preview");
  const [dateToleranceDays, setDateToleranceDays] = useState(0);
  const [externalJson, setExternalJson] = useState(EMPTY_ITEMS);
  const [accountId, setAccountId] = useState("");
  const [activityType, setActivityType] = useState("deposit");
  const [reason, setReason] = useState("");
  const [manualMizanItemId, setManualMizanItemId] = useState("");
  const [manualExternalItemId, setManualExternalItemId] = useState("");
  const [runDetail, setRunDetail] = useState<ReconciliationRunDetail | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const itemById = useMemo(() => {
    const map = new Map<string, ReconciliationItem>();
    runDetail?.items.forEach((item) => {
      map.set(item.id, item);
    });
    return map;
  }, [runDetail]);

  const runPreview = async () => {
    setError(null);
    setStatusMessage(null);
    try {
      const externalItems = parseInputItems(externalJson);
      const detail = await reconcileImportPreview({
        scopeType: "import",
        scopeId,
        mizanItems: [],
        externalItems,
        dateToleranceDays,
      });
      setRunDetail(detail);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not run reconciliation.");
    }
  };

  const acceptMatch = async (matchId: string) => {
    setError(null);
    setStatusMessage(null);
    try {
      const result = await acceptReconciliationAdjustment({
        matchId,
        accountId,
        activityType,
        reason,
      });
      setStatusMessage(`Adjustment accepted as activity ${result.activityId}.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not accept adjustment.");
    }
  };

  const ignoreMatch = async (matchId: string) => {
    setError(null);
    setStatusMessage(null);
    try {
      await ignoreReconciliationMatch({ matchId, reason });
      setStatusMessage("Match ignored.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not ignore match.");
    }
  };

  const createManualMatch = async () => {
    if (!runDetail) return;
    setError(null);
    setStatusMessage(null);
    try {
      await manualReconciliationMatch({
        runId: runDetail.run.id,
        mizanItemId: manualMizanItemId,
        externalItemId: manualExternalItemId,
        reason,
      });
      setStatusMessage("Manual match recorded.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not record manual match.");
    }
  };

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-6 p-6">
      <div>
        <h1 className="text-2xl font-semibold">Reconciliation Center</h1>
        <p className="text-muted-foreground text-sm">
          Prove Mizan rows against statement, document, or import evidence before writing anything.
        </p>
      </div>

      <section className="grid gap-3 md:grid-cols-[1fr_160px_auto]">
        <label className="grid gap-1 text-sm">
          Scope ID
          <Input value={scopeId} onChange={(event) => setScopeId(event.target.value)} />
        </label>
        <label className="grid gap-1 text-sm">
          Date tolerance
          <Input
            type="number"
            min={0}
            max={31}
            value={dateToleranceDays}
            onChange={(event) => setDateToleranceDays(Number(event.target.value))}
          />
        </label>
        <Button className="self-end" onClick={runPreview}>
          Run preview
        </Button>
      </section>

      <label className="grid gap-1 text-sm">
        External rows JSON
        <textarea
          className="border-input bg-background min-h-36 rounded-md border p-3 font-mono text-sm"
          value={externalJson}
          onChange={(event) => setExternalJson(event.target.value)}
        />
      </label>

      <section className="grid gap-3 md:grid-cols-3">
        <label className="grid gap-1 text-sm">
          Adjustment account
          <Input value={accountId} onChange={(event) => setAccountId(event.target.value)} />
        </label>
        <label className="grid gap-1 text-sm">
          Activity type
          <Input value={activityType} onChange={(event) => setActivityType(event.target.value)} />
        </label>
        <label className="grid gap-1 text-sm">
          Reason
          <Input value={reason} onChange={(event) => setReason(event.target.value)} />
        </label>
      </section>

      {error ? <p className="text-destructive text-sm">{error}</p> : null}
      {statusMessage ? <p className="text-sm">{statusMessage}</p> : null}

      {runDetail ? (
        <section className="overflow-x-auto">
          <table className="w-full border-collapse text-sm">
            <thead>
              <tr className="border-b text-left">
                <th className="py-2 pr-4">Mizan</th>
                <th className="py-2 pr-4">External</th>
                <th className="py-2 pr-4">Status</th>
                <th className="py-2 pr-4">Reason</th>
                <th className="py-2 pr-4">Actions</th>
              </tr>
            </thead>
            <tbody>
              {runDetail.matches.map((match) => {
                const mizanItem = match.mizanItemId ? itemById.get(match.mizanItemId) : null;
                const externalItem = match.externalItemId ? itemById.get(match.externalItemId) : null;
                return (
                  <tr key={match.id} className="border-b align-top">
                    <td className="py-3 pr-4">{formatItem(mizanItem)}</td>
                    <td className="py-3 pr-4">{formatItem(externalItem)}</td>
                    <td className="py-3 pr-4">{match.matchStatus.replaceAll("_", " ")}</td>
                    <td className="py-3 pr-4">{match.reason}</td>
                    <td className="flex gap-2 py-3 pr-4">
                      {match.matchStatus === "missing_in_mizan" ? (
                        <Button size="sm" onClick={() => void acceptMatch(match.id)}>
                          Accept adjustment
                        </Button>
                      ) : null}
                      <Button size="sm" variant="outline" onClick={() => void ignoreMatch(match.id)}>
                        Ignore
                      </Button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </section>
      ) : (
        <p className="text-muted-foreground text-sm">No reconciliation run yet.</p>
      )}

      {runDetail ? (
        <section className="grid gap-3 md:grid-cols-[1fr_1fr_auto]">
          <label className="grid gap-1 text-sm">
            Mizan item ID
            <Input
              value={manualMizanItemId}
              onChange={(event) => setManualMizanItemId(event.target.value)}
            />
          </label>
          <label className="grid gap-1 text-sm">
            External item ID
            <Input
              value={manualExternalItemId}
              onChange={(event) => setManualExternalItemId(event.target.value)}
            />
          </label>
          <Button className="self-end" variant="outline" onClick={() => void createManualMatch()}>
            Manual match
          </Button>
        </section>
      ) : null}
    </div>
  );
}

function formatItem(item: ReconciliationItem | null | undefined): string {
  if (!item) return "None";
  return [item.itemType, item.amount, item.currency, item.effectiveDate]
    .filter((value): value is string => Boolean(value))
    .join(" / ");
}

function parseInputItems(value: string): ReconciliationInputItem[] {
  const parsed: unknown = JSON.parse(value);
  if (!Array.isArray(parsed)) {
    throw new Error("External rows JSON must be an array.");
  }
  return parsed.map((item) => {
    if (!isRecord(item)) {
      throw new Error("Each external row must be an object.");
    }
    const itemType = asString(item.itemType, "itemType");
    const id = optionalString(item.id);
    return {
      ...(id ? { id } : {}),
      itemType,
      rawJson: toJsonValue(item.rawJson ?? item),
      amount: optionalString(item.amount),
      currency: optionalString(item.currency),
      effectiveDate: optionalString(item.effectiveDate),
    };
  });
}

function asString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${field} is required.`);
  }
  return value;
}

function optionalString(value: unknown): string | null {
  return typeof value === "string" && value.trim() !== "" ? value : null;
}

function toJsonValue(value: unknown): JsonValue {
  if (
    value === null ||
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map(toJsonValue);
  }
  if (isRecord(value)) {
    return Object.fromEntries(Object.entries(value).map(([key, entry]) => [key, toJsonValue(entry)]));
  }
  return null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
