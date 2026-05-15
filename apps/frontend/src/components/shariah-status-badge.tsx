import type { ShariahScreeningStatus } from "@/adapters";
import { Badge } from "@mizan/ui/components/ui/badge";

const STATUS_LABELS: Record<ShariahScreeningStatus, string> = {
  compliant: "Compliant",
  non_compliant: "Non-compliant",
  questionable: "Questionable",
  unknown: "Unknown",
  needs_review: "Needs review",
};

const STATUS_CLASS_NAMES: Record<ShariahScreeningStatus, string> = {
  compliant: "border-emerald-200 bg-emerald-50 text-emerald-700",
  non_compliant: "border-rose-200 bg-rose-50 text-rose-700",
  questionable: "border-amber-200 bg-amber-50 text-amber-700",
  unknown: "border-slate-200 bg-slate-50 text-slate-700",
  needs_review: "border-blue-200 bg-blue-50 text-blue-700",
};

export function ShariahStatusBadge({ status }: { status: ShariahScreeningStatus }) {
  return (
    <Badge variant="outline" className={STATUS_CLASS_NAMES[status]}>
      {STATUS_LABELS[status]}
    </Badge>
  );
}
