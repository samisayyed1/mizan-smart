import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  BulkUpdateValuationsRequest,
  BulkUpdateValuationsResult,
  ManualValuationAsset,
  ManualValuationHistoryRow,
} from "@/adapters";

const listManualValuationAssetsMock = vi.fn<() => Promise<ManualValuationAsset[]>>();
const bulkUpdateValuationsMock =
  vi.fn<(request: BulkUpdateValuationsRequest) => Promise<BulkUpdateValuationsResult>>();
const getManualValuationHistoryMock =
  vi.fn<(assetId: string) => Promise<ManualValuationHistoryRow[]>>();

vi.mock("@/adapters", () => ({
  listManualValuationAssets: () => listManualValuationAssetsMock(),
  bulkUpdateValuations: (request: BulkUpdateValuationsRequest) =>
    bulkUpdateValuationsMock(request),
  getManualValuationHistory: (assetId: string) => getManualValuationHistoryMock(assetId),
}));

vi.mock("@mizan/ui", () => ({
  Button: ({
    children,
    ...props
  }: React.ButtonHTMLAttributes<HTMLButtonElement> & { children: React.ReactNode }) => (
    <button type={props.type ?? "button"} {...props}>
      {children}
    </button>
  ),
  Icons: new Proxy(
    {},
    {
      get: (_target, prop) => () => <span>{String(prop)}</span>,
    },
  ),
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
  Label: (props: React.LabelHTMLAttributes<HTMLLabelElement>) => <label {...props} />,
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <main>{children}</main>,
  PageHeader: ({ heading }: { heading?: string }) => <header>{heading}</header>,
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [k: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

import ManualValuationsPage from "./manual-valuations-page";

const rows: ManualValuationAsset[] = [
  {
    assetId: "asset-1",
    name: "Primary residence",
    classification: "real_estate",
    currentValue: "750000",
    valuationDate: "2026-02-01",
    currency: "USD",
    notes: "prior appraisal",
    staleness: "critical",
    historyCount: 1,
  },
  {
    assetId: "asset-2",
    name: "Gold coins",
    classification: "gold",
    currentValue: "25000",
    valuationDate: "2026-04-01",
    currency: "USD",
    notes: null,
    staleness: "warning",
    historyCount: 0,
  },
];

beforeEach(() => {
  listManualValuationAssetsMock.mockReset().mockResolvedValue(rows);
  bulkUpdateValuationsMock.mockReset().mockResolvedValue({ updatedCount: 2, errors: [] });
  getManualValuationHistoryMock.mockReset().mockResolvedValue([
    {
      id: "v1",
      assetId: "asset-1",
      valuationDate: "2026-02-01",
      valueNative: "750000",
      currency: "USD",
      notes: "prior appraisal",
      createdAt: "2026-02-01T00:00:00Z",
    },
  ]);
});

describe("ManualValuationsPage", () => {
  it("renders stale indicators for critical and warning manual valuations", async () => {
    render(<ManualValuationsPage />);

    expect(await screen.findByText("Primary residence")).toBeInTheDocument();
    expect(screen.getByText("Critical: over 90 days old")).toBeInTheDocument();
    expect(screen.getByText("Warning: over 45 days old")).toBeInTheDocument();
  });

  it("rejects invalid decimal strings before bulk save", async () => {
    const user = userEvent.setup();
    render(<ManualValuationsPage />);

    const valueInput = await screen.findByDisplayValue("750000");
    await user.clear(valueInput);
    await user.type(valueInput, "1,000");
    await user.click(screen.getByText("Save values"));

    expect(await screen.findByText("Enter a valid decimal amount")).toBeInTheDocument();
    expect(bulkUpdateValuationsMock).not.toHaveBeenCalled();
  });

  it("saves a valid batch and preserves row payloads as decimal strings", async () => {
    const user = userEvent.setup();
    render(<ManualValuationsPage />);

    await screen.findByText("Primary residence");
    await user.click(screen.getByText("Save values"));

    await waitFor(() => expect(bulkUpdateValuationsMock).toHaveBeenCalledTimes(1));
    expect(bulkUpdateValuationsMock.mock.calls[0][0]).toEqual({
      rows: [
        {
          assetId: "asset-1",
          currentValue: "750000",
          valuationDate: "2026-02-01",
          currency: "USD",
          notes: "prior appraisal",
        },
        {
          assetId: "asset-2",
          currentValue: "25000",
          valuationDate: "2026-04-01",
          currency: "USD",
          notes: null,
        },
      ],
    });
  });

  it("loads valuation history from the row action", async () => {
    const user = userEvent.setup();
    render(<ManualValuationsPage />);

    await screen.findByText("Primary residence");
    await user.click(screen.getAllByText("View history")[0]);

    expect(await screen.findByTestId("valuation-history")).toHaveTextContent("750000 USD");
    expect(getManualValuationHistoryMock).toHaveBeenCalledWith("asset-1");
  });
});
