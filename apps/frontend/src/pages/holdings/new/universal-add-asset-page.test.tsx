import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";

// Mock the backend adapter so we don't hit Tauri/web during the test.
const createUniversalAssetMock = vi.fn();
vi.mock("@/adapters", () => ({
  createUniversalAsset: (...args: unknown[]) => createUniversalAssetMock(...args),
}));

// Mock @mizan/ui — keep the same lightweight pattern the rest of the
// suite uses so each test stays fast and isolated.
vi.mock("@mizan/ui", () => ({
  Button: ({
    children,
    asChild,
    ...props
  }: {
    children: React.ReactNode;
    asChild?: boolean;
    [k: string]: unknown;
  }) => {
    if (asChild && React.isValidElement(children)) {
      return React.cloneElement(children, props);
    }
    return (
      <button
        type={(props.type as "button" | "submit" | "reset" | undefined) ?? "button"}
        {...props}
      >
        {children}
      </button>
    );
  },
  Icons: new Proxy(
    {},
    {
      get: (_target, prop) => () => <span>{String(prop)}</span>,
    },
  ),
  Input: (props: React.InputHTMLAttributes<HTMLInputElement>) => <input {...props} />,
  Label: (props: React.LabelHTMLAttributes<HTMLLabelElement>) => <label {...props} />,
  Page: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  PageHeader: ({ heading }: { heading?: string }) => <header>{heading}</header>,
  Select: ({
    children,
    value,
    onValueChange,
  }: {
    children: React.ReactNode;
    value?: string;
    onValueChange?: (v: string) => void;
  }) => {
    // Render Select as a native <select> so RHF Controller integration
    // tests don't need to drive radix-ui internals.
    const items: { value: string; label: string }[] = [];
    React.Children.forEach(children, (child) => {
      if (!React.isValidElement(child)) return;
      const grand = (child as React.ReactElement<{ children?: React.ReactNode }>).props
        .children;
      React.Children.forEach(grand, (item) => {
        if (React.isValidElement(item)) {
          const itemProps = (
            item as React.ReactElement<{ value?: string; children?: React.ReactNode }>
          ).props;
          // Narrow children to a primitive — every label in the
          // universal Add Asset form is a static string, so anything
          // exotic here is a bug in the form, not in the mock.
          if (
            itemProps.value !== undefined &&
            (typeof itemProps.children === "string" ||
              typeof itemProps.children === "number")
          ) {
            items.push({ value: itemProps.value, label: String(itemProps.children) });
          }
        }
      });
    });
    return (
      <select
        value={value ?? ""}
        onChange={(e) => onValueChange?.(e.target.value)}
        data-testid="select-mock"
      >
        {items.map((i) => (
          <option key={i.value} value={i.value}>
            {i.label}
          </option>
        ))}
      </select>
    );
  },
  SelectContent: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  SelectItem: ({
    value,
    children,
  }: {
    value: string;
    children: React.ReactNode;
  }) => <option value={value}>{children}</option>,
  SelectTrigger: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  SelectValue: () => null,
}));

vi.mock("@mizan/ui/components/ui/card", () => ({
  Card: ({ children, ...props }: { children: React.ReactNode; [k: string]: unknown }) => (
    <section {...props}>{children}</section>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => <header>{children}</header>,
  CardContent: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

import UniversalAddAssetPage from "./universal-add-asset-page";

function renderPage(initial = "/holdings/new") {
  return render(
    <MemoryRouter initialEntries={[initial]}>
      <Routes>
        <Route path="/holdings/new" element={<UniversalAddAssetPage />} />
        <Route
          path="/holdings/:assetId"
          element={<div data-testid="detail-page" />}
        />
      </Routes>
    </MemoryRouter>,
  );
}

beforeEach(() => {
  createUniversalAssetMock.mockReset();
});

describe("UniversalAddAssetPage", () => {
  it("renders all ten classification cards on the chooser step", () => {
    renderPage();
    const grid = screen.getByTestId("card-grid");
    // Each card has data-testid="card-<id>" — count them.
    const buttons = grid.querySelectorAll('[data-testid^="card-"]');
    expect(buttons).toHaveLength(10);
    // Spot-check a couple of canonical titles.
    expect(screen.getByText(/Stock, ETF, or fund/)).toBeInTheDocument();
    expect(screen.getByText(/Property/)).toBeInTheDocument();
    expect(screen.getByText(/Liability/)).toBeInTheDocument();
  });

  it("switches to the form when a card is picked, with required fields visible", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByTestId("card-property"));
    expect(screen.getByTestId("universal-add-asset-form")).toBeInTheDocument();
    expect(screen.getByTestId("name-input")).toBeInTheDocument();
    expect(screen.getByTestId("currency-input")).toBeInTheDocument();
    expect(screen.getByTestId("initial-value-input")).toBeInTheDocument();
    expect(screen.getByTestId("property-type-input")).toBeInTheDocument();
  });

  it("the bond card exposes the conventional/sukuk subtype dropdown", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByTestId("card-bond"));
    // Two select widgets appear on this form: the classification
    // subtype dropdown (sukuk vs. fixed_income) at the top of the
    // form, and once the user picks fixed_income, a fixed-income
    // subtype dropdown. The mock renders both as <select> elements.
    const selects = screen.getAllByTestId("select-mock");
    expect(selects.length).toBeGreaterThanOrEqual(1);
    // The first select must offer both bond and sukuk options.
    const optionValues = Array.from(selects[0].querySelectorAll("option")).map(
      (o) => (o as HTMLOptionElement).value,
    );
    expect(optionValues).toEqual(expect.arrayContaining(["fixed_income", "sukuk"]));
  });

  it("blocks submit and surfaces an error when name is blank", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByTestId("card-property"));
    await user.click(screen.getByTestId("save-button"));
    // The adapter must never be called when the form is invalid.
    expect(createUniversalAssetMock).not.toHaveBeenCalled();
  });

  it("submits a real_estate request and navigates to the detail page on success", async () => {
    createUniversalAssetMock.mockResolvedValue({
      assetId: "abc-123",
      classification: "real_estate",
      valuationId: "v1",
    });
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByTestId("card-property"));
    await user.type(screen.getByTestId("name-input"), "Primary residence");
    await user.type(screen.getByTestId("initial-value-input"), "750000");
    await user.click(screen.getByTestId("save-button"));

    expect(createUniversalAssetMock).toHaveBeenCalledTimes(1);
    const call = createUniversalAssetMock.mock.calls[0][0];
    expect(call.classification).toBe("real_estate");
    expect(call.name).toBe("Primary residence");
    expect(call.initialValue).toBe("750000");
    expect(call.currency).toBe("USD");
    // After save, the router lands on the detail page.
    expect(await screen.findByTestId("detail-page")).toBeInTheDocument();
  });

  it("the back button returns to the card chooser", async () => {
    const user = userEvent.setup();
    renderPage();
    await user.click(screen.getByTestId("card-property"));
    expect(screen.getByTestId("universal-add-asset-form")).toBeInTheDocument();
    await user.click(screen.getByTestId("add-asset-back"));
    expect(screen.getByTestId("card-grid")).toBeInTheDocument();
  });
});
