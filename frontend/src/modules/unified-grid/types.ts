import type { CellValue, ColumnMeta, Row } from "@/modules/query/types/query.types";

export type { CellValue, ColumnMeta, Row };

export interface GridSort {
  column: string;
  direction: "asc" | "desc";
}
