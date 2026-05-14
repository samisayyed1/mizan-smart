import { invoke } from "./platform";

export type CapitalCallStatus = "expected" | "due" | "paid" | "cancelled";

export interface PrivateInvestment {
  assetId: string;
  manager: string;
  strategy: string;
  vintageYear?: number | null;
  commitmentAmount: string;
  commitmentCurrency: string;
  inceptionDate?: string | null;
  notes?: string | null;
}

export interface PrivateInvestmentValuation {
  id: string;
  assetId: string;
  valuationDate: string;
  nav: string;
  currency: string;
  sourceCitationId?: string | null;
}

export interface CapitalCall {
  id: string;
  assetId: string;
  noticeDate: string;
  dueDate: string;
  amount: string;
  currency: string;
  status: CapitalCallStatus;
  sourceCitationId?: string | null;
  notes?: string | null;
}

export interface PrivateDistribution {
  id: string;
  assetId: string;
  distributionDate: string;
  amount: string;
  currency: string;
  recallable: boolean;
  sourceCitationId?: string | null;
  notes?: string | null;
}

export interface PrivateInvestmentSummary {
  investment: PrivateInvestment;
  commitment: string;
  paidInCapital: string;
  unfundedCommitment: string;
  totalDistributions: string;
  recallableDistributions: string;
  currentNav: string;
  dpi?: string | null;
  rvpi?: string | null;
  tvpi?: string | null;
  moic?: string | null;
  warnings: string[];
}

export interface UpsertPrivateInvestmentRequest {
  assetId: string;
  manager: string;
  strategy: string;
  vintageYear?: number | null;
  commitmentAmount: string;
  commitmentCurrency: string;
  inceptionDate?: string | null;
  notes?: string | null;
}

export interface CreatePrivateInvestmentValuationRequest {
  assetId: string;
  valuationDate: string;
  nav: string;
  currency: string;
  sourceCitationId?: string | null;
}

export interface CreateCapitalCallRequest {
  assetId: string;
  noticeDate: string;
  dueDate: string;
  amount: string;
  currency: string;
  status: CapitalCallStatus;
  sourceCitationId?: string | null;
  notes?: string | null;
}

export interface UpdateCapitalCallStatusRequest {
  id: string;
  status: CapitalCallStatus;
}

export interface CreatePrivateDistributionRequest {
  assetId: string;
  distributionDate: string;
  amount: string;
  currency: string;
  recallable: boolean;
  sourceCitationId?: string | null;
  notes?: string | null;
}

export function upsertPrivateInvestment(
  request: UpsertPrivateInvestmentRequest,
): Promise<PrivateInvestment> {
  return invoke<PrivateInvestment>("upsert_private_investment", { request });
}

export function getPrivateInvestment(assetId: string): Promise<PrivateInvestment | null> {
  return invoke<PrivateInvestment | null>("get_private_investment", { assetId });
}

export function deletePrivateInvestment(assetId: string): Promise<void> {
  return invoke<void>("delete_private_investment", { assetId });
}

export function addPrivateInvestmentValuation(
  request: CreatePrivateInvestmentValuationRequest,
): Promise<PrivateInvestmentValuation> {
  return invoke<PrivateInvestmentValuation>("add_private_investment_valuation", { request });
}

export function addCapitalCall(request: CreateCapitalCallRequest): Promise<CapitalCall> {
  return invoke<CapitalCall>("add_capital_call", { request });
}

export function updateCapitalCallStatus(
  request: UpdateCapitalCallStatusRequest,
): Promise<CapitalCall> {
  return invoke<CapitalCall>("update_capital_call_status", { request });
}

export function addPrivateDistribution(
  request: CreatePrivateDistributionRequest,
): Promise<PrivateDistribution> {
  return invoke<PrivateDistribution>("add_private_distribution", { request });
}

export function getPrivateInvestmentSummary(
  assetId: string,
): Promise<PrivateInvestmentSummary | null> {
  return invoke<PrivateInvestmentSummary | null>("get_private_investment_summary", { assetId });
}
