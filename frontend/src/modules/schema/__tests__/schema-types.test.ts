import { describe, expect, it } from "vitest";
import {
  buildTreeData,
  sortColumnsForDisplay,
  type IntrospectResult,
  type SchemaColumnDto,
} from "../types/schema.types";

const EMPTY_RESULT: IntrospectResult = {
  schemas: [],
  tables: [],
  columns: [],
  primaryKeys: [],
  indexes: [],
  foreignKeys: [],
  views: [],
  triggers: [],
};

function makeResult(overrides: Partial<IntrospectResult>): IntrospectResult {
  return { ...EMPTY_RESULT, ...overrides };
}

describe("buildTreeData", () => {
  it("returns empty array for empty introspect result", () => {
    const tree = buildTreeData(EMPTY_RESULT, "");
    expect(tree).toEqual([]);
  });

  it("groups tables by schema", () => {
    const result = makeResult({
      schemas: [{ name: "public" }],
      tables: [
        { name: "users", schema: "public", rowCount: 100 },
        { name: "orders", schema: "public", rowCount: 200 },
      ],
    });
    const tree = buildTreeData(result, "");
    expect(tree).toHaveLength(1);
    expect(tree[0].type).toBe("schema");
    expect(tree[0].label).toBe("public");
    expect(tree[0].children).toHaveLength(2);
  });

  it("sorts schemas alphabetically", () => {
    const result = makeResult({
      schemas: [{ name: "zebra" }, { name: "alpha" }],
      tables: [
        { name: "t1", schema: "zebra", rowCount: null },
        { name: "t2", schema: "alpha", rowCount: null },
      ],
    });
    const tree = buildTreeData(result, "");
    expect(tree[0].label).toBe("alpha");
    expect(tree[1].label).toBe("zebra");
  });

  it("sorts tables alphabetically within schema", () => {
    const result = makeResult({
      schemas: [{ name: "public" }],
      tables: [
        { name: "zebra", schema: "public", rowCount: null },
        { name: "alpha", schema: "public", rowCount: null },
        { name: "middle", schema: "public", rowCount: null },
      ],
    });
    const tree = buildTreeData(result, "");
    const labels = tree[0].children!.map((c) => c.label);
    expect(labels).toEqual(["alpha", "middle", "zebra"]);
  });

  it("includes views alongside tables", () => {
    const result = makeResult({
      schemas: [{ name: "public" }],
      tables: [{ name: "users", schema: "public", rowCount: 10 }],
      views: [{ name: "v_active", schema: "public", definition: "SELECT 1" }],
    });
    const tree = buildTreeData(result, "");
    expect(tree[0].children).toHaveLength(2);
    const types = tree[0].children!.map((c) => c.type);
    expect(types).toContain("table");
    expect(types).toContain("view");
  });

  it("filters tables by search query", () => {
    const result = makeResult({
      schemas: [{ name: "public" }],
      tables: [
        { name: "users", schema: "public", rowCount: null },
        { name: "orders", schema: "public", rowCount: null },
        { name: "user_profiles", schema: "public", rowCount: null },
      ],
    });
    const tree = buildTreeData(result, "user");
    expect(tree).toHaveLength(1);
    expect(tree[0].children).toHaveLength(2);
    const labels = tree[0].children!.map((c) => c.label);
    expect(labels).toContain("users");
    expect(labels).toContain("user_profiles");
    expect(labels).not.toContain("orders");
  });

  it("excludes schemas with no matching tables/views when filtering", () => {
    const result = makeResult({
      schemas: [{ name: "public" }, { name: "audit" }],
      tables: [
        { name: "users", schema: "public", rowCount: null },
        { name: "logs", schema: "audit", rowCount: null },
      ],
    });
    const tree = buildTreeData(result, "user");
    expect(tree).toHaveLength(1);
    expect(tree[0].label).toBe("public");
  });

  it("handles schemas with only views (no tables)", () => {
    const result = makeResult({
      schemas: [{ name: "reporting" }],
      views: [{ name: "summary", schema: "reporting", definition: "SELECT 1" }],
    });
    const tree = buildTreeData(result, "");
    expect(tree).toHaveLength(1);
    expect(tree[0].label).toBe("reporting");
    expect(tree[0].children).toHaveLength(1);
  });

  it("sorts views alphabetically within schema", () => {
    const result = makeResult({
      schemas: [{ name: "public" }],
      views: [
        { name: "z_view", schema: "public", definition: "SELECT 1" },
        { name: "a_view", schema: "public", definition: "SELECT 2" },
        { name: "m_view", schema: "public", definition: "SELECT 3" },
      ],
    });
    const tree = buildTreeData(result, "");
    const viewNodes = tree[0].children!.filter((c) => c.type === "view");
    expect(viewNodes.map((v) => v.label)).toEqual(["a_view", "m_view", "z_view"]);
  });

  it("filters views by search query", () => {
    const result = makeResult({
      schemas: [{ name: "public" }],
      views: [
        { name: "active_users", schema: "public", definition: "SELECT 1" },
        { name: "order_summary", schema: "public", definition: "SELECT 2" },
      ],
    });
    const tree = buildTreeData(result, "active");
    expect(tree).toHaveLength(1);
    const viewNodes = tree[0].children!.filter((c) => c.type === "view");
    expect(viewNodes).toHaveLength(1);
    expect(viewNodes[0].label).toBe("active_users");
  });

  it("generates correct node ids", () => {
    const result = makeResult({
      schemas: [{ name: "public" }],
      tables: [{ name: "users", schema: "public", rowCount: null }],
      views: [{ name: "v_test", schema: "public", definition: "" }],
    });
    const tree = buildTreeData(result, "");
    expect(tree[0].id).toBe("schema:public");
    const tableNode = tree[0].children!.find((c) => c.type === "table");
    const viewNode = tree[0].children!.find((c) => c.type === "view");
    expect(tableNode!.id).toBe("table:public:users");
    expect(viewNode!.id).toBe("view:public:v_test");
  });
});

describe("sortColumnsForDisplay", () => {
  function makeCol(overrides: Partial<SchemaColumnDto>): SchemaColumnDto {
    return {
      name: "col",
      dataType: "TEXT",
      nullable: true,
      defaultValue: null,
      isPrimaryKey: false,
      tableName: "t",
      schema: "public",
      ...overrides,
    };
  }

  it("puts primary key columns first", () => {
    const cols = [
      makeCol({ name: "email", isPrimaryKey: false }),
      makeCol({ name: "id", isPrimaryKey: true }),
      makeCol({ name: "name", isPrimaryKey: false }),
    ];
    const sorted = sortColumnsForDisplay(cols);
    expect(sorted[0].name).toBe("id");
  });

  it("sorts non-PK columns alphabetically", () => {
    const cols = [
      makeCol({ name: "zebra", isPrimaryKey: false }),
      makeCol({ name: "alpha", isPrimaryKey: false }),
      makeCol({ name: "middle", isPrimaryKey: false }),
    ];
    const sorted = sortColumnsForDisplay(cols);
    expect(sorted.map((c) => c.name)).toEqual(["alpha", "middle", "zebra"]);
  });

  it("sorts multiple PKs first, then alphabetically", () => {
    const cols = [
      makeCol({ name: "b", isPrimaryKey: true }),
      makeCol({ name: "z", isPrimaryKey: false }),
      makeCol({ name: "a", isPrimaryKey: true }),
      makeCol({ name: "m", isPrimaryKey: false }),
    ];
    const sorted = sortColumnsForDisplay(cols);
    expect(sorted.map((c) => c.name)).toEqual(["a", "b", "m", "z"]);
  });

  it("does not mutate the original array", () => {
    const cols = [
      makeCol({ name: "b", isPrimaryKey: false }),
      makeCol({ name: "a", isPrimaryKey: false }),
    ];
    const original = [...cols];
    sortColumnsForDisplay(cols);
    expect(cols[0].name).toBe(original[0].name);
    expect(cols[1].name).toBe(original[1].name);
  });

  it("handles empty array", () => {
    expect(sortColumnsForDisplay([])).toEqual([]);
  });
});
