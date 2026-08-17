import { describe, expect, it } from "vitest";

import {
  isCanonicalTemporalValue,
  renderTemporalValue,
  type TemporalValue,
} from "../types/temporal.types";

const canonicalCases: TemporalValue[] = [
  { kind: "date", value: "2026-08-17" },
  { kind: "time", value: "23:59:59.123456" },
  { kind: "timetz", value: "23:59:59.123456+05:30" },
  { kind: "timestamp", value: "2026-08-17T23:59:59.123456" },
  { kind: "timestamptz", value: "2026-08-17T18:29:59.123456Z" },
];

describe("PostgreSQL temporal frontend contract", () => {
  it.each(canonicalCases)("accepts and renders $kind without conversion", (temporal) => {
    expect(isCanonicalTemporalValue(temporal)).toBe(true);
    expect(renderTemporalValue(temporal)).toBe(temporal.value);
  });

  it("rejects invented timezone semantics on timestamp", () => {
    expect(
      isCanonicalTemporalValue({
        kind: "timestamp",
        value: "2026-08-17T23:59:59.123456Z",
      }),
    ).toBe(false);
    expect(
      isCanonicalTemporalValue({
        kind: "timestamp",
        value: "2026-08-17T23:59:59.123456+07:00",
      }),
    ).toBe(false);
  });

  it("requires timestamptz to be normalized to canonical UTC Z", () => {
    expect(
      isCanonicalTemporalValue({
        kind: "timestamptz",
        value: "2026-08-17T23:59:59.123456+05:30",
      }),
    ).toBe(false);
    expect(
      isCanonicalTemporalValue({
        kind: "timestamptz",
        value: "2026-08-17T18:29:59.123456Z",
      }),
    ).toBe(true);
  });

  it("preserves six fractional microsecond digits exactly", () => {
    const temporal: TemporalValue = { kind: "time", value: "12:34:56.123400" };
    expect(isCanonicalTemporalValue(temporal)).toBe(true);
    expect(renderTemporalValue(temporal)).toBe("12:34:56.123400");
    expect(isCanonicalTemporalValue({ kind: "time", value: "12:34:56.123" })).toBe(false);
  });

  it("validates calendar/time boundaries without JavaScript Date", () => {
    expect(isCanonicalTemporalValue({ kind: "date", value: "2024-02-29" })).toBe(true);
    expect(isCanonicalTemporalValue({ kind: "date", value: "2026-02-29" })).toBe(false);
    expect(isCanonicalTemporalValue({ kind: "time", value: "24:00:00.000000" })).toBe(false);
    expect(
      isCanonicalTemporalValue({
        kind: "timetz",
        value: "12:00:00.000000+24:00",
      }),
    ).toBe(false);
  });
});
