// Universal Add Asset page (mizan-smart Phase 1 / Prompt 5).
//
// Two-step flow:
//   1. Pick a card. Stays on the same route — no separate wizard URL.
//   2. Fill the simple form (name, currency, initial value, valuation
//      date, plus a small set of class-specific fields). Submit creates
//      the asset and routes to its detail page.
//
// The full discriminated-union request shape lives in
// `apps/frontend/src/adapters/shared/universal-assets.ts` and is the
// wire contract with the Tauri command + Axum endpoint.

import { useState } from "react";
import { Controller, useForm } from "react-hook-form";
import { Link, useNavigate } from "react-router-dom";
import { zodResolver } from "@hookform/resolvers/zod";

import { createUniversalAsset } from "@/adapters";
import {
  Button,
  Icons,
  Input,
  Label,
  Page,
  PageContent,
  PageHeader,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@mizan/ui";
import { Card, CardContent, CardHeader } from "@mizan/ui/components/ui/card";

import { CLASSIFICATION_CARDS, type ClassificationCard } from "./classification-cards";
import { formSchema, todayIso, toRequest, type UniversalAssetFormValues } from "./schemas";

interface UniversalAssetFormProps {
  card: ClassificationCard;
  onBack: () => void;
  onSaved: (assetId: string) => void;
}

function UniversalAssetForm({ card, onBack, onSaved }: UniversalAssetFormProps) {
  const {
    register,
    handleSubmit,
    control,
    watch,
    formState: { errors, isSubmitting },
    setError,
  } = useForm<UniversalAssetFormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      classification: card.defaultClassification,
      currency: "USD",
      initialValueDate: todayIso(),
      // Pre-fill subtype-specific defaults so the discriminated request
      // is always valid even if the user doesn't touch the dropdowns.
      fixedIncomeSubtype:
        card.defaultClassification === "fixed_income" ? "bond" : undefined,
      commodityRequestType:
        card.defaultClassification === "gold"
          ? "gold"
          : card.defaultClassification === "silver"
            ? "silver"
            : card.defaultClassification === "commodity"
              ? "other_commodity"
              : undefined,
      liabilityType:
        card.defaultClassification === "liability" ? "mortgage" : undefined,
    },
  });

  const currentClassification = watch("classification");

  const onSubmit = handleSubmit(async (values) => {
    try {
      const request = toRequest(values);
      const result = await createUniversalAsset(request);
      onSaved(result.assetId);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setError("root", { message });
    }
  });

  return (
    <form onSubmit={onSubmit} className="space-y-6" data-testid="universal-add-asset-form">
      <header className="flex items-start justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold">{card.title}</h2>
          <p className="text-muted-foreground text-sm">{card.description}</p>
        </div>
        <Button type="button" variant="ghost" onClick={onBack} data-testid="add-asset-back">
          <Icons.ArrowLeft className="size-4" aria-hidden="true" />
          Change type
        </Button>
      </header>

      {card.subtypes && card.subtypes.length > 1 && (
        <div className="space-y-2">
          <Label htmlFor="classification">What kind?</Label>
          <Controller
            control={control}
            name="classification"
            render={({ field }) => (
              <Select value={field.value} onValueChange={field.onChange}>
                <SelectTrigger id="classification" data-testid="subtype-select">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {card.subtypes!.map((s) => (
                    <SelectItem key={s.value} value={s.value}>
                      {s.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          />
        </div>
      )}

      <div className="space-y-2">
        <Label htmlFor="name">Name</Label>
        <Input id="name" {...register("name")} data-testid="name-input" />
        {errors.name && <ErrorText>{errors.name.message}</ErrorText>}
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="space-y-2">
          <Label htmlFor="currency">Currency</Label>
          <Input
            id="currency"
            maxLength={3}
            placeholder="USD"
            {...register("currency", {
              onBlur: (e) => {
                e.target.value = e.target.value.toUpperCase();
              },
            })}
            data-testid="currency-input"
          />
          {errors.currency && <ErrorText>{errors.currency.message}</ErrorText>}
        </div>
        <div className="space-y-2">
          <Label htmlFor="initialValueDate">When was this value last checked?</Label>
          <Input
            id="initialValueDate"
            type="date"
            {...register("initialValueDate")}
            data-testid="initial-value-date-input"
          />
          {errors.initialValueDate && <ErrorText>{errors.initialValueDate.message}</ErrorText>}
        </div>
      </div>

      <div className="space-y-2">
        <Label htmlFor="initialValue">Current value</Label>
        <Input
          id="initialValue"
          inputMode="decimal"
          placeholder="0"
          {...register("initialValue")}
          data-testid="initial-value-input"
        />
        {errors.initialValue && <ErrorText>{errors.initialValue.message}</ErrorText>}
      </div>

      {/* Per-classification optional details. Every field is optional —
          the universal flow stays short. Per-class detail pages
          (built in Phase 3) deepen these later. */}
      {(currentClassification === "public_equity" ||
        currentClassification === "etf" ||
        currentClassification === "mutual_fund") && (
        <OptionalText
          id="isin"
          label="ISIN (optional)"
          register={register("isin")}
          testId="isin-input"
        />
      )}

      {currentClassification === "fixed_income" && (
        <>
          <div className="space-y-2">
            <Label htmlFor="fixedIncomeSubtype">Instrument type</Label>
            <Controller
              control={control}
              name="fixedIncomeSubtype"
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger id="fixedIncomeSubtype" data-testid="fi-subtype-select">
                    <SelectValue placeholder="Select an instrument type" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="bond">Bond</SelectItem>
                    <SelectItem value="treasury_bill">Treasury bill</SelectItem>
                    <SelectItem value="cd">Certificate of deposit</SelectItem>
                    <SelectItem value="structured_note">Structured note</SelectItem>
                    <SelectItem value="other">Other</SelectItem>
                  </SelectContent>
                </Select>
              )}
            />
          </div>
          <OptionalText
            id="issuer"
            label="Issuer (optional)"
            register={register("issuer")}
            testId="issuer-input"
          />
          <OptionalText
            id="maturityDate"
            label="Maturity date (optional)"
            type="date"
            register={register("maturityDate")}
            testId="maturity-date-input"
          />
        </>
      )}

      {(currentClassification === "sukuk" || currentClassification === "fixed_deposit") && (
        <>
          <OptionalText
            id="issuer"
            label="Issuer (optional)"
            register={register("issuer")}
            testId="issuer-input"
          />
          <OptionalText
            id="maturityDate"
            label="Maturity date (optional)"
            type="date"
            register={register("maturityDate")}
            testId="maturity-date-input"
          />
        </>
      )}

      {currentClassification === "real_estate" && (
        <>
          <OptionalText
            id="propertyType"
            label="Property type (optional)"
            placeholder="apartment, house, land, …"
            register={register("propertyType")}
            testId="property-type-input"
          />
          <OptionalText
            id="addressApproximate"
            label="Approximate location (optional)"
            placeholder="city or neighbourhood"
            register={register("addressApproximate")}
            testId="address-input"
          />
        </>
      )}

      {(currentClassification === "private_equity" ||
        currentClassification === "private_credit" ||
        currentClassification === "hedge_fund" ||
        currentClassification === "venture_capital") && (
        <OptionalText
          id="manager"
          label="Manager (optional)"
          register={register("manager")}
          testId="manager-input"
        />
      )}

      {currentClassification === "crypto" && (
        <OptionalText
          id="symbol"
          label="Symbol (optional)"
          placeholder="BTC, ETH, …"
          register={register("symbol")}
          testId="crypto-symbol-input"
        />
      )}

      {(currentClassification === "gold" ||
        currentClassification === "silver" ||
        currentClassification === "commodity") && (
        <>
          {currentClassification === "commodity" && (
            <div className="space-y-2">
              <Label htmlFor="commodityRequestType">Which metal / commodity?</Label>
              <Controller
                control={control}
                name="commodityRequestType"
                render={({ field }) => (
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger
                      id="commodityRequestType"
                      data-testid="commodity-type-select"
                    >
                      <SelectValue placeholder="Select" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="platinum">Platinum</SelectItem>
                      <SelectItem value="palladium">Palladium</SelectItem>
                      <SelectItem value="other_commodity">Other</SelectItem>
                    </SelectContent>
                  </Select>
                )}
              />
            </div>
          )}
          <div className="grid gap-4 md:grid-cols-3">
            <OptionalText
              id="weightValue"
              label="Weight (optional)"
              inputMode="decimal"
              placeholder="0"
              register={register("weightValue")}
              testId="weight-value-input"
            />
            <OptionalText
              id="weightUnit"
              label="Unit"
              placeholder="g, oz, kg, ton"
              register={register("weightUnit")}
              testId="weight-unit-input"
            />
            <OptionalText
              id="purity"
              label="Purity"
              placeholder="999, 24k, …"
              register={register("purity")}
              testId="purity-input"
            />
          </div>
        </>
      )}

      {(currentClassification === "insurance" ||
        currentClassification === "ulip" ||
        currentClassification === "pension") && (
        <OptionalText
          id="provider"
          label="Provider (optional)"
          register={register("provider")}
          testId="provider-input"
        />
      )}

      {currentClassification === "business_ownership" && (
        <>
          <OptionalText
            id="businessName"
            label="Business name (optional)"
            register={register("businessName")}
            testId="business-name-input"
          />
          <OptionalText
            id="ownershipPercent"
            label="Ownership %"
            inputMode="decimal"
            placeholder="0 to 100"
            register={register("ownershipPercent")}
            testId="ownership-percent-input"
          />
        </>
      )}

      {currentClassification === "collectible" && (
        <>
          <OptionalText
            id="collectibleType"
            label="Type (optional)"
            placeholder="watch, art, wine, …"
            register={register("collectibleType")}
            testId="collectible-type-input"
          />
          <OptionalText
            id="maker"
            label="Maker (optional)"
            register={register("maker")}
            testId="maker-input"
          />
        </>
      )}

      {currentClassification === "liability" && (
        <>
          <div className="space-y-2">
            <Label htmlFor="liabilityType">Liability type</Label>
            <Controller
              control={control}
              name="liabilityType"
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger id="liabilityType" data-testid="liability-type-select">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="mortgage">Mortgage</SelectItem>
                    <SelectItem value="loan">Loan</SelectItem>
                    <SelectItem value="credit_card">Credit card</SelectItem>
                    <SelectItem value="line_of_credit">Line of credit</SelectItem>
                    <SelectItem value="other_liability">Other</SelectItem>
                  </SelectContent>
                </Select>
              )}
            />
          </div>
          <OptionalText
            id="lender"
            label="Lender (optional)"
            register={register("lender")}
            testId="lender-input"
          />
        </>
      )}

      <div className="space-y-2">
        <Label htmlFor="notes">Notes (optional)</Label>
        <Input id="notes" {...register("notes")} data-testid="notes-input" />
      </div>

      {errors.root?.message && (
        <p className="text-destructive text-sm" role="alert" data-testid="form-error">
          {errors.root.message}
        </p>
      )}

      <div className="flex items-center justify-end gap-2">
        <Button type="button" variant="ghost" asChild>
          <Link to="/holdings">Cancel</Link>
        </Button>
        <Button type="submit" disabled={isSubmitting} data-testid="save-button">
          {isSubmitting ? "Saving…" : "Save asset"}
        </Button>
      </div>
    </form>
  );
}

function ErrorText({ children }: { children: React.ReactNode }) {
  return (
    <p className="text-destructive text-xs" role="alert">
      {children}
    </p>
  );
}

interface OptionalTextProps {
  id: string;
  label: string;
  type?: string;
  placeholder?: string;
  inputMode?: React.HTMLAttributes<HTMLInputElement>["inputMode"];
  // RHF's register() returns a ref + handlers — pass it straight through.
  register: ReturnType<ReturnType<typeof useForm<UniversalAssetFormValues>>["register"]>;
  testId: string;
}

function OptionalText({
  id,
  label,
  type,
  placeholder,
  inputMode,
  register,
  testId,
}: OptionalTextProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor={id}>{label}</Label>
      <Input
        id={id}
        type={type}
        placeholder={placeholder}
        inputMode={inputMode}
        {...register}
        data-testid={testId}
      />
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────
// Page
// ─────────────────────────────────────────────────────────────────────

export default function UniversalAddAssetPage() {
  const [selectedCard, setSelectedCard] = useState<ClassificationCard | null>(null);
  const navigate = useNavigate();

  return (
    <Page>
      <PageHeader
        heading="Add an asset"
        text="Stocks, property, gold, private investments, and more."
      />
      <PageContent>
        {selectedCard === null ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3" data-testid="card-grid">
            {CLASSIFICATION_CARDS.map((card) => {
              const Icon = Icons[card.icon];
              return (
                <button
                  key={card.id}
                  type="button"
                  onClick={() => setSelectedCard(card)}
                  data-testid={`card-${card.id}`}
                  className="hover:border-foreground/30 group flex flex-col items-start gap-3 rounded-lg border bg-card px-5 py-5 text-left transition-colors"
                >
                  <span className="bg-muted text-foreground/80 flex size-10 items-center justify-center rounded-md">
                    <Icon className="size-5" aria-hidden="true" />
                  </span>
                  <span className="block text-base font-semibold">{card.title}</span>
                  <span className="text-muted-foreground block text-sm">
                    {card.description}
                  </span>
                </button>
              );
            })}
          </div>
        ) : (
          <Card>
            <CardHeader className="pb-3">
              <h2 className="sr-only">Asset details</h2>
            </CardHeader>
            <CardContent>
              <UniversalAssetForm
                card={selectedCard}
                onBack={() => setSelectedCard(null)}
                onSaved={(assetId) => navigate(`/holdings/${assetId}`)}
              />
            </CardContent>
          </Card>
        )}
      </PageContent>
    </Page>
  );
}
