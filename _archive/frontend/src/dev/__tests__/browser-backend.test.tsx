import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";

import { apiInvoke } from "@/commons/utils/api";
import { bootstrapServices } from "@/app/app.module";
import { installBrowserBackend } from "../browser-backend";
import {
  buildIntrospectResult,
  buildTableDdl,
  buildTableInfo,
  MockSqlError,
  runStatement,
} from "../mock-database";

// `sonner` (mounted by <Toaster/> inside App) reads window.matchMedia at mount
// time; jsdom does not implement it.
if (typeof window.matchMedia !== "function") {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
}

beforeEach(() => {
  installBrowserBackend();
});

describe("fixture SQL engine", () => {
  it("runs SELECT * with column metadata from the fixture schema", () => {
    const result = runStatement("SELECT * FROM public.users LIMIT 5");

    expect(result.columns.map((c) => c.name)).toEqual([
      "id",
      "email",
      "name",
      "bio",
      "is_active",
      "created_at",
      "updated_at",
    ]);
    expect(result.rows).toHaveLength(5);
    expect(result.rowCount).toBe(5);
    // id is BIGINT → int64, is_active is BOOLEAN → bool, bio can be NULL.
    expect(result.rows[0][0]).toEqual({ type: "int64", value: "1" });
    expect(result.rows[0][4]).toEqual({ type: "bool", value: true });
    // Row 6 (Frank Miller) is the fixture row with a NULL bio.
    const all = runStatement("SELECT * FROM public.users");
    const frank = all.rows.find(
      (row) => (row[1] as { value: string }).value === "frank@example.com",
    );
    expect(frank?.[3]).toEqual({ type: "null" });
  });

  it("supports WHERE, ORDER BY and aggregates", () => {
    const pending = runStatement("SELECT * FROM public.orders WHERE status = 'pending'");
    expect(pending.rowCount).toBe(2);

    const counted = runStatement(
      "SELECT COUNT(*) AS n FROM public.orders WHERE status = 'pending'",
    );
    expect(counted.columns[0]).toEqual({ name: "n", dataType: "int8", nullable: true });
    expect(counted.rows[0][0]).toEqual({ type: "int64", value: "2" });

    const top = runStatement(
      "SELECT name AS product_name, price FROM public.products ORDER BY price DESC LIMIT 3",
    );
    expect(top.columns.map((c) => c.name)).toEqual(["product_name", "price"]);
    expect(top.rows[0][0]).toEqual({ type: "text", value: "MacBook Pro 14" });
    expect(top.rows[0][1]).toEqual({ type: "float64", value: 2499 });
  });

  it("keeps jsonb cells structured for the JSON cell renderer", () => {
    const result = runStatement(
      "SELECT entity_type, new_value FROM public.audit_log WHERE entity_type = 'product' LIMIT 1",
    );
    expect(result.columns[1].dataType).toBe("jsonb");
    expect(result.rows[0][1].type).toBe("json");
  });

  it("reports unknown relations as a PostgreSQL-shaped error", () => {
    expect(() => runStatement("SELECT * FROM public.orers")).toThrowError(MockSqlError);
    try {
      runStatement("SELECT * FROM public.orers");
      expect.unreachable("expected a MockSqlError");
    } catch (error) {
      expect((error as MockSqlError).error).toBe("QUERY_FAILED");
      expect((error as MockSqlError).message_id).toBe("error.query.failed");
    }
  });

  it("applies writes to the in-memory rows", () => {
    const before = runStatement("SELECT COUNT(*) FROM public.tags").rows[0][0];
    runStatement("INSERT INTO public.tags (name) VALUES ('preorder')");
    const after = runStatement("SELECT COUNT(*) FROM public.tags").rows[0][0];
    expect(Number((after as { value: string }).value)).toBe(
      Number((before as { value: string }).value) + 1,
    );
  });
});

describe("fixture introspection", () => {
  it("exposes tables, views, keys and indexes", () => {
    const result = buildIntrospectResult();
    expect(result.schemas).toEqual([{ name: "public" }]);
    expect(result.tables.map((t) => t.name)).toContain("orders");
    expect(result.views.map((v) => v.name)).toEqual(["active_users_summary", "product_catalog"]);
    expect(result.foreignKeys.map((fk) => fk.name)).toContain("orders_user_id_fkey");
    expect(result.indexes.map((i) => i.name)).toContain("idx_orders_status");
  });

  it("builds table info and DDL", () => {
    const info = buildTableInfo("public", "orders");
    expect(info?.primaryKey?.columns).toEqual(["id"]);
    expect(info?.foreignKeys).toHaveLength(1);

    const ddl = buildTableDdl("public", "orders");
    expect(ddl).toContain("CREATE TABLE public.orders");
    expect(ddl).toContain("CONSTRAINT orders_pkey PRIMARY KEY (id)");
    expect(ddl).toContain("CREATE INDEX idx_orders_status");
  });
});

describe("command router (through the real apiInvoke path)", () => {
  it("lists fixture connections", async () => {
    const connections =
      await apiInvoke<{ id: string; name: string; driver: string }[]>("list_connections");
    expect(connections.map((c) => c.name)).toContain("Acme Staging");
  });

  it("connects, introspects and executes a query", async () => {
    await apiInvoke<void>("connect", { id: "conn-staging" });
    const intro = await apiInvoke<{ tables: { name: string }[] }>("introspect", {
      connectionId: "conn-staging",
    });
    expect(intro.tables.map((t) => t.name)).toContain("products");

    const result = await apiInvoke<{ rowCount: number }>("execute_query", {
      connectionId: "conn-staging",
      sql: "SELECT * FROM public.products",
      executionId: "exec-1",
      database: "acme_staging",
      schema: "public",
    });
    expect(result.rowCount).toBe(10);
  });

  it("pages fetch_table_rows with filters and sorts", async () => {
    await apiInvoke<void>("connect", { id: "conn-staging" });
    const page = await apiInvoke<{ rows: unknown[][]; totalCount: number }>("fetch_table_rows", {
      connectionId: "conn-staging",
      request: {
        schema: "public",
        table: "products",
        filters: [{ column: "in_stock", op: "eq", value: { type: "bool", value: false } }],
        sorts: [{ column: "price", direction: "desc" }],
        page: 0,
        pageSize: 10,
      },
    });
    expect(page.totalCount).toBe(2);
    expect(page.rows[0][1]).toEqual({ type: "text", value: "Pixel 10" });
  });

  it("translates an unsupported relation into a user-facing error", async () => {
    await apiInvoke<void>("connect", { id: "conn-staging" });
    await expect(
      apiInvoke("execute_query", {
        connectionId: "conn-staging",
        sql: "SELECT * FROM public.orers",
        executionId: "exec-2",
      }),
    ).rejects.toMatchObject({ code: "QUERY_FAILED" });
  });

  it("rejects queries on a connection that is not connected", async () => {
    await expect(
      apiInvoke("execute_query", {
        connectionId: "conn-production",
        sql: "SELECT 1",
        executionId: "exec-3",
      }),
    ).rejects.toMatchObject({ code: "CONNECTION_FAILED" });
  });
});

describe("dev harness boots the real app shell", () => {
  it("renders the shell and lists a fixture connection", async () => {
    await bootstrapServices();
    const { default: App } = await import("@/App");

    render(<App />);

    // The shell chrome comes up and the fixture connection is reachable from
    // both the Explorer tree and the Welcome screen.
    await waitFor(() => {
      expect(screen.getAllByText("Acme Staging").length).toBeGreaterThan(0);
    });
    expect(screen.getByRole("contentinfo")).toBeInTheDocument();
    expect(screen.getByRole("navigation")).toBeInTheDocument();
    // Topbar quick-open surface + status bar metadata prove the chrome mounted.
    expect(screen.getAllByText("DB Pro").length).toBeGreaterThan(0);
  });
});
