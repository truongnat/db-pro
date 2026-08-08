import { z } from "zod";

import { defineAction } from "../registry";

import type { ActionResult } from "../types";

// ─── schema.table.create ─────────────────────────────────────

export const createTableAction = defineAction<
  {
    connectionId: string;
    schema: string;
    tableName: string;
    columns: Array<{
      name: string;
      dataType: string;
      nullable?: boolean;
      primaryKey?: boolean;
    }>;
  },
  { ddl: string }
>({
  id: "schema.table.create",
  title: "Create table",
  description: "Generate and execute DDL to create a new table.",
  category: "schema",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    tableName: z.string().min(1),
    columns: z.array(
      z.object({
        name: z.string().min(1),
        dataType: z.string().min(1),
        nullable: z.boolean().optional(),
        primaryKey: z.boolean().optional(),
      }),
    ),
  }),
  risk: "write",
  confirmation: { mode: "always", messageKey: "schema.confirmCreateTable" },

  async execute(input) {
    // TODO: wire to ISchemaService.executeDdl via DI container.
    // For now, generate the DDL and return it.

    const columnDefs = input.columns
      .map((col) => {
        const parts = [`  ${col.name} ${col.dataType}`];
        if (!col.nullable) parts.push("NOT NULL");
        if (col.primaryKey) parts.push("PRIMARY KEY");
        return parts.join(" ");
      })
      .join(",\n");

    const ddl = `CREATE TABLE ${input.schema}.${input.tableName} (\n${columnDefs}\n);`;

    return {
      status: "success",
      data: { ddl },
      effects: [
        {
          type: "schema.table.created",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.tableName,
        },
      ],
    } satisfies ActionResult<{ ddl: string }>;
  },
});

// ─── schema.table.alter ──────────────────────────────────────

export const alterTableAction = defineAction<
  {
    connectionId: string;
    schema: string;
    tableName: string;
    ddl: string;
  },
  { affectedRows: number }
>({
  id: "schema.table.alter",
  title: "Alter table",
  description: "Execute an ALTER TABLE DDL statement.",
  category: "schema",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    tableName: z.string().min(1),
    ddl: z.string().min(1),
  }),
  risk: "write",
  confirmation: { mode: "always", messageKey: "schema.confirmAlterTable" },

  async execute(input) {
    // TODO: wire to ISchemaService.executeDdl via DI container.

    return {
      status: "success",
      data: { affectedRows: 0 },
      effects: [
        {
          type: "schema.table.altered",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.tableName,
        },
      ],
    } satisfies ActionResult<{ affectedRows: number }>;
  },
});

// ─── schema.table.drop ───────────────────────────────────────

export const dropTableAction = defineAction<
  { connectionId: string; schema: string; tableName: string },
  void
>({
  id: "schema.table.drop",
  title: "Drop table",
  description: "Drop a table from the database. This is irreversible.",
  category: "schema",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    tableName: z.string().min(1),
  }),
  risk: "destructive",
  confirmation: { mode: "always", messageKey: "schema.confirmDropTable" },

  async execute(input) {
    // TODO: wire to ISchemaService.executeDdl via DI container.

    return {
      status: "success",
      effects: [
        {
          type: "schema.table.dropped",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.tableName,
        },
      ],
    };
  },
});
