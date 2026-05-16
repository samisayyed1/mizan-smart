import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  addManualFeeEntry,
  exportReport,
  generateReport,
  getConcentrationFragilitySummary,
  getFeeIntelligenceSummary,
  getReportRun,
  type GenerateReportRequest,
} from "./report-builder";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("report builder adapter", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("generates deterministic report runs through the shared invoke channel", async () => {
    const request = { reportType: "tax_pack" as const, baseCurrency: "USD" };
    invokeMock.mockResolvedValueOnce({ id: "run-1", sections: [] });

    await generateReport(request);

    expect(invokeMock).toHaveBeenCalledWith("generate_report", { request });
  });

  it("passes the selected month for monthly wealth letters", async () => {
    const request = {
      reportType: "monthly_wealth_letter" as const,
      baseCurrency: "USD",
      periodMonth: "2026-05",
    };
    invokeMock.mockResolvedValueOnce({ id: "run-monthly", sections: [] });

    await generateReport(request);

    expect(invokeMock).toHaveBeenCalledWith("generate_report", { request });
  });

  it("passes selected sections for estate binders", async () => {
    const request: GenerateReportRequest = {
      reportType: "estate_binder" as const,
      baseCurrency: "USD",
      includedSections: ["accounts", "documents_manifest"],
    };
    invokeMock.mockResolvedValueOnce({ id: "run-estate", sections: [] });

    await generateReport(request);

    expect(invokeMock).toHaveBeenCalledWith("generate_report", { request });
  });

  it("loads report runs by id", async () => {
    invokeMock.mockResolvedValueOnce(null);

    await getReportRun("run-1");

    expect(invokeMock).toHaveBeenCalledWith("get_report_run", { reportRunId: "run-1" });
  });

  it("requests report export bundles by id", async () => {
    invokeMock.mockResolvedValueOnce({ fileName: "report.html", bytes: [] });

    await exportReport("run-1");

    expect(invokeMock).toHaveBeenCalledWith("export_report", { reportRunId: "run-1" });
  });

  it("saves manual fee entries and loads fee intelligence summaries", async () => {
    const input = {
      feeDate: "2026-05-10",
      category: "transaction_fees" as const,
      amount: "12.34",
      currency: "USD",
      notes: "Broker statement",
    };
    invokeMock.mockResolvedValueOnce({ id: "fee-1", ...input });
    invokeMock.mockResolvedValueOnce({ periodMonth: "2026-05", totals: [] });

    await addManualFeeEntry(input);
    await getFeeIntelligenceSummary("2026-05");

    expect(invokeMock).toHaveBeenNthCalledWith(1, "add_manual_fee_entry", { input });
    expect(invokeMock).toHaveBeenNthCalledWith(2, "get_fee_intelligence_summary", {
      periodMonth: "2026-05",
    });
  });

  it("loads concentration and fragility summaries", async () => {
    invokeMock.mockResolvedValueOnce({
      asOfDate: "2026-05-16",
      baseCurrency: "USD",
      totalWealth: "100",
      exposures: [],
      findings: [],
      emptyState: false,
      islamicModeEnabled: false,
      taxonomyState: "missing",
    });

    await getConcentrationFragilitySummary("USD");

    expect(invokeMock).toHaveBeenCalledWith("get_concentration_fragility_summary", {
      baseCurrency: "USD",
    });
  });
});
