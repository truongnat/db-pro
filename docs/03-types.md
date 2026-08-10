# 06 — DB Client — TypeScript Types (Synced with Rust Backend)

---

## 1. Domain Types (mirrors Rust `core/domain/`)

### 1.1 Connection Types

```typescript
// frontend/src/modules/connection/types/connection.types.ts

export type ConnectionId = string; // UUID string

export interface ConnectionConfig {
  name: string;
  host: string;
  port: number;
  database: string;
  username: string;
  encryptedPassword: string; // base64-encoded AES-256-GCM ciphertext
  driver: DriverType;
  sslMode: SslMode;
  sshTunnel: SshTunnelConfig | null;
}

export type DriverType = 'postgres' | 'sqlite';

export type SslMode = 'disable' | 'require' | 'verify-ca' | 'verify-full';

export interface SshTunnelConfig {
  host: string;
  port: number;
  user: string;
  privateKeyPath: string;
}

export interface Connection {
  id: ConnectionId;
  config: ConnectionConfig;
  createdAt: string; // ISO 8601 UTC
  updatedAt: string; // ISO 8601 UTC
}
```

**Sync with Rust**: `ConnectionId` maps to `ConnectionId(pub Uuid)`, `ConnectionConfig` maps to `ConnectionConfig` struct, `DriverType` maps to `enum DriverType`, `SslMode` maps to `enum SslMode`.

### 1.2 Query Types

```typescript
// frontend/src/modules/query/types/query.types.ts

export interface ColumnMeta {
  name: string;
  dataType: string;
  nullable: boolean;
}

export type Row = string[];

export interface QueryResult {
  columns: ColumnMeta[];
  rows: Row[];
  rowCount: number;
  durationMs: number;
}

export type QueryErrorType = 'validation' | 'not_found' | 'conflict' | 'unauthorized' | 'internal';

export interface QueryError {
  error: QueryErrorType;
  message: string; // i18n key
  messageId: string; // e.g., 'DB01001'
  details: unknown[];
}
```

**Sync with Rust**: `QueryResult` maps to `QueryResult` struct, `QueryErrorType` maps to `QueryError` enum variants, `QueryError` maps to `DbErrorDto`.

### 1.3 Schema Types

```typescript
// frontend/src/modules/schema/types/schema.types.ts

export interface Schema {
  name: string;
}

export interface Table {
  name: string;
  schema: string;
}

export interface Column {
  name: string;
  dataType: string;
  nullable: boolean;
  defaultValue: string | null;
}

export interface Index {
  name: string;
  columns: string[];
  unique: boolean;
}

export interface ForeignKey {
  name: string;
  fromTable: string;
  fromColumn: string;
  toTable: string;
  toColumn: string;
}

export interface View {
  name: string;
  definition: string;
}
```

**Sync with Rust**: Each interface maps 1:1 to the corresponding Rust struct in `core/domain/schema.rs`.

---

## 2. Tauri Command Types (DTOs at boundary)

### 2.1 Command Input Types

```typescript
// frontend/src/commons/types/command-input.types.ts

export interface ExecuteQueryInput {
  connectionId: string; // ConnectionId (UUID)
  sql: string;
}

export interface SaveConnectionInput {
  config: ConnectionConfig;
}

export interface TestConnectionInput {
  config: ConnectionConfig;
}

export interface ListConnectionsInput {}

export interface DeleteConnectionInput {
  connectionId: string;
}

export interface IntrospectInput {
  connectionId: string;
}

export interface ExportCsvInput {
  connectionId: string;
  sql: string;
  params: unknown[];
}

export interface ExportJsonInput {
  connectionId: string;
  sql: string;
  params: unknown[];
}
```

### 2.2 Command Output Types

```typescript
// frontend/src/commons/types/command-output.types.ts

export type ExecuteQueryOutput = QueryResult;

export type SaveConnectionOutput = Connection;

export type TestConnectionOutput = { success: boolean; message?: string };

export type ListConnectionsOutput = Connection[];

export type DeleteConnectionOutput = { success: boolean };

export type IntrospectOutput = {
  schemas: Schema[];
  tables: Table[];
  columns: Column[];
  indexes: Index[];
  foreignKeys: ForeignKey[];
  views: View[];
};

export type ExportCsvOutput = { fileContent: string; fileName: string };

export type ExportJsonOutput = { fileContent: string; fileName: string };
```

**Sync with Rust**: These DTOs mirror the `QueryResultDto`, `ConnectionDto`, etc. that Tauri commands return. Domain types never cross the boundary directly.

---

## 3. Service Types (mirrors Rust `core/application/`)

### 3.1 Connection Service

```typescript
// frontend/src/modules/connection/services/connection.service.ts

import type { Connection, ConnectionConfig, ConnectionId } from '../types/connection.types';

export interface IConnectionService {
  create(config: ConnectionConfig): Promise<Connection>;
  list(): Promise<Connection[]>;
  getById(id: ConnectionId): Promise<Connection | null>;
  update(id: ConnectionId, config: Partial<ConnectionConfig>): Promise<Connection>;
  delete(id: ConnectionId): Promise<void>;
  testConnectivity(config: ConnectionConfig): Promise<{ success: boolean; message?: string }>;
  connect(id: ConnectionId): Promise<void>;
  disconnect(id: ConnectionId): Promise<void>;
}
```

### 3.2 Query Service

```typescript
// frontend/src/modules/query/services/query.service.ts

import type { QueryResult, QueryError } from '../types/query.types';
import type { ConnectionId } from '../../connection/types/connection.types';

export interface IQueryService {
  execute(connectionId: ConnectionId, sql: string): Promise<QueryResult>;
  getHistory(connectionId: ConnectionId, limit?: number): Promise<QueryHistoryEntry[]>;
  saveToHistory(connectionId: ConnectionId, sql: string, result: QueryResult): Promise<void>;
}

export interface QueryHistoryEntry {
  sql: string;
  executedAt: string; // ISO 8601 UTC
  durationMs: number;
  rowCount: number;
}
```

### 3.3 Schema Service

```typescript
// frontend/src/modules/schema/services/schema.service.ts

import type { Schema, Table, Column, Index, ForeignKey, View } from '../types/schema.types';
import type { ConnectionId } from '../../connection/types/connection.types';

export interface ISchemaService {
  introspect(connectionId: ConnectionId): Promise<IntrospectResult>;
  getTableDdl(connectionId: ConnectionId, schema: string, table: string): Promise<string>;
}

export interface IntrospectResult {
  schemas: Schema[];
  tables: Table[];
  columns: Column[];
  indexes: Index[];
  foreignKeys: ForeignKey[];
  views: View[];
}
```

### 3.4 Export Service

```typescript
// frontend/src/modules/export/services/export.service.ts

import type { ConnectionId } from '../../connection/types/connection.types';

export type ExportFormat = 'csv' | 'json' | 'xlsx';

export interface IExportService {
  exportCsv(connectionId: ConnectionId, sql: string, params: unknown[]): Promise<ExportResult>;
  exportJson(connectionId: ConnectionId, sql: string, params: unknown[]): Promise<ExportResult>;
  exportExcel(connectionId: ConnectionId, sql: string, params: unknown[]): Promise<ExportResult>;
}

export interface ExportResult {
  fileContent: string;
  fileName: string;
  mimeType: string;
  rowCount: number;
}
```

---

## 4. Store Types (Zustand)

### 4.1 Connection Store

```typescript
// frontend/src/commons/stores/connection.store.ts

import type { Connection, ConnectionConfig } from '../../modules/connection/types/connection.types';

interface ConnectionStore {
  connections: Connection[];
  activeConnectionId: ConnectionId | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  setConnections: (connections: Connection[]) => void;
  addConnection: (connection: Connection) => void;
  updateConnection: (id: ConnectionId, config: Partial<ConnectionConfig>) => void;
  deleteConnection: (id: ConnectionId) => void;
  setActiveConnection: (id: ConnectionId | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}
```

### 4.2 Query History Store

```typescript
// frontend/src/commons/stores/query-history.store.ts

import type { QueryHistoryEntry } from '../../modules/query/services/query.service';

interface QueryHistoryStore {
  history: QueryHistoryEntry[];
  isLoading: boolean;

  setHistory: (history: QueryHistoryEntry[]) => void;
  addEntry: (entry: QueryHistoryEntry) => void;
  clearHistory: () => void;
}
```

### 4.3 Theme Store

```typescript
// frontend/src/commons/stores/theme.store.ts

interface ThemeStore {
  mode: 'light' | 'dark';
  setMode: (mode: 'light' | 'dark') => void;
}
```

### 4.4 Settings Store

```typescript
// frontend/src/commons/stores/settings.store.ts

interface SettingsStore {
  language: 'ja' | 'en';
  defaultConnectionId: ConnectionId | null;
  pageSize: 25 | 50 | 100 | 200;

  setLanguage: (lang: 'ja' | 'en') => void;
  setDefaultConnection: (id: ConnectionId | null) => void;
  setPageSize: (size: 25 | 50 | 100 | 200) => void;
}
```

---

## 5. DI Container Types

```typescript
// frontend/src/commons/di/registry.ts

export const SERVICE_NAMES = {
  CONNECTION_SERVICE: 'IConnectionService',
  QUERY_SERVICE: 'IQueryService',
  SCHEMA_SERVICE: 'ISchemaService',
  EXPORT_SERVICE: 'IExportService',
} as const;

export type ServiceName = typeof SERVICE_NAMES[keyof typeof SERVICE_NAMES];

export interface ServiceRegistry {
  [SERVICE_NAMES.CONNECTION_SERVICE]: IConnectionService;
  [SERVICE_NAMES.QUERY_SERVICE]: IQueryService;
  [SERVICE_NAMES.SCHEMA_SERVICE]: ISchemaService;
  [SERVICE_NAMES.EXPORT_SERVICE]: IExportService;
}
```

**Sync with Rust**: Mirrors `ServiceRegistry` in `commons/di/registry.ts`, using the same `SERVICE_NAMES` keys.

---

## 6. Query Key Types (TanStack Query)

```typescript
// frontend/src/modules/query/queries/query.keys.ts

import type { ConnectionId } from '../../connection/types/connection.types';

export const queryKeys = {
  connections: ['connections'] as const,
  connection: (id: ConnectionId) => ['connection', id] as const,
  queryHistory: (connectionId: ConnectionId) => ['query-history', connectionId] as const,
  schema: (connectionId: ConnectionId) => ['schema', connectionId] as const,
  queryResult: (connectionId: ConnectionId, sql: string) => ['query-result', connectionId, sql] as const,
} satisfies Record<string, (...args: unknown[]) => readonly unknown[]>;
```

---

## 7. Type Mapping: Rust ↔ TypeScript

| Rust Type | TypeScript Type | Notes |
|---|---|---|
| `ConnectionId(pub Uuid)` | `string` (UUID) | UUID as string at boundary |
| `ConnectionConfig` | `ConnectionConfig` interface | 1:1 mapping |
| `DriverType` enum | `'postgres' \| 'sqlite'` | String enum |
| `SslMode` enum | `'disable' \| 'require' \| 'verify-ca' \| 'verify-full'` | String enum |
| `QueryResult` | `QueryResult` interface | 1:1 mapping |
| `QueryError` | `QueryError` interface | 1:1 mapping |
| `Schema` | `Schema` interface | 1:1 mapping |
| `Table` | `Table` interface | 1:1 mapping |
| `Column` | `Column` interface | 1:1 mapping |
| `Index` | `Index` interface | 1:1 mapping |
| `ForeignKey` | `ForeignKey` interface | 1:1 mapping |
| `View` | `View` interface | 1:1 mapping |
| `ConnectionHandle` | Opaque `string` | Internal, not exposed to FE |
| `ExplainPlan` | `unknown` | JSON value, typed at FE usage |
| `Vec<T>` | `T[]` | Array |
| `Option<T>` | `T \| null` | Nullable |
| `Result<T, E>` | `Promise<T>` (errors via rejection) | Async |
| `serde_json::Value` | `unknown` | Typed at FE usage |

---

## 8. File Organization (Types)

```
frontend/src/
├── commons/
│   ├── types/
│   │   ├── command-input.types.ts
│   │   ├── command-output.types.ts
│   │   └── error.types.ts
│   └── di/
│       ├── registry.ts
│       └── container.ts
├── modules/
│   ├── connection/
│   │   ├── types/
│   │   │   └── connection.types.ts
│   │   └── services/
│   │       ├── connection.service.ts
│   │       └── connection.agent.ts
│   ├── query/
│   │   ├── types/
│   │   │   └── query.types.ts
│   │   ├── queries/
│   │   │   └── QU01001.queries.ts
│   │   └── services/
│   │       └── query.service.ts
│   ├── schema/
│   │   ├── types/
│   │   │   └── schema.types.ts
│   │   └── services/
│   │       └── schema.service.ts
│   └── export/
│       ├── types/
│       │   └── export.types.ts
│       └── services/
│           └── export.service.ts
```

---

## 9. Type Safety Rules

| Rule | Detail |
|---|---|
| No `any` type | Enforced by `@typescript-eslint/no-explicit-any` |
| All Tauri command inputs typed | Input interfaces in `command-input.types.ts` |
| All Tauri command outputs typed | Output interfaces in `command-output.types.ts` |
| Domain types never used directly in Tauri commands | Always use DTOs |
| Service interfaces use `I` prefix | `IConnectionService`, `IQueryService` |
| Types in `types/` subdirectory per module | Never mix types with components |
| Re-export types from module `index.ts` | `export * from './types/connection.types'` |
| `unknown` for JSON values at boundary | Cast to specific type at usage site |
| `null` for optional fields | Never `undefined` in API responses |
| `readonly` for immutable data | All interfaces use `readonly` where applicable |