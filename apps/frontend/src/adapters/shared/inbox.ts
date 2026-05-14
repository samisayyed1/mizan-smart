import { invoke } from "./platform";

export type InboxItemType =
  | "alert"
  | "document"
  | "valuation"
  | "tax"
  | "income"
  | "private_investment"
  | "security"
  | "ai_suggestion"
  | "web_evidence";

export type InboxSeverity = "info" | "warning" | "critical";
export type InboxStatus = "active" | "snoozed" | "dismissed" | "resolved";

export interface InboxItem {
  id: string;
  itemType: InboxItemType;
  title: string;
  description: string;
  severity: InboxSeverity;
  dueDate: string | null;
  sourceEntityType: string;
  sourceEntityId: string;
  actionRoute: string;
  status: InboxStatus;
  createdAt: string;
}

export async function listWealthInboxItems(): Promise<InboxItem[]> {
  return invoke<InboxItem[]>("list_wealth_inbox_items", {});
}
