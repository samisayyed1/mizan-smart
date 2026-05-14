import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  CapitalCall,
  CreateCapitalCallRequest,
  PrivateInvestmentDetail,
  UpdateCapitalCallStatusRequest,
} from "@/adapters";

const getPrivateInvestmentDetailMock = vi.fn<(assetId: string) => Promise<PrivateInvestmentDetail | null>>();
const addCapitalCallMock = vi.fn<(request: CreateCapitalCallRequest) => Promise<CapitalCall>>();
const updateCapitalCallStatusMock = vi.fn<
  (request: UpdateCapitalCallStatusRequest) => Promise<CapitalCall>
>();

vi.mock("@/adapters", () => ({
  getPrivateInvestmentDetail: (assetId: string) => getPrivateInvestmentDetailMock(assetId),
  addCapitalCall: (request: CreateCapitalCallRequest) => addCapitalCallMock(request),
  updateCapitalCallStatus: (request: UpdateCapitalCallStatusRequest) =>
    updateCapitalCallStatusMock(request),
  addPrivateInvestmentValuation: vi.fn(),
  addPrivateDistribution: vi.fn(),
}));

import PrivateInvestmentDetailPage from "./private-investment-detail-page";

function baseDetail(overrides: Partial<PrivateInvestmentDetail> = {}): PrivateInvestmentDetail {
  return {
    summary: {
      investment: {
        assetId: "asset-1",
        manager: "Acme Capital",
        strategy: "Buyout",
        vintageYear: 2024,
        commitmentAmount: "1000",
        commitmentCurrency: "USD",
        inceptionDate: null,
        notes: null,
      },
      commitment: "1000",
      paidInCapital: "0",
      unfundedCommitment: "1000",
      totalDistributions: "0",
      recallableDistributions: "0",
      currentNav: "0",
      dpi: null,
      rvpi: null,
      tvpi: null,
      moic: null,
      warnings: [],
    },
    valuations: [],
    capitalCalls: [],
    distributions: [],
    upcomingCapitalCalls: [],
    jCurve: [],
    sourceCitationIds: [],
    ...overrides,
  };
}

function dueCall(overrides: Partial<CapitalCall> = {}): CapitalCall {
  return {
    id: "call-1",
    assetId: "asset-1",
    noticeDate: "2026-05-01",
    dueDate: "2026-05-15",
    amount: "250",
    currency: "USD",
    status: "due",
    sourceCitationId: "citation-1",
    notes: null,
    ...overrides,
  };
}

function renderPage() {
  return render(
    <MemoryRouter initialEntries={["/private-investments/asset-1"]}>
      <Routes>
        <Route path="/private-investments/:assetId" element={<PrivateInvestmentDetailPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("PrivateInvestmentDetailPage", () => {
  beforeEach(() => {
    getPrivateInvestmentDetailMock.mockReset().mockResolvedValue(baseDetail());
    addCapitalCallMock.mockReset().mockResolvedValue(dueCall());
    updateCapitalCallStatusMock.mockReset().mockResolvedValue(dueCall({ status: "paid" }));
  });

  it("renders an empty private fund without fake chart data", async () => {
    renderPage();

    expect(await screen.findByText("Acme Capital")).toBeInTheDocument();
    expect(screen.getAllByText("$1,000.00").length).toBeGreaterThan(0);
    expect(screen.getByText("No cashflow or NAV data yet.")).toBeInTheDocument();
    expect(screen.getByText("No expected or due capital calls.")).toBeInTheDocument();
  });

  it("renders populated metrics, upcoming calls, and linked citations", async () => {
    getPrivateInvestmentDetailMock.mockResolvedValue(
      baseDetail({
        summary: {
          ...baseDetail().summary,
          paidInCapital: "400",
          unfundedCommitment: "600",
          totalDistributions: "80",
          currentNav: "450",
          dpi: "0.2",
          rvpi: "1.125",
          tvpi: "1.325",
          moic: "1.325",
        },
        capitalCalls: [dueCall({ status: "paid" })],
        upcomingCapitalCalls: [dueCall()],
        jCurve: [
          { date: "2026-05-15", cumulativeNetCashflow: "-400", nav: null },
          { date: "2026-06-30", cumulativeNetCashflow: "-320", nav: "450" },
        ],
        sourceCitationIds: ["citation-1"],
      }),
    );

    renderPage();

    expect(await screen.findByText("$400.00")).toBeInTheDocument();
    expect(screen.getByText("1.13x")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Private investment J-curve" })).toBeInTheDocument();
    expect(screen.getAllByText(/Source citation: citation-1/).length).toBeGreaterThan(0);
  });

  it("adds a capital call and reloads metrics", async () => {
    const user = userEvent.setup();
    getPrivateInvestmentDetailMock
      .mockResolvedValueOnce(baseDetail())
      .mockResolvedValueOnce(
        baseDetail({
          summary: { ...baseDetail().summary, unfundedCommitment: "875" },
          upcomingCapitalCalls: [dueCall({ amount: "125" })],
        }),
      );
    renderPage();

    await screen.findByText("Acme Capital");
    await user.type(screen.getByLabelText("Call amount"), "125");
    await user.click(screen.getByRole("button", { name: /Save call/ }));

    await waitFor(() => {
      expect(addCapitalCallMock).toHaveBeenCalledWith(
        expect.objectContaining({ assetId: "asset-1", amount: "125", currency: "USD", status: "due" }),
      );
    });
    expect(await screen.findByText("Capital call saved.")).toBeInTheDocument();
    expect(getPrivateInvestmentDetailMock).toHaveBeenCalledTimes(2);
  });

  it("marks a capital call paid and refreshes the detail payload", async () => {
    const user = userEvent.setup();
    getPrivateInvestmentDetailMock
      .mockResolvedValueOnce(baseDetail({ upcomingCapitalCalls: [dueCall()] }))
      .mockResolvedValueOnce(
        baseDetail({
          summary: { ...baseDetail().summary, paidInCapital: "250", unfundedCommitment: "750" },
          capitalCalls: [dueCall({ status: "paid" })],
        }),
      );

    renderPage();

    await screen.findByText("Due 2026-05-15");
    await user.click(screen.getByRole("button", { name: /Mark paid/ }));

    await waitFor(() => {
      expect(updateCapitalCallStatusMock).toHaveBeenCalledWith({ id: "call-1", status: "paid" });
    });
    expect(await screen.findByText("Capital call marked paid.")).toBeInTheDocument();
  });
});
