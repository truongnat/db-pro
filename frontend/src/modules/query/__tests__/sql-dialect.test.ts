import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { useConnectionStore } from "@/commons/stores/connection.store";
import type { Connection } from "@/modules/connection/types/connection.types";

import {
  getDialectForConnection,
  getSqlDialect,
  type SqlDialect,
} from "../sql/dialect";

const CONNECTION_POSTGRES: Connection = {
  id: "pg-1",
  name: "Postgres",
  host: "localhost",
  port: 5432,
  database: "app",
  username: "dev",
  driver: "postgres",
  sslMode: "disable",
  createdAt: "2026-01-01",
  updatedAt: "2026-01-01",
};

const CONNECTION_SQLITE: Connection = {
  id: "sqlite-1",
  name: "Sqlite",
  host: "",
  port: 0,
  database: "app.db",
  username: "",
  driver: "sqlite",
  sslMode: "disable",
  createdAt: "2026-01-01",
  updatedAt: "2026-01-01",
};

describe("SQL dialect capabilities (UX-R7.2a)", () => {
  let postgres: SqlDialect;
  let sqlite: SqlDialect;

  beforeEach(() => {
    postgres = getSqlDialect("postgres");
    sqlite = getSqlDialect("sqlite");
    useConnectionStore.setState({ connections: [] });
  });

  afterEach(() => {
    useConnectionStore.setState({ connections: [] });
  });

  describe("quoteIdentifier", () => {
    it("quotes identifiers on postgres", () => {
      expect(postgres.quoteIdentifier("users")).toBe('"users"');
    });

    it("quotes identifiers on sqlite", () => {
      expect(sqlite.quoteIdentifier("users")).toBe('"users"');
    });

    it("escapes embedded double quotes on postgres", () => {
      expect(postgres.quoteIdentifier('a"b')).toBe('"a""b"');
    });

    it("escapes embedded double quotes on sqlite", () => {
      expect(sqlite.quoteIdentifier('user"name')).toBe('"user""name"');
    });
  });

  describe("qualify", () => {
    it("qualifies schema.object on postgres", () => {
      expect(postgres.qualify("public", "users")).toBe('"public"."users"');
    });

    it("defaults postgres schema to public when null", () => {
      expect(postgres.qualify(null, "users")).toBe('"public"."users"');
    });

    it("qualifies schema.object on sqlite", () => {
      expect(sqlite.qualify("main", "users")).toBe('"main"."users"');
    });

    it("drops schema on sqlite when null", () => {
      expect(sqlite.qualify(null, "users")).toBe('"users"');
    });
  });

  describe("generateSelect", () => {
    it("generates SELECT with schema on postgres", () => {
      expect(postgres.generateSelect({ schema: "public", table: "users" })).toBe(
        'SELECT * FROM "public"."users";',
      );
    });

    it("applies limit on postgres", () => {
      expect(
        postgres.generateSelect({ schema: "public", table: "users", limit: 100 }),
      ).toBe('SELECT * FROM "public"."users" LIMIT 100;');
    });

    it("generates unqualified SELECT on sqlite without schema", () => {
      expect(sqlite.generateSelect({ schema: null, table: "users" })).toBe(
        'SELECT * FROM "users";',
      );
    });

    it("applies limit on sqlite", () => {
      expect(sqlite.generateSelect({ schema: "main", table: "users", limit: 100 })).toBe(
        'SELECT * FROM "main"."users" LIMIT 100;',
      );
    });

    it("omits limit when limit is 0", () => {
      expect(postgres.generateSelect({ schema: "public", table: "users", limit: 0 })).toBe(
        'SELECT * FROM "public"."users";',
      );
    });
  });

  describe("formatter language", () => {
    it("maps postgres to sql-formatter postgresql", () => {
      expect(postgres.formatterLanguage).toBe("postgresql");
    });

    it("maps sqlite to sql-formatter sqlite", () => {
      expect(sqlite.formatterLanguage).toBe("sqlite");
    });
  });

  describe("dialect resolver", () => {
    it("returns a stable dialect instance per driver", () => {
      expect(getSqlDialect("postgres")).toBe(postgres);
      expect(getSqlDialect("sqlite")).toBe(sqlite);
    });

    it("resolves dialect from a known connection", () => {
      useConnectionStore.setState({ connections: [CONNECTION_POSTGRES, CONNECTION_SQLITE] });
      expect(getDialectForConnection("pg-1")).toBe(postgres);
      expect(getDialectForConnection("sqlite-1")).toBe(sqlite);
    });

    it("falls back to postgres when the connection is unknown or null", () => {
      expect(getDialectForConnection(null)).toBe(postgres);
      expect(getDialectForConnection("missing")).toBe(postgres);
    });
  });
});
