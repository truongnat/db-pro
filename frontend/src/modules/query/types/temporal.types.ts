export type TemporalKind = "date" | "time" | "timetz" | "timestamp" | "timestamptz";

export interface TemporalValue {
  kind: TemporalKind;
  value: string;
}

const DATE_RE = /^(\d{4})-(\d{2})-(\d{2})$/;
const TIME_RE = /^(\d{2}):(\d{2}):(\d{2})\.(\d{6})$/;
const TIMETZ_RE = /^(\d{2}):(\d{2}):(\d{2})\.(\d{6})([+-])(\d{2}):(\d{2})$/;

function isLeapYear(year: number): boolean {
  return year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
}

function isCanonicalDate(value: string): boolean {
  const match = DATE_RE.exec(value);
  if (!match) return false;

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  if (month < 1 || month > 12 || day < 1) return false;

  const daysInMonth = [31, isLeapYear(year) ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  return day <= daysInMonth[month - 1];
}

function isCanonicalTime(value: string): boolean {
  const match = TIME_RE.exec(value);
  if (!match) return false;

  const hour = Number(match[1]);
  const minute = Number(match[2]);
  const second = Number(match[3]);
  return hour <= 23 && minute <= 59 && second <= 59;
}

function isCanonicalTimeTz(value: string): boolean {
  const match = TIMETZ_RE.exec(value);
  if (!match) return false;

  if (!isCanonicalTime(value.slice(0, 15))) return false;
  const offsetHour = Number(match[6]);
  const offsetMinute = Number(match[7]);
  return offsetHour <= 23 && offsetMinute <= 59;
}

function isCanonicalTimestamp(value: string): boolean {
  return (
    value.length === 26 &&
    value[10] === "T" &&
    isCanonicalDate(value.slice(0, 10)) &&
    isCanonicalTime(value.slice(11))
  );
}

export function isCanonicalTemporalValue(value: TemporalValue): boolean {
  switch (value.kind) {
    case "date":
      return isCanonicalDate(value.value);
    case "time":
      return isCanonicalTime(value.value);
    case "timetz":
      return isCanonicalTimeTz(value.value);
    case "timestamp":
      return isCanonicalTimestamp(value.value);
    case "timestamptz":
      return (
        value.value.length === 27 &&
        value.value.endsWith("Z") &&
        isCanonicalTimestamp(value.value.slice(0, -1))
      );
  }
}

/**
 * Temporal display/copy/export contract: return the canonical provider string
 * exactly. Never construct a JavaScript Date for DATE/TIME/TIMESTAMP values.
 */
export function renderTemporalValue(value: TemporalValue): string {
  return value.value;
}
