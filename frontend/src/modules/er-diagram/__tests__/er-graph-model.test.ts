import { describe, it, expect } from "vitest";
import { buildErGraphModel } from "../renderer/er-graph-model";
import type { IntrospectResult } from "@/modules/schema/types/schema.types";

function fixture(): IntrospectResult {
  return {
    tables: [
      { schema: "public", name: "customers" },
      { schema: "public", name: "orders" },
      { schema: "public", name: "order_items" },
      { schema: "other", name: "external" },
    ],
    columns: [
      { schema: "public", tableName: "customers", name: "id", dataType: "bigint", nullable: false },
      { schema: "public", tableName: "customers", name: "email", dataType: "text", nullable: true },
      { schema: "public", tableName: "orders", name: "id", dataType: "bigint", nullable: false },
      {
        schema: "public",
        tableName: "orders",
        name: "customer_id",
        dataType: "bigint",
        nullable: false,
      },
      {
        schema: "public",
        tableName: "orders",
        name: "total",
        dataType: "numeric",
        nullable: false,
      },
      {
        schema: "public",
        tableName: "order_items",
        name: "id",
        dataType: "bigint",
        nullable: false,
      },
      {
        schema: "public",
        tableName: "order_items",
        name: "order_id",
        dataType: "bigint",
        nullable: false,
      },
      { schema: "other", tableName: "external", name: "id", dataType: "bigint", nullable: false },
    ],
    primaryKeys: [
      { schema: "public", tableName: "customers", columns: ["id"] },
      { schema: "public", tableName: "orders", columns: ["id"] },
      { schema: "public", tableName: "order_items", columns: ["id"] },
    ],
    foreignKeys: [
      {
        schema: "public",
        fromTable: "orders",
        fromColumn: "customer_id",
        toSchema: "public",
        toTable: "customers",
        toColumn: "id",
        name: "orders_customer_id_fkey",
      },
      {
        schema: "public",
        fromTable: "order_items",
        fromColumn: "order_id",
        toSchema: "public",
        toTable: "orders",
        toColumn: "id",
        name: "order_items_order_id_fkey",
      },
      // FK whose target lives in another schema — must be excluded.
      {
        schema: "public",
        fromTable: "orders",
        fromColumn: "external_id",
        toSchema: "other",
        toTable: "external",
        toColumn: "id",
        name: "orders_external_id_fkey",
      },
    ],
  } as unknown as IntrospectResult;
}

describe("buildErGraphModel", () => {
  it("builds tables with column and FK counts", () => {
    const model = buildErGraphModel(fixture(), "public");
    expect(model.tables).toHaveLength(3);

    const customers = model.tables.find((t) => t.label === "customers")!;
    expect(customers.columnCount).toBe(2);
    expect(customers.fkCount).toBe(0); // referenced, not referencing

    const orders = model.tables.find((t) => t.label === "orders")!;
    expect(orders.columnCount).toBe(3);
    expect(orders.fkCount).toBe(1); // customer_id FK

    expect(model.stats.tables).toBe(3);
    expect(model.stats.columns).toBe(7);
  });

  it("includes only relations whose both endpoints are in the schema", () => {
    const model = buildErGraphModel(fixture(), "public");
    expect(model.relations).toHaveLength(2);
    expect(model.relations.every((r) => !r.id.includes("external"))).toBe(true);
    expect(model.stats.relations).toBe(2);
  });

  it("builds an undirected adjacency index", () => {
    const model = buildErGraphModel(fixture(), "public");
    const ordersId = "public.orders";
    const customersId = "public.customers";

    expect(model.adjacency.get(ordersId)?.has(customersId)).toBe(true);
    expect(model.adjacency.get(customersId)?.has(ordersId)).toBe(true);
  });

  it("ignores tables of other schemas entirely", () => {
    const model = buildErGraphModel(fixture(), "other");
    expect(model.tables).toHaveLength(1);
    expect(model.tables[0].label).toBe("external");
    expect(model.relations).toHaveLength(0);
  });
});
