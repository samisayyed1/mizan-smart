import {
  applyCorporateAction,
  listCorporateActions,
  previewCorporateAction,
  type ApplyCorporateActionRequest,
  type CorporateAction,
  type CorporateActionPreview,
  type CorporateActionType,
} from "@/adapters";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Button } from "@mizan/ui/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@mizan/ui/components/ui/card";
import { Input } from "@mizan/ui/components/ui/input";
import { Label } from "@mizan/ui/components/ui/label";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@mizan/ui/components/ui/alert-dialog";
import { useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

const actionTypes = [
  { value: "split", label: "Split" },
  { value: "reverse_split", label: "Reverse split" },
  { value: "symbol_change", label: "Symbol change" },
] as const;

const decimalString = z
  .string()
  .trim()
  .regex(/^\d+(\.\d+)?$/, "Use a positive decimal")
  .refine((value) => value !== "0", "Use a positive decimal");

const corporateActionSchema = z
  .object({
    actionType: z.enum(["split", "reverse_split", "symbol_change"]),
    effectiveDate: z.string().min(1, "Effective date is required"),
    ratioNumerator: z.string().optional(),
    ratioDenominator: z.string().optional(),
    newSymbol: z.string().optional(),
  })
  .superRefine((values, ctx) => {
    if (values.actionType === "split" || values.actionType === "reverse_split") {
      if (!decimalString.safeParse(values.ratioNumerator ?? "").success) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["ratioNumerator"],
          message: "Numerator is required",
        });
      }
      if (!decimalString.safeParse(values.ratioDenominator ?? "").success) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          path: ["ratioDenominator"],
          message: "Denominator is required",
        });
      }
    }
    if (values.actionType === "symbol_change" && !values.newSymbol?.trim()) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ["newSymbol"],
        message: "New symbol is required",
      });
    }
  });

type CorporateActionFormValues = z.infer<typeof corporateActionSchema>;

interface CorporateActionsPanelProps {
  assetId: string;
}

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

function actionLabel(value: CorporateActionType): string {
  return actionTypes.find((item) => item.value === value)?.label ?? value.replaceAll("_", " ");
}

function buildRequest(
  assetId: string,
  values: CorporateActionFormValues,
): ApplyCorporateActionRequest {
  const isSplit = values.actionType === "split" || values.actionType === "reverse_split";
  return {
    assetId,
    actionType: values.actionType,
    effectiveDate: values.effectiveDate,
    ratioNumerator: isSplit ? values.ratioNumerator?.trim() : null,
    ratioDenominator: isSplit ? values.ratioDenominator?.trim() : null,
    newSymbol: values.actionType === "symbol_change" ? values.newSymbol?.trim().toUpperCase() : null,
    sourceCitationId: null,
  };
}

export function CorporateActionsPanel({ assetId }: CorporateActionsPanelProps) {
  const queryClient = useQueryClient();
  const [preview, setPreview] = useState<CorporateActionPreview | null>(null);
  const [previewRequest, setPreviewRequest] = useState<ApplyCorporateActionRequest | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);

  const form = useForm<CorporateActionFormValues>({
    resolver: zodResolver(corporateActionSchema),
    defaultValues: {
      actionType: "split",
      effectiveDate: todayIso(),
      ratioNumerator: "2",
      ratioDenominator: "1",
      newSymbol: "",
    },
  });

  const selectedAction = form.watch("actionType");
  const historyQuery = useQuery({
    queryKey: ["corporate-actions", assetId],
    queryFn: () => listCorporateActions(assetId),
    enabled: assetId.length > 0,
  });

  const previewMutation = useMutation({
    mutationFn: previewCorporateAction,
    onSuccess: (result, request) => {
      setPreview(result);
      setPreviewRequest(request);
    },
    onError: (error) => {
      toast.error("Could not preview corporate action", { description: String(error) });
    },
  });

  const applyMutation = useMutation({
    mutationFn: applyCorporateAction,
    onSuccess: () => {
      setConfirmOpen(false);
      setPreview(null);
      setPreviewRequest(null);
      form.reset({
        actionType: "split",
        effectiveDate: todayIso(),
        ratioNumerator: "2",
        ratioDenominator: "1",
        newSymbol: "",
      });
      queryClient.invalidateQueries();
      toast.success("Corporate action applied");
    },
    onError: (error) => {
      toast.error("Could not apply corporate action", { description: String(error) });
    },
  });

  const history = useMemo(
    () => historyQuery.data ?? [],
    [historyQuery.data],
  );

  const onPreview = form.handleSubmit((values) => {
    const request = buildRequest(assetId, values);
    setPreview(null);
    setPreviewRequest(null);
    previewMutation.mutate(request);
  });

  const ratioFieldsVisible = selectedAction === "split" || selectedAction === "reverse_split";
  const symbolFieldVisible = selectedAction === "symbol_change";

  return (
    <Card data-testid="corporate-actions-panel">
      <CardHeader>
        <CardTitle className="text-sm font-medium">Corporate Actions</CardTitle>
      </CardHeader>
      <CardContent className="space-y-5">
        <form className="grid gap-4 md:grid-cols-5" onSubmit={onPreview}>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="corporate-action-type">Action</Label>
            <select
              id="corporate-action-type"
              className="border-input bg-background ring-offset-background focus-visible:ring-ring h-9 w-full rounded-md border px-3 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2"
              {...form.register("actionType")}
              onChange={(event) => {
                form.register("actionType").onChange(event);
                setPreview(null);
                setPreviewRequest(null);
              }}
            >
              {actionTypes.map((type) => (
                <option key={type.value} value={type.value}>
                  {type.label}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="corporate-action-date">Effective date</Label>
            <Input id="corporate-action-date" type="date" {...form.register("effectiveDate")} />
            {form.formState.errors.effectiveDate && (
              <p className="text-destructive text-xs">
                {form.formState.errors.effectiveDate.message}
              </p>
            )}
          </div>

          {ratioFieldsVisible && (
            <>
              <div className="space-y-2">
                <Label htmlFor="corporate-action-numerator">Numerator</Label>
                <Input
                  id="corporate-action-numerator"
                  inputMode="decimal"
                  {...form.register("ratioNumerator")}
                />
                {form.formState.errors.ratioNumerator && (
                  <p className="text-destructive text-xs">
                    {form.formState.errors.ratioNumerator.message}
                  </p>
                )}
              </div>
              <div className="space-y-2">
                <Label htmlFor="corporate-action-denominator">Denominator</Label>
                <Input
                  id="corporate-action-denominator"
                  inputMode="decimal"
                  {...form.register("ratioDenominator")}
                />
                {form.formState.errors.ratioDenominator && (
                  <p className="text-destructive text-xs">
                    {form.formState.errors.ratioDenominator.message}
                  </p>
                )}
              </div>
            </>
          )}

          {symbolFieldVisible && (
            <div className="space-y-2 md:col-span-2">
              <Label htmlFor="corporate-action-symbol">New symbol</Label>
              <Input
                id="corporate-action-symbol"
                autoCapitalize="characters"
                {...form.register("newSymbol")}
              />
              {form.formState.errors.newSymbol && (
                <p className="text-destructive text-xs">
                  {form.formState.errors.newSymbol.message}
                </p>
              )}
            </div>
          )}

          <div className="flex items-end">
            <Button type="submit" disabled={previewMutation.isPending}>
              Preview
            </Button>
          </div>
        </form>

        <p className="text-muted-foreground text-xs">
          Corporate actions are never applied from web evidence automatically. Review the preview,
          then confirm to write the audit event.
        </p>

        {preview && (
          <div className="rounded-md border p-3" data-testid="corporate-action-preview">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p className="text-sm font-medium">{actionLabel(preview.actionType)} preview</p>
                <p className="text-muted-foreground text-xs">
                  Effective {preview.effectiveDate}
                  {preview.ratio ? `, ratio ${preview.ratio}` : ""}
                  {preview.newSymbol ? `, new symbol ${preview.newSymbol}` : ""}
                </p>
              </div>
              <Button
                type="button"
                variant="outline"
                disabled={!previewRequest || applyMutation.isPending}
                onClick={() => setConfirmOpen(true)}
              >
                Apply reviewed action
              </Button>
            </div>

            {preview.positions.length > 0 ? (
              <div className="mt-3 overflow-x-auto">
                <table className="w-full min-w-[560px] text-left text-sm">
                  <thead className="text-muted-foreground text-xs">
                    <tr>
                      <th className="py-2 font-medium">Account</th>
                      <th className="py-2 font-medium">Quantity</th>
                      <th className="py-2 font-medium">Average cost</th>
                      <th className="py-2 font-medium">Cost basis</th>
                    </tr>
                  </thead>
                  <tbody>
                    {preview.positions.map((position) => (
                      <tr key={position.accountId} className="border-t">
                        <td className="py-2">{position.accountId}</td>
                        <td className="py-2">
                          {position.quantityBefore} to {position.quantityAfter}
                        </td>
                        <td className="py-2">
                          {position.averageCostBefore} to {position.averageCostAfter}
                        </td>
                        <td className="py-2">
                          {position.totalCostBasis} {position.currency}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <p className="text-muted-foreground mt-3 text-sm">
                No open positions are affected by this preview.
              </p>
            )}

            {preview.warnings.map((warning) => (
              <p key={warning} className="text-muted-foreground mt-2 text-xs">
                {warning}
              </p>
            ))}
          </div>
        )}

        <ActionHistory actions={history} isLoading={historyQuery.isLoading} />
      </CardContent>

      <AlertDialog open={confirmOpen} onOpenChange={setConfirmOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Apply corporate action</AlertDialogTitle>
            <AlertDialogDescription>
              This writes an immutable reviewed corporate action and updates the ledger model for
              this asset. Continue only after checking the preview.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                if (previewRequest) {
                  applyMutation.mutate(previewRequest);
                }
              }}
            >
              Confirm
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </Card>
  );
}

function ActionHistory({
  actions,
  isLoading,
}: {
  actions: CorporateAction[];
  isLoading: boolean;
}) {
  if (isLoading) {
    return <p className="text-muted-foreground text-sm">Loading corporate actions...</p>;
  }

  if (actions.length === 0) {
    return (
      <p className="text-muted-foreground text-sm" data-testid="corporate-actions-empty">
        No reviewed corporate actions have been recorded for this asset.
      </p>
    );
  }

  return (
    <div className="space-y-2">
      <p className="text-sm font-medium">Reviewed history</p>
      <div className="divide-y rounded-md border">
        {actions.map((action) => (
          <div key={action.id} className="flex flex-wrap items-center justify-between gap-2 p-3">
            <div>
              <p className="text-sm font-medium">{actionLabel(action.actionType)}</p>
              <p className="text-muted-foreground text-xs">{action.effectiveDate}</p>
            </div>
            <p className="text-muted-foreground text-xs">
              {action.actionType === "symbol_change"
                ? `New symbol ${action.newSymbol ?? "-"}`
                : `Ratio ${action.ratioNumerator ?? "-"}:${action.ratioDenominator ?? "-"}`}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}
