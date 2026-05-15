import { listShariahScreeningProfiles } from "@/adapters";
import { ShariahStatusBadge } from "@/components/shariah-status-badge";
import { useSettingsContext } from "@/lib/settings-provider";
import { Page, PageContent, PageHeader } from "@mizan/ui";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { useQuery } from "@tanstack/react-query";

export default function ShariahScreeningPage() {
  const { settings } = useSettingsContext();
  const enabled = settings?.shariahModeEnabled === true;
  const profilesQuery = useQuery({
    queryKey: ["shariah-screening-profiles"],
    queryFn: listShariahScreeningProfiles,
    enabled,
  });

  if (!enabled) {
    return (
      <Page>
        <PageHeader heading="Screening" text="Enable optional Islamic finance tools in Settings." />
        <PageContent>
          <Card>
            <CardContent className="text-muted-foreground p-6 text-sm">
              Islamic finance tools are disabled for this profile.
            </CardContent>
          </Card>
        </PageContent>
      </Page>
    );
  }

  const defaultProfile = profilesQuery.data?.find((profile) => profile.isDefault);

  return (
    <Page>
      <PageHeader heading="Screening" text="Review assets with optional Islamic finance criteria." />
      <PageContent className="space-y-4">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Default Screening Profile</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm">
            {defaultProfile ? (
              <div className="grid gap-2 sm:grid-cols-3">
                <Metric label="Debt" value={`< ${defaultProfile.debtThreshold}`} />
                <Metric label="Liquid assets" value={`< ${defaultProfile.liquidAssetsThreshold}`} />
                <Metric label="Impure income" value={`< ${defaultProfile.impureIncomeThreshold}`} />
              </div>
            ) : (
              <p className="text-muted-foreground">Screening profile is not available.</p>
            )}
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Insufficient data status:</span>
              <ShariahStatusBadge status="unknown" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Zakat</CardTitle>
          </CardHeader>
          <CardContent className="text-muted-foreground text-sm">
            Zakat calculation will use approved asset data in a later step. No values are estimated
            here.
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Purification</CardTitle>
          </CardHeader>
          <CardContent className="text-muted-foreground text-sm">
            Dividend purification will require reviewed income facts before any amount is shown.
          </CardContent>
        </Card>
      </PageContent>
    </Page>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border p-3">
      <div className="text-muted-foreground text-xs">{label}</div>
      <div className="text-base font-medium">{value}</div>
    </div>
  );
}
