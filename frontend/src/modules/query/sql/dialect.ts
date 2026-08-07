import type { SqlLanguage } from "sql-formatter";

import { useConnectionStore } from "@/commons/stores/connection.store";
import type { Connection, DriverType } from "@/modules/connection/types/connection.types";

import { postgresDialect } from "./postgres";
import { sqliteDialect } from "./sqlite";

export interface SelectOptions {
  schema: string | null;
  table: string;
  limit?: number;
}

export interface SqlDialect {
  readonly driver: DriverType;
  readonly formatterLanguage: SqlLanguage;
  quoteIdentifier(name: string): string;
  qualify(schema: string | null, object: string): string;
  generateSelect(options: SelectOptions): string;
}

const dialects: Record<DriverType, SqlDialect> = {
  postgres: postgresDialect,
  sqlite: sqliteDialect,
};

export function getSqlDialect(driver: DriverType): SqlDialect {
  return dialects[driver];
}

export function getDialectForConnection(connectionId: string | null): SqlDialect {
  const connection = (useConnectionStore.getState().connections as Connection[]).find(
    (c) => c.id === connectionId,
  );
  return connection ? getSqlDialect(connection.driver) : postgresDialect;
}
