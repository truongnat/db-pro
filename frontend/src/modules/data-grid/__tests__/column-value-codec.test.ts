import { describe, expect, it } from "vitest";
import {
  normalizeColumnType,
  isCellTypeEditable,
  getUnsupportedEditReason,
} from "../utils/column-value-codec";

describe("normalizeColumnType", () => {
  it("maps boolean/bool to bool", () => {
    expect(normalizeColumnType("boolean")).toBe("bool");
    expect(normalizeColumnType("BOOL")).toBe("bool");
  });

  it("maps uuid to uuid", () => {
    expect(normalizeColumnType("uuid")).toBe("uuid");
  });

  it("maps json/jsonb to json", () => {
    expect(normalizeColumnType("json")).toBe("json");
    expect(normalizeColumnType("JSONB")).toBe("json");
  });

  it("maps safe integer types to int64", () => {
    expect(normalizeColumnType("integer")).toBe("int64");
    expect(normalizeColumnType("INT")).toBe("int64");
    expect(normalizeColumnType("smallint")).toBe("int64");
    expect(normalizeColumnType("serial")).toBe("int64");
    expect(normalizeColumnType("smallserial")).toBe("int64");
    expect(normalizeColumnType("int2")).toBe("int64");
    expect(normalizeColumnType("int4")).toBe("int64");
  });

  it("maps bigint/int8/bigserial to bigint (non-editable)", () => {
    expect(normalizeColumnType("bigint")).toBe("bigint");
    expect(normalizeColumnType("BIGINT")).toBe("bigint");
    expect(normalizeColumnType("int8")).toBe("bigint");
    expect(normalizeColumnType("bigserial")).toBe("bigint");
  });

  it("maps interval to text (not int64)", () => {
    expect(normalizeColumnType("interval")).toBe("text");
    expect(normalizeColumnType("INTERVAL")).toBe("text");
  });

  it("strips precision before classifying", () => {
    expect(normalizeColumnType("varchar(255)")).toBe("text");
    expect(normalizeColumnType("numeric(30,10)")).toBe("numeric");
    expect(normalizeColumnType("decimal(38,18)")).toBe("decimal");
  });

  it("maps numeric to numeric (not float64)", () => {
    expect(normalizeColumnType("numeric")).toBe("numeric");
    expect(normalizeColumnType("NUMERIC(30,10)")).toBe("numeric");
  });

  it("maps decimal to decimal (not float64)", () => {
    expect(normalizeColumnType("decimal")).toBe("decimal");
    expect(normalizeColumnType("DECIMAL(38,18)")).toBe("decimal");
  });

  it("maps float/double/real to float64", () => {
    expect(normalizeColumnType("float")).toBe("float64");
    expect(normalizeColumnType("DOUBLE PRECISION")).toBe("float64");
    expect(normalizeColumnType("real")).toBe("float64");
  });

  it("maps timestamp/date/time to datetime", () => {
    expect(normalizeColumnType("timestamp")).toBe("datetime");
    expect(normalizeColumnType("TIMESTAMPTZ")).toBe("datetime");
    expect(normalizeColumnType("date")).toBe("datetime");
  });

  it("maps bytea/blob to bytes", () => {
    expect(normalizeColumnType("bytea")).toBe("bytes");
    expect(normalizeColumnType("BLOB")).toBe("bytes");
  });

  it("maps unknown types to text", () => {
    expect(normalizeColumnType("varchar")).toBe("text");
    expect(normalizeColumnType("text")).toBe("text");
    expect(normalizeColumnType("citext")).toBe("text");
  });
});

describe("isCellTypeEditable", () => {
  it("returns false for bytes", () => {
    expect(isCellTypeEditable("bytes")).toBe(false);
  });

  it("returns false for numeric", () => {
    expect(isCellTypeEditable("numeric")).toBe(false);
  });

  it("returns false for decimal", () => {
    expect(isCellTypeEditable("decimal")).toBe(false);
  });

  it("returns false for bigint", () => {
    expect(isCellTypeEditable("bigint")).toBe(false);
  });

  it("returns true for editable types", () => {
    expect(isCellTypeEditable("text")).toBe(true);
    expect(isCellTypeEditable("int64")).toBe(true);
    expect(isCellTypeEditable("float64")).toBe(true);
    expect(isCellTypeEditable("bool")).toBe(true);
    expect(isCellTypeEditable("uuid")).toBe(true);
    expect(isCellTypeEditable("datetime")).toBe(true);
    expect(isCellTypeEditable("json")).toBe(true);
  });
});

describe("getUnsupportedEditReason", () => {
  it("returns reason for bytes", () => {
    expect(getUnsupportedEditReason("bytes")).toContain("Binary");
  });

  it("returns reason for numeric", () => {
    expect(getUnsupportedEditReason("numeric")).toContain("decimal");
  });

  it("returns reason for decimal", () => {
    expect(getUnsupportedEditReason("decimal")).toContain("decimal");
  });

  it("returns reason for bigint", () => {
    expect(getUnsupportedEditReason("bigint")).toContain("integer");
  });

  it("returns null for editable types", () => {
    expect(getUnsupportedEditReason("text")).toBeNull();
    expect(getUnsupportedEditReason("int64")).toBeNull();
    expect(getUnsupportedEditReason("bool")).toBeNull();
  });
});
