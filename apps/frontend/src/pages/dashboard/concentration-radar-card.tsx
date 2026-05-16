import type { ConcentrationFragilitySummary } from "@/adapters";
import { Card, CardContent } from "@mizan/ui/components/ui/card";

interface ConcentrationRadarCardProps {
  summary: ConcentrationFragilitySummary;
}

export function ConcentrationRadarCard({ summary }: ConcentrationRadarCardProps) {
  if (summary.emptyState) {
    return null;
  }

  const findings = summary.findings.slice(0, 3);

  return (
    <Card data-testid="concentration-radar-card">
      <CardContent className="space-y-3 p-4">
        <div>
          <p className="text-sm font-medium">Concentration radar</p>
          <p className="text-muted-foreground text-xs">As of {summary.asOfDate}</p>
        </div>
        {findings.length > 0 ? (
          <ul className="space-y-2">
            {findings.map((finding) => (
              <li key={`${finding.dimension}-${finding.label}`} className="text-sm">
                {finding.message}
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-muted-foreground text-sm">
            No concentration thresholds crossed in current deterministic data.
          </p>
        )}
        {summary.taxonomyState === "missing" ? (
          <p className="text-muted-foreground text-xs">
            Sector and country taxonomy exposure is unavailable.
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}

