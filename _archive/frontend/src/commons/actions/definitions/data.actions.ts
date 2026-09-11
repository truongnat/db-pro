import { z } from "zod";

import { defineAction } from "../registry";

import type { ActionResult } from "../types";
import type { ExportResult } from "@/modules/export/types/export.types";

// ─── data.refresh ────────────────────────────────────────────

export const refreshDataAction = defineAction<
  { connectionId: string; schema: string; table: string },
  void
>({
  id: "data.refresh",
  title: "Refresh table data",
  description: "Re-fetch the data grid contents for a table.",
  category: "data",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    table: z.string().min(1),
  }),
  risk: "read",

  availability() {
    return { status: "unavailable", reason: "not_implemented" };
  },

  async execute(input) {
    // TODO: wire to IDataGridService.fetchRows via DI container.
    // The service will re-fetch from the backend and update TanStack Query cache.

    return {
      status: "success",
      effects: [
        {
          type: "data.refreshed",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.table,
        },
      ],
    };
  },
});

// ─── data.row.insert ─────────────────────────────────────────

export const insertRowAction = defineAction<
  {
    connectionId: string;
    schema: string;
    table: string;
    values: Record<string, unknown>;
  },
  { insertedCount: number }
>({
  id: "data.row.insert",
  title: "Insert row",
  description: "Insert a new row into a table.",
  category: "data",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    table: z.string().min(1),
    values: z.record(z.string(), z.unknown()),
  }),
  risk: "write",
  confirmation: { mode: "destructive-only", messageKey: "data.confirmInsert" },

  availability() {
    return { status: "unavailable", reason: "not_implemented" };
  },

  async execute(input) {
    // TODO: wire to IDataGridService.insertRow via DI container.

    return {
      status: "success",
      data: { insertedCount: 1 },
      effects: [
        {
          type: "data.row.inserted",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.table,
        },
      ],
    } satisfies ActionResult<{ insertedCount: number }>;
  },
});

// ─── data.row.update ─────────────────────────────────────────

export const updateRowAction = defineAction<
  {
    connectionId: string;
    schema: string;
    table: string;
    primaryKey: Record<string, unknown>;
    values: Record<string, unknown>;
  },
  { updatedCount: number }
>({
  id: "data.row.update",
  title: "Update row",
  description: "Update an existing row in a table.",
  category: "data",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    table: z.string().min(1),
    primaryKey: z.record(z.string(), z.unknown()),
    values: z.record(z.string(), z.unknown()),
  }),
  risk: "write",
  confirmation: { mode: "destructive-only", messageKey: "data.confirmUpdate" },

  availability() {
    return { status: "unavailable", reason: "not_implemented" };
  },

  async execute(input) {
    // TODO: wire to IDataGridService.updateRow via DI container.

    return {
      status: "success",
      data: { updatedCount: 1 },
      effects: [
        {
          type: "data.row.updated",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.table,
        },
      ],
    } satisfies ActionResult<{ updatedCount: number }>;
  },
});

// ─── data.row.delete ─────────────────────────────────────────

export const deleteRowAction = defineAction<
  {
    connectionId: string;
    schema: string;
    table: string;
    primaryKey: Record<string, unknown>;
  },
  { deletedCount: number }
>({
  id: "data.row.delete",
  title: "Delete row",
  description: "Delete a row from a table.",
  category: "data",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    table: z.string().min(1),
    primaryKey: z.record(z.string(), z.unknown()),
  }),
  risk: "destructive",
  confirmation: { mode: "always", messageKey: "data.confirmDelete" },

  availability() {
    return { status: "unavailable", reason: "not_implemented" };
  },

  async execute(input) {
    // TODO: wire to IDataGridService.deleteRow via DI container.

    return {
      status: "success",
      data: { deletedCount: 1 },
      effects: [
        {
          type: "data.row.deleted",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.table,
        },
      ],
    } satisfies ActionResult<{ deletedCount: number }>;
  },
});

// ─── data.filter ─────────────────────────────────────────────

export const filterDataAction = defineAction<
  {
    connectionId: string;
    schema: string;
    table: string;
    filter: Record<string, unknown>;
  },
  void
>({
  id: "data.filter",
  title: "Filter table data",
  description: "Apply a filter to the data grid.",
  category: "data",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    table: z.string().min(1),
    filter: z.record(z.string(), z.unknown()),
  }),
  risk: "read",

  availability() {
    return { status: "unavailable", reason: "not_implemented" };
  },

  async execute(input) {
    // TODO: wire to data grid store / TanStack Query cache to apply filter.

    return {
      status: "success",
      effects: [
        {
          type: "data.filtered",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.table,
          filterColumns: Object.keys(input.filter),
        },
      ],
    };
  },
});

// ─── data.sort ───────────────────────────────────────────────

export const sortDataAction = defineAction<
  {
    connectionId: string;
    schema: string;
    table: string;
    column: string;
    direction: "asc" | "desc";
  },
  void
>({
  id: "data.sort",
  title: "Sort table data",
  description: "Apply a sort to the data grid.",
  category: "data",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    schema: z.string().min(1),
    table: z.string().min(1),
    column: z.string().min(1),
    direction: z.enum(["asc", "desc"]),
  }),
  risk: "read",

  availability() {
    return { status: "unavailable", reason: "not_implemented" };
  },

  async execute(input) {
    // TODO: wire to data grid store / TanStack Query cache to apply sort.

    return {
      status: "success",
      effects: [
        {
          type: "data.sorted",
          connectionId: input.connectionId,
          schema: input.schema,
          table: input.table,
          column: input.column,
          direction: input.direction,
        },
      ],
    };
  },
});

// ─── data.export ─────────────────────────────────────────────

export const exportDataAction = defineAction<
  {
    connectionId: string;
    sql: string;
    format: "csv" | "json" | "excel";
  },
  { fileName: string; rowCount: number }
>({
  id: "data.export",
  title: "Export data",
  description: "Export query results to CSV, JSON, or Excel.",
  category: "data",
  inputSchema: z.object({
    connectionId: z.string().min(1),
    sql: z.string().min(1),
    format: z.enum(["csv", "json", "excel"]),
  }),
  risk: "read",

  async execute(input) {
    const { createExportService } = await import("@/modules/export/services/export.service");
    const service = createExportService();

    let result: ExportResult;
    switch (input.format) {
      case "csv":
        result = await service.exportCsv(input.connectionId, input.sql);
        break;
      case "json":
        result = await service.exportJson(input.connectionId, input.sql);
        break;
      case "excel":
        result = await service.exportExcel(input.connectionId, input.sql);
        break;
    }

    return {
      status: "success",
      data: { fileName: result.fileName, rowCount: result.rowCount },
      effects: [
        {
          type: "data.exported",
          connectionId: input.connectionId,
          format: input.format,
          fileName: result.fileName,
        },
      ],
    } satisfies ActionResult<{ fileName: string; rowCount: number }>;
  },
});
