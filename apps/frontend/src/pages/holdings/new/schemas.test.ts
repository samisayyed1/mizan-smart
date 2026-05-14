import { describe, expect, it } from "vitest";

import { formSchema, todayIso, toRequest, type UniversalAssetFormValues } from "./schemas";

function baseValues(overrides: Partial<UniversalAssetFormValues> = {}): UniversalAssetFormValues {
  return {
    classification: "real_estate",
    name: "Primary residence",
    currency: "USD",
    initialValue: "750000",
    initialValueDate: "2026-05-14",
    notes: undefined,
    isin: undefined,
    fixedIncomeSubtype: undefined,
    issuer: undefined,
    maturityDate: undefined,
    propertyType: undefined,
    addressApproximate: undefined,
    manager: undefined,
    symbol: undefined,
    commodityRequestType: undefined,
    weightValue: undefined,
    weightUnit: undefined,
    purity: undefined,
    provider: undefined,
    businessName: undefined,
    ownershipPercent: undefined,
    collectibleType: undefined,
    maker: undefined,
    liabilityType: undefined,
    lender: undefined,
    ...overrides,
  };
}

describe("formSchema", () => {
  it("accepts a minimal well-formed real_estate submission", () => {
    const result = formSchema.safeParse(baseValues());
    expect(result.success).toBe(true);
  });

  it("rejects blank name", () => {
    const result = formSchema.safeParse(baseValues({ name: "   " }));
    expect(result.success).toBe(false);
  });

  it("rejects lowercase currency", () => {
    const result = formSchema.safeParse(baseValues({ currency: "usd" }));
    expect(result.success).toBe(false);
  });

  it("rejects non-decimal initial value", () => {
    const result = formSchema.safeParse(baseValues({ initialValue: "ten thousand" }));
    expect(result.success).toBe(false);
  });

  it("rejects malformed date", () => {
    const result = formSchema.safeParse(baseValues({ initialValueDate: "14-05-2026" }));
    expect(result.success).toBe(false);
  });

  it("accepts optional fields left blank", () => {
    const result = formSchema.safeParse(
      baseValues({
        weightValue: "",
        maturityDate: "",
        notes: undefined,
      }),
    );
    expect(result.success).toBe(true);
  });
});

describe("toRequest", () => {
  it("builds the correct discriminator for real_estate", () => {
    const req = toRequest(
      baseValues({
        classification: "real_estate",
        propertyType: "apartment",
        addressApproximate: "London",
      }),
    );
    if (req.classification !== "real_estate") throw new Error("wrong variant");
    expect(req.name).toBe("Primary residence");
    expect(req.currency).toBe("USD");
    expect(req.initialValue).toBe("750000");
    expect(req.propertyType).toBe("apartment");
    expect(req.addressApproximate).toBe("London");
  });

  it("threads fixed_income subtype + maturity into the request", () => {
    const req = toRequest(
      baseValues({
        classification: "fixed_income",
        name: "T-Bill",
        initialValue: "10000",
        fixedIncomeSubtype: "treasury_bill",
        issuer: "US Treasury",
        maturityDate: "2026-08-14",
      }),
    );
    if (req.classification !== "fixed_income") throw new Error("wrong variant");
    expect(req.instrumentSubtype).toBe("treasury_bill");
    expect(req.issuer).toBe("US Treasury");
    expect(req.maturityDate).toBe("2026-08-14");
  });

  it("uppercases the currency", () => {
    const req = toRequest(baseValues({ classification: "cash", currency: "gbp" }));
    expect(req.currency).toBe("GBP");
  });

  it("nulls-out empty optional strings", () => {
    const req = toRequest(
      baseValues({
        classification: "crypto",
        symbol: "   ",
      }),
    );
    if (req.classification !== "crypto") throw new Error("wrong variant");
    expect(req.symbol).toBeNull();
  });

  it("defaults liability type when unset", () => {
    const req = toRequest(
      baseValues({
        classification: "liability",
        name: "Mortgage",
        initialValue: "400000",
      }),
    );
    if (req.classification !== "liability") throw new Error("wrong variant");
    // Without an explicit dropdown value we default to other_liability
    // so the wire request stays valid.
    expect(req.liabilityType).toBe("other_liability");
  });

  it("forwards commodity weight + purity for gold", () => {
    const req = toRequest(
      baseValues({
        classification: "gold",
        weightValue: "10",
        weightUnit: "oz",
        purity: "999",
      }),
    );
    if (req.classification !== "gold") throw new Error("wrong variant");
    expect(req.weightValue).toBe("10");
    expect(req.weightUnit).toBe("oz");
    expect(req.purity).toBe("999");
  });
});

describe("todayIso", () => {
  it("returns a yyyy-mm-dd string", () => {
    expect(todayIso()).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });
});
