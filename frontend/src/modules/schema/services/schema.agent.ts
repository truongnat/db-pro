import type {
  IntrospectResult,
  SchemaColumnDto,
  SchemaForeignKeyDto,
  SchemaIndexDto,
  TableInfo,
} from "../types/schema.types";

const COLUMNS: SchemaColumnDto[] = [
  {
    name: "id",
    dataType: "uuid",
    nullable: false,
    defaultValue: "gen_random_uuid()",
    isPrimaryKey: true,
    tableName: "users",
    schema: "public",
  },
  {
    name: "email",
    dataType: "varchar(255)",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "users",
    schema: "public",
  },
  {
    name: "name",
    dataType: "varchar(100)",
    nullable: true,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "users",
    schema: "public",
  },
  {
    name: "created_at",
    dataType: "timestamptz",
    nullable: false,
    defaultValue: "now()",
    isPrimaryKey: false,
    tableName: "users",
    schema: "public",
  },
  {
    name: "id",
    dataType: "uuid",
    nullable: false,
    defaultValue: "gen_random_uuid()",
    isPrimaryKey: true,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "user_id",
    dataType: "uuid",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "total",
    dataType: "numeric(12,2)",
    nullable: false,
    defaultValue: "0",
    isPrimaryKey: false,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "status",
    dataType: "varchar(20)",
    nullable: false,
    defaultValue: "'pending'",
    isPrimaryKey: false,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "created_at",
    dataType: "timestamptz",
    nullable: false,
    defaultValue: "now()",
    isPrimaryKey: false,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "id",
    dataType: "uuid",
    nullable: false,
    defaultValue: "gen_random_uuid()",
    isPrimaryKey: true,
    tableName: "products",
    schema: "public",
  },
  {
    name: "name",
    dataType: "varchar(200)",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "products",
    schema: "public",
  },
  {
    name: "price",
    dataType: "numeric(10,2)",
    nullable: false,
    defaultValue: null,
    isPrimaryKey: false,
    tableName: "products",
    schema: "public",
  },
];

const INDEXES: SchemaIndexDto[] = [
  {
    name: "users_email_key",
    columns: ["email"],
    unique: true,
    tableName: "users",
    schema: "public",
  },
  {
    name: "orders_user_id_idx",
    columns: ["user_id"],
    unique: false,
    tableName: "orders",
    schema: "public",
  },
  {
    name: "orders_status_idx",
    columns: ["status"],
    unique: false,
    tableName: "orders",
    schema: "public",
  },
];

const FOREIGN_KEYS: SchemaForeignKeyDto[] = [
  {
    name: "orders_user_id_fkey",
    fromTable: "orders",
    fromColumn: "user_id",
    toTable: "users",
    toColumn: "id",
    schema: "public",
    toSchema: "public",
  },
];

const DDL_MAP: Record<string, string> = {
  "public:users": `CREATE TABLE public.users (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    email varchar(255) NOT NULL,
    name varchar(100),
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT users_pkey PRIMARY KEY (id),
    CONSTRAINT users_email_key UNIQUE (email)
);`,
  "public:orders": `CREATE TABLE public.orders (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    total numeric(12,2) NOT NULL DEFAULT 0,
    status varchar(20) NOT NULL DEFAULT 'pending',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT orders_pkey PRIMARY KEY (id)
);

CREATE INDEX orders_user_id_idx ON public.orders USING btree (user_id);
CREATE INDEX orders_status_idx ON public.orders USING btree (status);

ALTER TABLE public.orders
    ADD CONSTRAINT orders_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);`,
  "public:products": `CREATE TABLE public.products (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    name varchar(200) NOT NULL,
    price numeric(10,2) NOT NULL,
    CONSTRAINT products_pkey PRIMARY KEY (id)
);`,
};

export class MockSchemaService {
  async introspect(_connectionId: string, _forceRefresh?: boolean): Promise<IntrospectResult> {
    return {
      schemas: [{ name: "public" }],
      tables: [
        { name: "users", schema: "public", rowCount: 150 },
        { name: "orders", schema: "public", rowCount: 1024 },
        { name: "products", schema: "public", rowCount: 42 },
      ],
      columns: COLUMNS,
      primaryKeys: [
        { constraintName: "users_pkey", columns: ["id"], tableName: "users", schema: "public" },
        { constraintName: "orders_pkey", columns: ["id"], tableName: "orders", schema: "public" },
        {
          constraintName: "products_pkey",
          columns: ["id"],
          tableName: "products",
          schema: "public",
        },
      ],
      indexes: INDEXES,
      foreignKeys: FOREIGN_KEYS,
      views: [],
    };
  }

  async getTableInfo(_connectionId: string, schema: string, table: string): Promise<TableInfo> {
    const columns = COLUMNS.filter((c) => c.schema === schema && c.tableName === table);
    return {
      table: { name: table, schema, rowCount: 100 },
      columns,
      primaryKey: {
        constraintName: `${table}_pkey`,
        columns: ["id"],
        tableName: table,
        schema,
      },
      indexes: INDEXES.filter((i) => i.schema === schema && i.tableName === table),
      foreignKeys: FOREIGN_KEYS.filter((fk) => fk.schema === schema && fk.fromTable === table),
    };
  }

  async getTableDdl(_connectionId: string, schema: string, table: string): Promise<string> {
    return DDL_MAP[`${schema}:${table}`] ?? `-- DDL for ${schema}.${table}`;
  }

  async executeDdl(_connectionId: string, _sql: string): Promise<{ affectedRows: number }> {
    return { affectedRows: 0 };
  }

  async invalidateCache(_connectionId: string): Promise<void> {}
}

export function createMockSchemaService(): MockSchemaService {
  return new MockSchemaService();
}
