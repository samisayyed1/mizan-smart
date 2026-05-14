// Card chooser for the universal Add Asset flow.
// Phase 1 / Prompt 5 of docs/mizan-smart-plan/PLAN.md.
//
// Ten senior-friendly cards, each anchored on a default classification.
// Cards with multiple possible subtypes carry the dropdown in the form
// step rather than splitting into separate cards — fewer choices on
// the chooser screen is the whole point.

import type { UniversalAssetClassification } from "@/adapters";
import { Icons } from "@mizan/ui";

export type ClassificationCardId =
  | "stock"
  | "bond"
  | "fixed_deposit_cash"
  | "property"
  | "private_investment"
  | "commodity"
  | "crypto"
  | "insurance"
  | "business_other"
  | "liability";

export interface ClassificationCard {
  id: ClassificationCardId;
  title: string;
  description: string;
  icon: keyof typeof Icons;
  defaultClassification: UniversalAssetClassification;
  /**
   * Subtype choices presented as a dropdown on the form step.
   * Each value is the wire `classification` to send.
   */
  subtypes?: { value: UniversalAssetClassification; label: string }[];
}

export const CLASSIFICATION_CARDS: ClassificationCard[] = [
  {
    id: "stock",
    title: "Stock, ETF, or fund",
    description: "Public-market shares, exchange-traded funds, mutual funds.",
    icon: "TrendingUp",
    defaultClassification: "public_equity",
    subtypes: [
      { value: "public_equity", label: "Stock (single company)" },
      { value: "etf", label: "ETF" },
      { value: "mutual_fund", label: "Mutual fund" },
    ],
  },
  {
    id: "bond",
    title: "Bond or sukuk",
    description: "Corporate, sovereign, or Islamic fixed-income instruments.",
    icon: "FileText",
    defaultClassification: "fixed_income",
    subtypes: [
      { value: "fixed_income", label: "Bond (conventional)" },
      { value: "sukuk", label: "Sukuk (Islamic)" },
    ],
  },
  {
    id: "fixed_deposit_cash",
    title: "Fixed deposit or cash",
    description: "Term deposits, CDs, or plain cash balances.",
    icon: "Wallet",
    defaultClassification: "fixed_deposit",
    subtypes: [
      { value: "fixed_deposit", label: "Fixed deposit / CD" },
      { value: "cash", label: "Cash balance" },
    ],
  },
  {
    id: "property",
    title: "Property",
    description: "Residential, commercial, or land.",
    icon: "Building",
    defaultClassification: "real_estate",
  },
  {
    id: "private_investment",
    title: "Private investment",
    description: "Private equity, private credit, hedge fund, or VC.",
    icon: "Briefcase",
    defaultClassification: "private_equity",
    subtypes: [
      { value: "private_equity", label: "Private equity" },
      { value: "private_credit", label: "Private credit" },
      { value: "hedge_fund", label: "Hedge fund" },
      { value: "venture_capital", label: "Venture capital" },
    ],
  },
  {
    id: "commodity",
    title: "Gold or commodity",
    description: "Physical gold, silver, platinum, palladium, or other.",
    icon: "Gem",
    defaultClassification: "gold",
    subtypes: [
      { value: "gold", label: "Gold" },
      { value: "silver", label: "Silver" },
      { value: "commodity", label: "Other commodity" },
    ],
  },
  {
    id: "crypto",
    title: "Crypto",
    description: "Cryptocurrencies and digital assets.",
    icon: "Bitcoin",
    defaultClassification: "crypto",
  },
  {
    id: "insurance",
    title: "Insurance or pension",
    description: "Insurance policies, ULIPs, or pension accounts.",
    icon: "Shield",
    defaultClassification: "insurance",
    subtypes: [
      { value: "insurance", label: "Insurance policy" },
      { value: "ulip", label: "ULIP" },
      { value: "pension", label: "Pension" },
    ],
  },
  {
    id: "business_other",
    title: "Business or other",
    description: "Business ownership, collectibles, or custom asset.",
    icon: "Store",
    defaultClassification: "business_ownership",
    subtypes: [
      { value: "business_ownership", label: "Business ownership" },
      { value: "collectible", label: "Collectible (art, watch, …)" },
      { value: "custom", label: "Custom / other" },
    ],
  },
  {
    id: "liability",
    title: "Liability",
    description: "Mortgages, loans, credit cards, or other debt.",
    icon: "ArrowDown",
    defaultClassification: "liability",
  },
];
