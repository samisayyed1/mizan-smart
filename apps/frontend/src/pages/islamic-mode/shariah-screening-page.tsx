import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";

import {
  getAssetShariahScreening,
  listShariahScreeningAudit,
  listShariahScreeningProfiles,
  upsertAssetShariahScreening,
  type AssetShariahScreening,
  type ShariahScreeningAuditEntry,
  type ShariahScreeningProfile,
  type ShariahScreeningStatus,
} from "@/adapters";
import { ShariahStatusBadge } from "@/components/shariah-status-badge";
import { useSettingsContext } from "@/lib/settings-provider";
import { Page, PageContent, PageHeader } from "@mizan/ui";
import { Button } from "@mizan/ui/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { Input } from "@mizan/ui/components/ui/input";
import { Label } from "@mizan/ui/components/ui/label";
import { Textarea } from "@mizan/ui/components/ui/textarea";

const decimalRatio = z
  .string()
  .trim()
  .min(1, "Required")
  .regex(/^(0(\.\d+)?|1(\.0+)?)$/, "Use a decimal ratio from 0 to 1");

const optionalText = z.string().trim();

const screeningSchema = z
  .object({
    assetId: z.string().trim().min(1, "Asset ID is required"),
    profileId: z.string().trim().min(1, "Profile is required"),
    debtRatio: decimalRatio,
    liquidAssetsRatio: decimalRatio,
    impureIncomeRatio: decimalRatio,
    sourceCitationId: optionalText,
    notes: optionalText,
    manualOverrideStatus: z.enum([
      "none",
      "compliant",
      "non_compliant",
      "questionable",
      "unknown",
      "needs_review",
    ]),
    manualOverrideReason: optionalText,
  })
  .refine(
    (value) =>
      value.manualOverrideStatus === "none" || value.manualOverrideReason.trim().length > 0,
    {
      path: ["manualOverrideReason"],
      message: "Manual override requires a reason",
    },
  );

type ScreeningFormValues = z.infer<typeof screeningSchema>;

export default function ShariahScreeningPage() {
  const { settings } = useSettingsContext();
  const enabled = settings?.shariahModeEnabled === true;
  const queryClient = useQueryClient();
  const [reviewKey, setReviewKey] = useState<{ assetId: string; profileId: string } | null>(null);

  const profilesQuery = useQuery({
    queryKey: ["shariah-screening-profiles"],
    queryFn: listShariahScreeningProfiles,
    enabled,
  });
  const defaultProfile = profilesQuery.data?.find((profile) => profile.isDefault);

  const form = useForm<ScreeningFormValues>({
    resolver: zodResolver(screeningSchema),
    defaultValues: {
      assetId: "",
      profileId: "",
      debtRatio: "",
      liquidAssetsRatio: "",
      impureIncomeRatio: "",
      sourceCitationId: "",
      notes: "",
      manualOverrideStatus: "none",
      manualOverrideReason: "",
    },
  });

  useEffect(() => {
    if (defaultProfile && !form.getValues("profileId")) {
      form.setValue("profileId", defaultProfile.id);
    }
  }, [defaultProfile, form]);

  const screeningQuery = useQuery({
    queryKey: ["asset-shariah-screening", reviewKey?.assetId, reviewKey?.profileId],
    queryFn: () => {
      if (!reviewKey) return null;
      return getAssetShariahScreening(reviewKey.assetId, reviewKey.profileId);
    },
    enabled: enabled && reviewKey !== null,
  });

  const auditQuery = useQuery({
    queryKey: ["asset-shariah-screening-audit", reviewKey?.assetId, reviewKey?.profileId],
    queryFn: () => {
      if (!reviewKey) return [];
      return listShariahScreeningAudit(reviewKey.assetId, reviewKey.profileId);
    },
    enabled: enabled && reviewKey !== null,
  });

  const upsertMutation = useMutation({
    mutationFn: (values: ScreeningFormValues) =>
      upsertAssetShariahScreening({
        assetId: values.assetId,
        profileId: values.profileId,
        ratios: {
          debtRatio: values.debtRatio,
          liquidAssetsRatio: values.liquidAssetsRatio,
          impureIncomeRatio: values.impureIncomeRatio,
        },
        sourceCitationId: emptyToNull(values.sourceCitationId),
        notes: emptyToNull(values.notes),
        manualOverrideStatus:
          values.manualOverrideStatus === "none" ? null : values.manualOverrideStatus,
        manualOverrideReason: emptyToNull(values.manualOverrideReason),
      }),
    onSuccess: (screening) => {
      const key = { assetId: screening.assetId, profileId: screening.profileId };
      setReviewKey(key);
      void queryClient.invalidateQueries({
        queryKey: ["asset-shariah-screening", key.assetId, key.profileId],
      });
      void queryClient.invalidateQueries({
        queryKey: ["asset-shariah-screening-audit", key.assetId, key.profileId],
      });
    },
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

  return (
    <Page>
      <PageHeader heading="Screening" text="Review assets with optional Islamic finance criteria." />
      <PageContent className="space-y-4">
        <ProfilesCard profiles={profilesQuery.data ?? []} defaultProfile={defaultProfile} />
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Asset Screening Review</CardTitle>
          </CardHeader>
          <CardContent>
            <form
              className="grid gap-4"
              onSubmit={form.handleSubmit((values) => upsertMutation.mutate(values))}
            >
              <div className="grid gap-3 md:grid-cols-2">
                <Field label="Asset ID" error={form.formState.errors.assetId?.message}>
                  <Input {...form.register("assetId")} placeholder="asset id" />
                </Field>
                <Field label="Profile ID" error={form.formState.errors.profileId?.message}>
                  <Input {...form.register("profileId")} placeholder="profile id" />
                </Field>
              </div>
              <div className="grid gap-3 md:grid-cols-3">
                <Field
                  label="User-entered debt ratio"
                  error={form.formState.errors.debtRatio?.message}
                >
                  <Input {...form.register("debtRatio")} inputMode="decimal" placeholder="0.10" />
                </Field>
                <Field
                  label="User-entered liquid assets ratio"
                  error={form.formState.errors.liquidAssetsRatio?.message}
                >
                  <Input
                    {...form.register("liquidAssetsRatio")}
                    inputMode="decimal"
                    placeholder="0.20"
                  />
                </Field>
                <Field
                  label="User-entered impure income ratio"
                  error={form.formState.errors.impureIncomeRatio?.message}
                >
                  <Input
                    {...form.register("impureIncomeRatio")}
                    inputMode="decimal"
                    placeholder="0.01"
                  />
                </Field>
              </div>
              <Field label="Source citation ID" error={form.formState.errors.sourceCitationId?.message}>
                <Input {...form.register("sourceCitationId")} placeholder="optional citation id" />
              </Field>
              <Field label="Notes" error={form.formState.errors.notes?.message}>
                <Textarea {...form.register("notes")} placeholder="screening notes" />
              </Field>
              <div className="grid gap-3 md:grid-cols-2">
                <Field
                  label="Manual override status"
                  error={form.formState.errors.manualOverrideStatus?.message}
                >
                  <select
                    className="border-input bg-background h-10 rounded-md border px-3 text-sm"
                    {...form.register("manualOverrideStatus")}
                  >
                    <option value="none">No manual override</option>
                    <option value="compliant">Compliant</option>
                    <option value="non_compliant">Non-compliant</option>
                    <option value="questionable">Questionable</option>
                    <option value="unknown">Unknown</option>
                    <option value="needs_review">Needs review</option>
                  </select>
                </Field>
                <Field
                  label="Manual override reason"
                  error={form.formState.errors.manualOverrideReason?.message}
                >
                  <Input {...form.register("manualOverrideReason")} />
                </Field>
              </div>
              <p className="text-muted-foreground text-xs">
                Screening is an auditable review aid only. Mizan does not provide final religious
                or legal advice.
              </p>
              {upsertMutation.error ? (
                <p className="text-destructive text-sm">{upsertMutation.error.message}</p>
              ) : null}
              <Button type="submit" disabled={upsertMutation.isPending}>
                Save Review
              </Button>
            </form>
          </CardContent>
        </Card>
        <ReviewStatusCard screening={screeningQuery.data} audit={auditQuery.data ?? []} />
      </PageContent>
    </Page>
  );
}

function ProfilesCard({
  profiles,
  defaultProfile,
}: {
  profiles: ShariahScreeningProfile[];
  defaultProfile?: ShariahScreeningProfile;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Screening Profiles</CardTitle>
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
        {profiles.length > 0 ? (
          <ul className="text-muted-foreground space-y-1">
            {profiles.map((profile) => (
              <li key={profile.id}>
                {profile.name}
                {profile.isDefault ? " (default)" : ""}
              </li>
            ))}
          </ul>
        ) : null}
      </CardContent>
    </Card>
  );
}

function ReviewStatusCard({
  screening,
  audit,
}: {
  screening?: AssetShariahScreening | null;
  audit: ShariahScreeningAuditEntry[];
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Review Status</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3 text-sm">
        {screening ? (
          <>
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Current status:</span>
              <ShariahStatusBadge status={screening.status} />
            </div>
            {screening.sourceCitationId ? (
              <p className="text-muted-foreground">
                Document-backed citation: {screening.sourceCitationId}
              </p>
            ) : (
              <p className="text-muted-foreground">Ratios are user-entered without a citation.</p>
            )}
            {screening.manualOverrideReason ? (
              <p className="text-muted-foreground">
                Manual override reason: {screening.manualOverrideReason}
              </p>
            ) : null}
          </>
        ) : (
          <p className="text-muted-foreground">No screening review has been saved yet.</p>
        )}
        <div className="space-y-2">
          <div className="font-medium">Audit History</div>
          {audit.length > 0 ? (
            <ul className="space-y-2">
              {audit.map((entry) => (
                <li key={entry.id} className="rounded-md border p-3">
                  <div>{formatStatus(entry.previousStatus)} to {formatStatus(entry.newStatus)}</div>
                  <div className="text-muted-foreground text-xs">{entry.createdAt}</div>
                  {entry.notes ? <div className="text-muted-foreground">{entry.notes}</div> : null}
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-muted-foreground">No audit entries yet.</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <Label>{label}</Label>
      {children}
      {error ? <div className="text-destructive text-xs">{error}</div> : null}
    </div>
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

function formatStatus(status?: ShariahScreeningStatus | null): string {
  if (!status) return "not reviewed";
  return status.replace(/_/g, " ");
}

function emptyToNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}
