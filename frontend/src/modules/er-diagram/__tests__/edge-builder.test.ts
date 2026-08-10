import { describe, it, expect } from "vitest";
import { groupForeignKeys } from "../utils/edge-builder";

const fk = (overrides: Record<string, string> = {}) => ({
  name: "fk_default",
  fromTable: "orders",
  fromColumn: "user_id",
  toTable: "users",
  toColumn: "id",
  schema: "public",
  toSchema: "public",
  ...overrides,
});

describe("groupForeignKeys", () => {
  it("returns one group per single-column FK", () => {
    const fks = [
      fk({ name: "fk_orders_user", fromColumn: "user_id", toColumn: "id" }),
      fk({
        name: "fk_orders_product",
        fromTable: "orders",
        fromColumn: "product_id",
        toTable: "products",
        toColumn: "id",
      }),
    ];
    const visible = new Set(["public.orders", "public.users", "public.products"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(2);
    expect(groups[0].columns).toHaveLength(1);
    expect(groups[1].columns).toHaveLength(1);
  });

  it("merges composite FK columns into a single group", () => {
    // Composite FK: (tenant_id, parent_id) REFERENCES parent(tenant_id, id)
    const fks = [
      fk({
        name: "fk_composite",
        fromTable: "child",
        fromColumn: "tenant_id",
        toTable: "parent",
        toColumn: "tenant_id",
      }),
      fk({
        name: "fk_composite",
        fromTable: "child",
        fromColumn: "parent_id",
        toTable: "parent",
        toColumn: "id",
      }),
    ];
    const visible = new Set(["public.child", "public.parent"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(1);
    expect(groups[0].key).toBe("public.fk_composite");
    expect(groups[0].columns).toHaveLength(2);
    expect(groups[0].columns[0]).toEqual({ from: "tenant_id", to: "tenant_id" });
    expect(groups[0].columns[1]).toEqual({ from: "parent_id", to: "id" });
  });

  it("does not merge FKs with different constraint names", () => {
    const fks = [
      fk({ name: "fk_a", fromColumn: "col_a", toColumn: "id" }),
      fk({ name: "fk_b", fromColumn: "col_b", toColumn: "id" }),
    ];
    const visible = new Set(["public.orders", "public.users"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(2);
  });

  it("excludes FKs whose fromTable is not visible", () => {
    const fks = [
      fk({ name: "fk_visible", fromTable: "orders" }),
      fk({ name: "fk_hidden", fromTable: "hidden_table" }),
    ];
    const visible = new Set(["public.orders"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(1);
    expect(groups[0].fk.name).toBe("fk_visible");
  });

  it("handles self-referencing FK", () => {
    const fks = [
      fk({
        name: "fk_parent",
        fromTable: "categories",
        fromColumn: "parent_id",
        toTable: "categories",
        toColumn: "id",
      }),
    ];
    const visible = new Set(["public.categories"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(1);
    expect(groups[0].fk.fromTable).toBe("categories");
    expect(groups[0].fk.toTable).toBe("categories");
  });

  it("handles empty FK list", () => {
    const groups = groupForeignKeys([], new Set());
    expect(groups).toHaveLength(0);
  });

  it("produces stable edge keys based on constraint name", () => {
    const fks = [fk({ name: "orders_user_fkey", fromColumn: "user_id", toColumn: "id" })];
    const visible = new Set(["public.orders", "public.users"]);
    const groups = groupForeignKeys(fks, visible);

    // Key should be schema-qualified constraint name, not a sequential index.
    expect(groups[0].key).toBe("public.orders_user_fkey");
  });
});
