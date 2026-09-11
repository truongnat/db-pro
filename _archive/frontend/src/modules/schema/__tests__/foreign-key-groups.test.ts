import { describe, expect, it } from "vitest";

import { groupForeignKeys } from "../utils/foreign-key-groups";
import type { SchemaForeignKeyDto } from "../types/schema.types";

function fk(overrides: Partial<SchemaForeignKeyDto> = {}): SchemaForeignKeyDto {
  return {
    name: "child_parent_fkey",
    fromTable: "child",
    fromColumns: ["parent_id"],
    toTable: "parent",
    toColumns: ["id"],
    schema: "public",
    toSchema: "public",
    ...overrides,
  };
}

describe("groupForeignKeys", () => {
  it("passes through composite FK with multiple columns", () => {
    const result = groupForeignKeys([
      fk({ fromColumns: ["tenant_id", "parent_id"], toColumns: ["tenant_id", "id"] }),
    ]);

    expect(result).toHaveLength(1);
    expect(result[0]?.fromColumns).toEqual(["tenant_id", "parent_id"]);
    expect(result[0]?.toColumns).toEqual(["tenant_id", "id"]);
  });

  it("does not merge constraints with different identities", () => {
    const result = groupForeignKeys([
      fk({ name: "child_parent_fkey" }),
      fk({ name: "child_owner_fkey", fromColumns: ["owner_id"], toColumns: ["id"] }),
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
