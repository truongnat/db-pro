import type { GridSort } from "../types/data-grid.types";

export function cycleColumnSort(sorts: GridSort[], column: string): GridSort[] {
  const existing = sorts.find((s) => s.column === column);
  if (!existing) {
    return [{ column, direction: "asc" }];
  }
  if (existing.direction === "asc") {
    return [{ column, direction: "desc" }];
  }
  return [];
}
