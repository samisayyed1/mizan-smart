import type { GoldenImportTemplateConfig, ImportMappingData } from "@/lib/types";
import type { DraftActivity } from "../context";

export interface GoldenPreviewIssue {
  field: string;
  message: string;
}

export interface GoldenRowIssues {
  errors: Record<string, string[]>;
  warnings: Record<string, string[]>;
  duplicateOfLineNumber?: number;
}

export function getGoldenTemplate(
  mapping: Pick<ImportMappingData, "goldenTemplate"> | null | undefined,
): GoldenImportTemplateConfig | undefined {
  return mapping?.goldenTemplate;
}

export function getGoldenRequiredFields(mapping: ImportMappingData | null): string[] | null {
  const fields = getGoldenTemplate(mapping)?.requiredFields ?? [];
  return fields.length > 0 ? fields : null;
}

export function validateGoldenHeaders(
  headers: string[],
  goldenTemplate: GoldenImportTemplateConfig | undefined,
): { errors: GoldenPreviewIssue[]; warnings: GoldenPreviewIssue[] } {
  if (!goldenTemplate) {
    return { errors: [], warnings: [] };
  }

  const headerSet = new Set(headers);
  const strictSet = new Set(goldenTemplate.strictHeaders);
  const errors: GoldenPreviewIssue[] = [];
  const warnings: GoldenPreviewIssue[] = [];

  for (const requiredHeader of goldenTemplate.requiredHeaders) {
    if (!headerSet.has(requiredHeader)) {
      errors.push({
        field: "headers",
        message: `${goldenTemplate.displayName} is missing required column "${requiredHeader}".`,
      });
    }
  }

  for (const header of headers) {
    if (!strictSet.has(header)) {
      warnings.push({
        field: "headers",
        message: `${goldenTemplate.displayName} does not define column "${header}". Review it before importing.`,
      });
    }
  }

  return { errors, warnings };
}

export function buildGoldenRowIssues(
  headers: string[],
  rows: string[][],
  mapping: ImportMappingData | null,
): Map<number, GoldenRowIssues> {
  const goldenTemplate = getGoldenTemplate(mapping);
  const issuesByRow = new Map<number, GoldenRowIssues>();
  if (!goldenTemplate || !mapping) {
    return issuesByRow;
  }

  const headerIssues = validateGoldenHeaders(headers, goldenTemplate);
  const headerErrors = headerIssues.errors.map((issue) => issue.message);
  const headerWarnings = headerIssues.warnings.map((issue) => issue.message);

  const headerIndex = new Map<string, number>();
  headers.forEach((header, index) => headerIndex.set(header, index));

  const requiredFields = getGoldenRequiredFields(mapping) ?? [];
  const seenRows = new Map<string, number>();

  rows.forEach((row, rowIndex) => {
    const errors: Record<string, string[]> = {};
    const warnings: Record<string, string[]> = {};

    if (headerErrors.length > 0) {
      errors.headers = headerErrors;
    }
    if (headerWarnings.length > 0) {
      warnings.headers = headerWarnings;
    }

    for (const field of requiredFields) {
      const mappedHeader = mapping.fieldMappings[field];
      const value = firstMappedValue(row, headerIndex, mappedHeader);
      if (!value.trim()) {
        errors[field] = [`${field} is required by ${goldenTemplate.displayName}.`];
      }
    }

    const rowKey = normalizeRowKey(row);
    const duplicateOfLineNumber = seenRows.get(rowKey);
    if (duplicateOfLineNumber !== undefined) {
      warnings._duplicate = [`Duplicate of line ${duplicateOfLineNumber}.`];
    } else {
      seenRows.set(rowKey, rowIndex + 1);
    }

    if (
      Object.keys(errors).length > 0 ||
      Object.keys(warnings).length > 0 ||
      duplicateOfLineNumber !== undefined
    ) {
      issuesByRow.set(rowIndex, {
        errors,
        warnings,
        duplicateOfLineNumber,
      });
    }
  });

  return issuesByRow;
}

export function mergeGoldenIssuesIntoDrafts(
  drafts: DraftActivity[],
  issuesByRow: Map<number, GoldenRowIssues>,
): DraftActivity[] {
  if (issuesByRow.size === 0) {
    return drafts;
  }

  return drafts.map((draft) => {
    const issues = issuesByRow.get(draft.rowIndex);
    if (!issues) {
      return draft;
    }

    const errors = mergeIssueMaps(draft.errors, issues.errors);
    const warnings = mergeIssueMaps(draft.warnings, issues.warnings);
    const hasErrors = Object.keys(errors).length > 0;
    const hasWarnings = Object.keys(warnings).length > 0;
    const isDuplicate = issues.duplicateOfLineNumber !== undefined;

    return {
      ...draft,
      errors,
      warnings,
      duplicateOfLineNumber: issues.duplicateOfLineNumber ?? draft.duplicateOfLineNumber,
      status: hasErrors ? "error" : isDuplicate ? "duplicate" : hasWarnings ? "warning" : draft.status,
    };
  });
}

function firstMappedValue(
  row: string[],
  headerIndex: Map<string, number>,
  mappedHeader: string | string[] | undefined,
): string {
  if (!mappedHeader) return "";
  const candidates = Array.isArray(mappedHeader) ? mappedHeader : [mappedHeader];
  for (const header of candidates) {
    const index = headerIndex.get(header);
    if (index === undefined) continue;
    const value = row[index]?.trim() ?? "";
    if (value) return value;
  }
  return "";
}

function normalizeRowKey(row: string[]): string {
  return row.map((value) => value.trim().toUpperCase()).join("\u001f");
}

function mergeIssueMaps(
  left: Record<string, string[]>,
  right: Record<string, string[]>,
): Record<string, string[]> {
  const merged: Record<string, string[]> = { ...left };
  for (const [field, messages] of Object.entries(right)) {
    const existing = merged[field] ?? [];
    merged[field] = [...existing, ...messages.filter((message) => !existing.includes(message))];
  }
  return merged;
}
