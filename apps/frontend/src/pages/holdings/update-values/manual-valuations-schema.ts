import * as z from "zod";

export const manualValuationRowSchema = z.object({
  assetId: z.string().trim().min(1, "Asset is required"),
  currentValue: z
    .string()
    .trim()
    .min(1, "Current value is required")
    .refine((value) => /^-?\d+(?:\.\d+)?$/.test(value), "Enter a valid decimal amount"),
  valuationDate: z
    .string()
    .trim()
    .regex(/^\d{4}-\d{2}-\d{2}$/, "Use yyyy-mm-dd"),
  currency: z
    .string()
    .trim()
    .length(3, "Use a 3-letter currency code")
    .regex(/^[A-Z]{3}$/, "Use uppercase ISO currency"),
  notes: z.string().optional().nullable(),
});

export const manualValuationBatchSchema = z.object({
  rows: z.array(manualValuationRowSchema),
});

export function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}
