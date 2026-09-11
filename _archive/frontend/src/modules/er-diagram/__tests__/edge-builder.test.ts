import { describe, it, expect } from "vitest";
import { groupForeignKeys } from "../utils/edge-builder";

const fk = (overrides: Record<string, unknown> = {}) => ({
  name: "fk_default",
  fromTable: "orders",
  fromColumns: ["user_id"],
  toTable: "users",
  toColumns: ["id"],
  schema: "public",
  toSchema: "public",
  ...overrides,
});

describe("groupForeignKeys", () => {
  it("returns one group per single-column FK", () => {
    const fks = [
      fk({ name: "fk_orders_user", fromColumns: ["user_id"], toColumns: ["id"] }),
      fk({
        name: "fk_orders_product",
        fromTable: "orders",
        fromColumns: ["product_id"],
        toTable: "products",
        toColumns: ["id"],
      }),
    ];
    const visible = new Set(["public.orders", "public.users", "public.products"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(2);
    expect(groups[0].columns).toHaveLength(1);
    expect(groups[1].columns).toHaveLength(1);
  });

  it("handles composite FK with multiple columns", () => {
    // Composite FK: (tenant_id, parent_id) REFERENCES parent(tenant_id, id)
    // Backend already groups this into a single FK row with arrays
    const fks = [
      fk({
        name: "fk_composite",
        fromTable: "child",
        fromColumns: ["tenant_id", "parent_id"],
        toTable: "parent",
        toColumns: ["tenant_id", "id"],
      }),
    ];
    const visible = new Set(["public.child", "public.parent"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(1);
    expect(groups[0].key).toBe("public.child.fk_composite");
    expect(groups[0].columns).toHaveLength(2);
    expect(groups[0].columns[0]).toEqual({ from: "tenant_id", to: "tenant_id" });
    expect(groups[0].columns[1]).toEqual({ from: "parent_id", to: "id" });
  });

  it("does not merge FKs with different constraint names", () => {
    const fks = [
      fk({ name: "fk_a", fromColumns: ["col_a"], toColumns: ["id"] }),
      fk({ name: "fk_b", fromColumns: ["col_b"], toColumns: ["id"] }),
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
    const visible = new Set(["public.orders", "public.users"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(1);
    expect(groups[0].fk.name).toBe("fk_visible");
  });

  it("handles self-referencing FK", () => {
    const fks = [
      fk({
        name: "fk_parent",
        fromTable: "categories",
        fromColumns: ["parent_id"],
        toTable: "categories",
        toColumns: ["id"],
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
    const fks = [fk({ name: "orders_user_fkey", fromColumns: ["user_id"], toColumns: ["id"] })];
    const visible = new Set(["public.orders", "public.users"]);
    const groups = groupForeignKeys(fks, visible);

    // Key should include schema + fromTable + constraint name, not a sequential index.
    expect(groups[0].key).toBe("public.orders.orders_user_fkey");
  });

  it("does not merge same-named FKs on different tables", () => {
    const fks = [
      fk({
        name: "fk_same",
        fromTable: "orders",
        fromColumns: ["user_id"],
        toTable: "users",
        toColumns: ["id"],
      }),
      fk({
        name: "fk_same",
        fromTable: "products",
        fromColumns: ["user_id"],
        toTable: "users",
        toColumns: ["id"],
      }),
    ];
    const visible = new Set(["public.orders", "public.products", "public.users"]);
    const groups = groupForeignKeys(fks, visible);

    // Same constraint name but different fromTable → must NOT merge.
    expect(groups).toHaveLength(2);
    expect(groups[0].key).toBe("public.orders.fk_same");
    expect(groups[1].key).toBe("public.products.fk_same");
  });

  it("excludes FKs whose toTable is not visible (cross-schema)", () => {
    const fks = [
      fk({ name: "fk_local", fromTable: "orders", toTable: "users" }),
      fk({ name: "fk_remote", fromTable: "orders", toTable: "external_ref", toSchema: "other" }),
    ];
    const visible = new Set(["public.orders", "public.users"]);
    const groups = groupForeignKeys(fks, visible);

    expect(groups).toHaveLength(1);
    expect(groups[0].fk.name).toBe("fk_local");
  });
});
