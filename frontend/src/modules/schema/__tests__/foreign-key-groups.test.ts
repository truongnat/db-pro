import { describe, expect, it } from "vitest";

import { groupForeignKeys } from "../utils/foreign-key-groups";
import type { SchemaForeignKeyDto } from "../types/schema.types";

function fk(overrides: Partial<SchemaForeignKeyDto> = {}): SchemaForeignKeyDto {
  return {
    name: "child_parent_fkey",
    fromTable: "child",
    fromColumn: "parent_id",
    toTable: "parent",
    toColumn: "id",
    schema: "public",
    toSchema: "public",
    ...overrides,
  };
}

describe("groupForeignKeys", () => {
  it("groups PostgreSQL-style composite FK rows into one ordered relation", () => {
    const result = groupForeignKeys([
      fk({ fromColumn: "tenant_id", toColumn: "tenant_id" }),
      fk({ fromColumn: "parent_id", toColumn: "id" }),
    ]);

    expect(result).toHaveLength(1);
    expect(result[0]?.fromColumns).toEqual(["tenant_id", "parent_id"]);
    expect(result[0]?.toColumns).toEqual(["tenant_id", "id"]);
  });

  it("does not merge constraints with different identities", () => {
    const result = groupForeignKeys([
      fk({ name: "child_parent_fkey" }),
      fk({ name: "child_owner_fkey", fromColumn: "owner_id", toColumn: "id" }),
    ]);

    expect(result).toHaveLength(2);
  });

  it("keeps cross-schema target identity in the group key", () => {
    const result = groupForeignKeys([
      fk({ name: "fk_shared", toSchema: "accounts" }),
      fk({ name: "fk_shared", toSchema: "archive" }),
    ]);

    expect(result).toHaveLength(2);
  });
});
