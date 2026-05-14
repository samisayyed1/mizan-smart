import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { InputHTMLAttributes, ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import ReconciliationCenterPage from "./reconciliation-center-page";

type MockCommand = (...args: unknown[]) => Promise<unknown>;

const reconcileImportPreview = vi.fn<MockCommand>();
const acceptReconciliationAdjustment = vi.fn<MockCommand>();
const ignoreReconciliationMatch = vi.fn<MockCommand>();
const manualReconciliationMatch = vi.fn<MockCommand>();

vi.mock("@/adapters", () => ({
  reconcileImportPreview: (...args: unknown[]) => reconcileImportPreview(...args),
  acceptReconciliationAdjustment: (...args: unknown[]) => acceptReconciliationAdjustment(...args),
  ignoreReconciliationMatch: (...args: unknown[]) => ignoreReconciliationMatch(...args),
  manualReconciliationMatch: (...args: unknown[]) => manualReconciliationMatch(...args),
}));

vi.mock("@mizan/ui", () => ({
  Button: ({
    children,
    onClick,
    type,
  }: {
    children: ReactNode;
    onClick?: () => void;
    type?: "button" | "submit";
  }) => (
    <button type={type ?? "button"} onClick={onClick}>
      {children}
    </button>
  ),
  Input: (props: InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
}));

describe("ReconciliationCenterPage", () => {
  beforeEach(() => {
    reconcileImportPreview.mockReset();
    acceptReconciliationAdjustment.mockReset();
    ignoreReconciliationMatch.mockReset();
    manualReconciliationMatch.mockReset();
  });

  it("renders side-by-side rows from an import preview run", async () => {
    reconcileImportPreview.mockResolvedValue({
      run: {
        id: "run-1",
        scopeType: "import",
        scopeId: "import-preview",
        status: "completed",
        dateToleranceDays: 0,
        createdAt: "2026-05-14T00:00:00Z",
        completedAt: "2026-05-14T00:00:00Z",
      },
      items: [
        {
          id: "external-1",
          runId: "run-1",
          itemType: "activity",
          sourceSide: "external",
          rawJson: {},
          normalizedHash: "hash",
          amount: "25.00",
          currency: "USD",
          effectiveDate: "2026-05-14",
          status: "open",
        },
      ],
      matches: [
        {
          id: "match-1",
          runId: "run-1",
          mizanItemId: null,
          externalItemId: "external-1",
          matchStatus: "missing_in_mizan",
          confidence: "0.00",
          reason: "External item has no matching Mizan item.",
          createdAt: "2026-05-14T00:00:00Z",
        },
      ],
    });

    render(<ReconciliationCenterPage />);
    fireEvent.change(screen.getByLabelText("External rows JSON"), {
      target: {
        value:
          '[{"id":"external-1","itemType":"activity","amount":"25.00","currency":"USD","effectiveDate":"2026-05-14"}]',
      },
    });
    fireEvent.click(screen.getByText("Run preview"));

    expect(await screen.findByText("missing in mizan")).toBeInTheDocument();
    expect(screen.getByText("activity / 25.00 / USD / 2026-05-14")).toBeInTheDocument();
    expect(screen.getByText("Accept adjustment")).toBeInTheDocument();
  });

  it("requires user action before accepting an adjustment", async () => {
    acceptReconciliationAdjustment.mockResolvedValue({ activityId: "activity-1" });
    reconcileImportPreview.mockResolvedValue({
      run: {
        id: "run-1",
        scopeType: "import",
        scopeId: "import-preview",
        status: "completed",
        dateToleranceDays: 0,
        createdAt: "2026-05-14T00:00:00Z",
        completedAt: "2026-05-14T00:00:00Z",
      },
      items: [
        {
          id: "external-1",
          runId: "run-1",
          itemType: "activity",
          sourceSide: "external",
          rawJson: {},
          normalizedHash: "hash",
          amount: "25.00",
          currency: "USD",
          effectiveDate: "2026-05-14",
          status: "open",
        },
      ],
      matches: [
        {
          id: "match-1",
          runId: "run-1",
          mizanItemId: null,
          externalItemId: "external-1",
          matchStatus: "missing_in_mizan",
          confidence: "0.00",
          reason: "External item has no matching Mizan item.",
          createdAt: "2026-05-14T00:00:00Z",
        },
      ],
    });

    render(<ReconciliationCenterPage />);
    fireEvent.click(screen.getByText("Run preview"));
    fireEvent.change(screen.getByLabelText("Adjustment account"), { target: { value: "account-1" } });
    fireEvent.change(screen.getByLabelText("Reason"), { target: { value: "statement row" } });
    fireEvent.click(await screen.findByText("Accept adjustment"));

    await waitFor(() => {
      expect(acceptReconciliationAdjustment).toHaveBeenCalledWith({
        matchId: "match-1",
        accountId: "account-1",
        activityType: "deposit",
        reason: "statement row",
      });
    });
  });
});
