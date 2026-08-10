import { describe, expect, it } from "vitest";
import { classifyConstraintError } from "../utils/constraint-errors";

describe("classifyConstraintError", () => {
  it("classifies unique violation", () => {
    const err =
      'duplicate key value violates unique constraint "users_email_key"\nDETAIL: Key (email)=(test@example.com) already exists.';
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("unique-violation");
    expect(result.userMessage).toContain("already exists");
  });

  it("classifies foreign key violation", () => {
    const err =
      'insert or update on table "orders" violates foreign key constraint "orders_user_id_fkey"\nDETAIL: Key (user_id)=(999) is not present in table "users".';
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("foreign-key-violation");
    expect(result.userMessage).toContain("does not exist");
  });

  it("classifies not-null violation", () => {
    const err = 'null value in column "email" of relation "users" violates not-null constraint';
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("not-null-violation");
    expect(result.userMessage).toContain("cannot be set to NULL");
  });

  it("classifies check violation", () => {
    const err = 'new row for relation "users" violates check constraint "users_age_check"';
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("check-violation");
    expect(result.userMessage).toContain("check constraint");
  });

  it("classifies invalid input syntax", () => {
    const err = 'invalid input syntax for type integer: "abc"';
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("invalid-type");
    expect(result.userMessage).toContain("not a valid");
  });

  it("classifies value too long", () => {
    const err = "value too long for type character varying(50)";
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("value-too-long");
    expect(result.userMessage).toContain("maximum allowed length");
  });

  it("classifies numeric out of range", () => {
    const err = "numeric field overflow (out of range)";
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("numeric-out-of-range");
  });

  it("classifies division by zero", () => {
    const err = "division by zero";
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("division-by-zero");
  });

  it("classifies serialization failure", () => {
    const err = "could not serialize access due to concurrent update";
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("serialization-failure");
    expect(result.userMessage).toContain("concurrent");
  });

  it("classifies deadlock", () => {
    const err = "deadlock detected";
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("deadlock");
  });

  it("classifies read-only", () => {
    const err = "cannot execute INSERT in a read-only transaction";
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("read-only");
  });

  it("falls back to unknown for unrecognized errors", () => {
    const err = "some random error";
    const result = classifyConstraintError(err);
    expect(result.kind).toBe("unknown");
    expect(result.userMessage).toBe("some random error");
  });

  it("truncates very long error messages", () => {
    const err = "x".repeat(300);
    const result = classifyConstraintError(err);
    expect(result.userMessage.length).toBeLessThanOrEqual(201); // 200 + …
  });

  describe("structured details", () => {
    it("uses structured details for unique violation", () => {
      const result = classifyConstraintError("ignored", {
        constraint_type: "unique",
        constraint: "users_email_key",
        table: "users",
        column: "email",
      });
      expect(result.kind).toBe("unique-violation");
      expect(result.userMessage).toContain("email");
      expect(result.constraint).toBe("users_email_key");
      expect(result.table).toBe("users");
    });

    it("uses structured details for foreign key violation", () => {
      const result = classifyConstraintError("ignored", {
        constraint_type: "foreign_key",
        constraint: "orders_user_id_fkey",
        table: "orders",
        column: null,
      });
      expect(result.kind).toBe("foreign-key-violation");
      expect(result.constraint).toBe("orders_user_id_fkey");
    });

    it("uses structured details for not-null violation", () => {
      const result = classifyConstraintError("ignored", {
        constraint_type: "not_null",
        constraint: "",
        table: "users",
        column: "name",
      });
      expect(result.kind).toBe("not-null-violation");
      expect(result.userMessage).toContain("name");
    });

    it("uses structured details for check violation", () => {
      const result = classifyConstraintError("ignored", {
        constraint_type: "check",
        constraint: "users_age_check",
        table: "users",
        column: null,
      });
      expect(result.kind).toBe("check-violation");
      expect(result.userMessage).toContain("users_age_check");
    });

    it("falls back to regex when details is null", () => {
      const err = 'null value in column "email" violates not-null constraint';
      const result = classifyConstraintError(err, null);
      expect(result.kind).toBe("not-null-violation");
    });
  });
});
