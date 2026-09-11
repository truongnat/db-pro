import type { LodLevel } from "../../utils/lod";

export interface TableColumnData {
  name: string;
  dataType: string;
  nullable: boolean;
  isPrimaryKey: boolean;
  isForeignKey: boolean;
}

export interface TableNodeData {
  label: string;
  schema: string;
  columns: TableColumnData[];
  /** Resolved level-of-detail for this node (injected by the diagram). */
  lod: LodLevel;
  /** Manual compact toggle — caps rendering at summary. */
  compact: boolean;
  [key: string]: unknown;
}
