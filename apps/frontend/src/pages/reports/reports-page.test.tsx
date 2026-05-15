import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { exportReport, generateReport } from "@/adapters";

vi.mock("@/adapters", () => ({
  exportReport: vi.fn(),
  generateReport: vi.fn(),
}));

vi.mock("@mizan/ui", () => ({
  Button: ({
    children,
    ...props
  }: {
    children: React.ReactNode;
    [key: string]: unknown;
  }) => (
    <button type="button" {...props}>
      {children}
    </button>
  ),
  Icons: {
    TrendingUp: (props: React.HTMLAttributes<HTMLSpanElement>) => <span {...props}>TrendingUp</span>,
    HandCoins: (props: React.HTMLAttributes<HTMLSpanElement>) => <span {...props}>HandCoins</span>,
    PieChart: (props: React.HTMLAttributes<HTMLSpanElement>) => <span {...props}>PieChart</span>,
    ShieldCheck: (props: React.HTMLAttributes<HTMLSpanElement>) => <span {...props}>ShieldCheck</span>,
    FileText: (props: React.HTMLAttributes<HTMLSpanElement>) => <span {...props}>FileText</span>,
  },
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageHeader: ({ heading }: { heading?: string }) => <header>{heading}</header>,
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children }: { children: React.ReactNode }) => <section>{children}</section>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardHeader: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  CardTitle: ({ children }: { children: React.ReactNode }) => <h2>{children}</h2>,
}));

vi.mock("@mizan/ui/components/ui/input", () => ({
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
}));

vi.mock("@mizan/ui/components/ui/label", () => ({
  Label: ({ children, ...props }: React.LabelHTMLAttributes<HTMLLabelElement>) => (
    <label {...props}>{children}</label>
  ),
}));

import ReportsPage from "./reports-page";

const generateReportMock = vi.mocked(generateReport);
const exportReportMock = vi.mocked(exportReport);

describe("ReportsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    URL.createObjectURL = vi.fn(() => "blob:report");
    URL.revokeObjectURL = vi.fn();
    HTMLAnchorElement.prototype.click = vi.fn();
  });

  it("renders report type selection and an honest empty preview state", () => {
    renderPage();

    expect(screen.getByText("Report Builder")).toBeInTheDocument();
    expect(screen.getByLabelText("Report type")).toHaveValue("tax_pack");
    expect(screen.getByText(/Generate a report to preview/i)).toBeInTheDocument();
  });

  it("generates a report preview with citation status", async () => {
    generateReportMock.mockResolvedValueOnce(reportRun());
    renderPage();

    fireEvent.click(screen.getByRole("button", { name: /Generate Preview/i }));

    await waitFor(() => {
      expect(generateReportMock).toHaveBeenCalledWith({
        reportType: "tax_pack",
        baseCurrency: "USD",
      });
    });
    expect(await screen.findByText("Dividend")).toBeInTheDocument();
    expect(screen.getByText("citation-1")).toBeInTheDocument();
    expect(screen.getByText("Missing citation")).toBeInTheDocument();
  });

  it("downloads generated report exports", async () => {
    generateReportMock.mockResolvedValueOnce(reportRun());
    exportReportMock.mockResolvedValueOnce({
      fileName: "report.html",
      mimeType: "text/html",
      bytes: [60, 104, 49, 62],
    });
    renderPage();

    fireEvent.click(screen.getByRole("button", { name: /Generate Preview/i }));
    await screen.findByText("Dividend");
    fireEvent.click(screen.getByRole("button", { name: /Export HTML/i }));

    await waitFor(() => {
      expect(exportReportMock).toHaveBeenCalledWith("run-1");
    });
    expect(URL.createObjectURL).toHaveBeenCalled();
    expect(HTMLAnchorElement.prototype.click).toHaveBeenCalled();
  });

  it("generates a monthly wealth letter for the selected month", async () => {
    generateReportMock.mockResolvedValueOnce({
      ...reportRun(),
      reportType: "monthly_wealth_letter",
      sections: [
        {
          id: "section-monthly",
          reportRunId: "run-1",
          title: "Income received",
          sectionOrder: 0,
          metadataJson: null,
          lines: [
            {
              id: "line-monthly",
              sectionId: "section-monthly",
              label: "Dividend and interest received",
              amount: "14.34",
              currency: "USD",
              valueText: null,
              sourceCitationId: null,
              metadataJson: null,
            },
          ],
        },
      ],
    });
    renderPage();

    fireEvent.change(screen.getByLabelText("Report type"), {
      target: { value: "monthly_wealth_letter" },
    });
    fireEvent.change(screen.getByLabelText("Month"), { target: { value: "2026-05" } });
    fireEvent.click(screen.getByRole("button", { name: /Generate Preview/i }));

    await waitFor(() => {
      expect(generateReportMock).toHaveBeenCalledWith({
        reportType: "monthly_wealth_letter",
        baseCurrency: "USD",
        periodMonth: "2026-05",
      });
    });
    expect(await screen.findByText("Dividend and interest received")).toBeInTheDocument();
  });

  it("generates an estate binder with selected sections only", async () => {
    generateReportMock.mockResolvedValueOnce({
      ...reportRun(),
      reportType: "estate_binder",
      disclaimer: "Estate Binder is an organizational checklist only.",
      sections: [
        {
          id: "section-estate",
          reportRunId: "run-1",
          title: "Accounts",
          sectionOrder: 0,
          metadataJson: null,
          lines: [
            {
              id: "line-estate",
              sectionId: "section-estate",
              label: "Taxable",
              amount: null,
              currency: null,
              valueText: "brokerage account in USD",
              sourceCitationId: null,
              metadataJson: null,
            },
          ],
        },
      ],
    });
    renderPage();

    fireEvent.change(screen.getByLabelText("Report type"), {
      target: { value: "estate_binder" },
    });
    fireEvent.click(screen.getByLabelText("Assets"));
    fireEvent.click(screen.getByRole("button", { name: /Generate Preview/i }));

    await waitFor(() => {
      expect(generateReportMock).toHaveBeenCalledWith(
        expect.objectContaining({
          reportType: "estate_binder",
          baseCurrency: "USD",
          includedSections: expect.not.arrayContaining(["assets"]),
        }),
      );
    });
    expect(await screen.findByText("Taxable")).toBeInTheDocument();
    expect(screen.getByText(/organizational checklist/i)).toBeInTheDocument();
  });
});

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ReportsPage />
    </QueryClientProvider>,
  );
}

function reportRun() {
  return {
    id: "run-1",
    reportType: "tax_pack" as const,
    baseCurrency: "USD",
    status: "generated" as const,
    createdAt: "2026-05-16T00:00:00Z",
    completedAt: "2026-05-16T00:00:00Z",
    disclaimer: "Deterministic report preview only.",
    sections: [
      {
        id: "section-1",
        reportRunId: "run-1",
        title: "Tax Pack Report",
        sectionOrder: 0,
        metadataJson: null,
        lines: [
          {
            id: "line-1",
            sectionId: "section-1",
            label: "Dividend",
            amount: "12.34",
            currency: "USD",
            valueText: null,
            sourceCitationId: "citation-1",
            metadataJson: null,
          },
          {
            id: "line-2",
            sectionId: "section-1",
            label: "Interest",
            amount: null,
            currency: null,
            valueText: "Missing source citation",
            sourceCitationId: null,
            metadataJson: null,
          },
        ],
      },
    ],
  };
}
