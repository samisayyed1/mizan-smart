const DEFAULT_DECIMAL_SCALE = 12;

export interface ReportTotalInvariantInput {
  reportName: string;
  lineAmounts: readonly string[];
  reportedTotal: string;
}

export interface AccuracyInvariantResult {
  ok: boolean;
  code?: string;
  message?: string;
}

export function reportTotalMatchesLineSum(
  input: ReportTotalInvariantInput,
): AccuracyInvariantResult {
  const lineSum = input.lineAmounts.reduce(
    (sum, amount) => sum + parseDecimalToScaledBigInt(amount),
    0n,
  );
  const reportedTotal = parseDecimalToScaledBigInt(input.reportedTotal);

  if (lineSum === reportedTotal) {
    return { ok: true };
  }

  return {
    ok: false,
    code: "report_total_line_sum_mismatch",
    message: `${input.reportName} total ${input.reportedTotal} does not equal line sum ${formatScaledBigInt(lineSum)}.`,
  };
}

export function sumDecimalStrings(values: readonly string[]): string {
  return formatScaledBigInt(
    values.reduce((sum, value) => sum + parseDecimalToScaledBigInt(value), 0n),
  );
}

function parseDecimalToScaledBigInt(value: string, scale = DEFAULT_DECIMAL_SCALE): bigint {
  const trimmed = value.trim();
  const match = /^([+-])?(\d+)(?:\.(\d+))?$/.exec(trimmed);
  if (!match) {
    throw new Error(`Invalid decimal value: ${value}`);
  }

  const sign = match[1] === "-" ? -1n : 1n;
  const whole = match[2] ?? "0";
  const fraction = (match[3] ?? "").padEnd(scale, "0").slice(0, scale);
  return sign * (BigInt(whole) * 10n ** BigInt(scale) + BigInt(fraction || "0"));
}

function formatScaledBigInt(value: bigint, scale = DEFAULT_DECIMAL_SCALE): string {
  const sign = value < 0n ? "-" : "";
  const absolute = value < 0n ? -value : value;
  const divisor = 10n ** BigInt(scale);
  const whole = absolute / divisor;
  const fraction = (absolute % divisor).toString().padStart(scale, "0").replace(/0+$/, "");
  return fraction.length > 0 ? `${sign}${whole}.${fraction}` : `${sign}${whole}`;
}
