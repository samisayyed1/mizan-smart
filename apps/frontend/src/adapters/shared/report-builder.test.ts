import { describe, expect, it, vi } from "vitest";

import { exportReport, generateReport, getReportRun } from "./report-builder";
import { invoke } from "./platform";

vi.mock("./platform", () => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<unknown>>(),
}));

const invokeMock = vi.mocked(invoke);

describe("report builder adapter", () => {
  it("generates deterministic report runs through the shared invoke channel", async () => {
    const request = { reportType: "tax_pack" as const, baseCurrency: "USD" };
    invokeMock.mockResolvedValueOnce({ id: "run-1", sections: [] });

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
});
