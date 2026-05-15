import type {
  AppliedCorporateAction,
  ApplyCorporateActionRequest,
  CorporateAction,
  CorporateActionPreview,
} from "@/adapters";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const listCorporateActionsMock = vi.fn<(assetId: string) => Promise<CorporateAction[]>>();
const previewCorporateActionMock =
  vi.fn<(request: ApplyCorporateActionRequest) => Promise<CorporateActionPreview>>();
const applyCorporateActionMock =
  vi.fn<(request: ApplyCorporateActionRequest) => Promise<AppliedCorporateAction>>();

vi.mock("@/adapters", () => ({
  listCorporateActions: (assetId: string) => listCorporateActionsMock(assetId),
  previewCorporateAction: (request: ApplyCorporateActionRequest) =>
    previewCorporateActionMock(request),
  applyCorporateAction: (request: ApplyCorporateActionRequest) =>
    applyCorporateActionMock(request),
}));

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [key: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  CardTitle: ({ children }: { children: React.ReactNode }) => <h2>{children}</h2>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock("@mizan/ui/components/ui/button", () => ({
  Button: ({ children, ...props }: React.ButtonHTMLAttributes<HTMLButtonElement>) => (
    <button {...props}>{children}</button>
  ),
}));

vi.mock("@mizan/ui/components/ui/input", () => ({
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
}));

vi.mock("@mizan/ui/components/ui/label", () => ({
  Label: ({ children, ...props }: React.LabelHTMLAttributes<HTMLLabelElement>) => (
    <label {...props}>{children}</label>
  ),
}));

vi.mock("@mizan/ui/components/ui/alert-dialog", () => ({
  AlertDialog: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  AlertDialogAction: ({
    children,
    ...props
  }: React.ButtonHTMLAttributes<HTMLButtonElement>) => <button {...props}>{children}</button>,
  AlertDialogCancel: ({ children }: { children: React.ReactNode }) => <button>{children}</button>,
  AlertDialogContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  AlertDialogDescription: ({ children }: { children: React.ReactNode }) => <p>{children}</p>,
  AlertDialogFooter: ({ children }: { children: React.ReactNode }) => <footer>{children}</footer>,
  AlertDialogHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  AlertDialogTitle: ({ children }: { children: React.ReactNode }) => <h3>{children}</h3>,
}));

import { CorporateActionsPanel } from "./corporate-actions-panel";

function renderPanel() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={client}>
      <CorporateActionsPanel assetId="asset-1" />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  listCorporateActionsMock.mockReset();
  previewCorporateActionMock.mockReset();
  applyCorporateActionMock.mockReset();
});

describe("CorporateActionsPanel", () => {
  it("renders an honest empty action history", async () => {
    listCorporateActionsMock.mockResolvedValueOnce([]);

    renderPanel();

    expect(await screen.findByTestId("corporate-actions-empty")).toHaveTextContent(
      "No reviewed corporate actions",
    );
  });

  it("previews and confirms a reviewed split without web auto-apply", async () => {
    listCorporateActionsMock.mockResolvedValue([]);
    previewCorporateActionMock.mockResolvedValueOnce({
      assetId: "asset-1",
      actionType: "split",
      effectiveDate: "2026-01-15",
      ratio: "2",
      newSymbol: null,
      positions: [
        {
          accountId: "acc-1",
          quantityBefore: "10",
          quantityAfter: "20",
          averageCostBefore: "200",
          averageCostAfter: "100",
          totalCostBasis: "2000",
          currency: "USD",
        },
      ],
      warnings: [],
    });
    applyCorporateActionMock.mockResolvedValueOnce({
      action: {
        id: "action-1",
        assetId: "asset-1",
        actionType: "split",
        effectiveDate: "2026-01-15",
        ratioNumerator: "2",
        ratioDenominator: "1",
        createdAt: "2026-01-15T00:00:00Z",
      },
      preview: {
        assetId: "asset-1",
        actionType: "split",
        effectiveDate: "2026-01-15",
        ratio: "2",
        positions: [],
        warnings: [],
      },
    });

    renderPanel();

    fireEvent.change(screen.getByLabelText("Effective date"), {
      target: { value: "2026-01-15" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Preview" }));

    expect(await screen.findByTestId("corporate-action-preview")).toHaveTextContent("10 to 20");
    expect(screen.getByText(/never applied from web evidence automatically/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Apply reviewed action" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await waitFor(() => {
      expect(applyCorporateActionMock).toHaveBeenCalledWith({
        assetId: "asset-1",
        actionType: "split",
        effectiveDate: "2026-01-15",
        ratioNumerator: "2",
        ratioDenominator: "1",
        newSymbol: null,
        sourceCitationId: null,
      });
    });
  });
});
