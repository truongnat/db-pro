import type { QueryGridModifiers } from "./QueryResultGrid";

export type QueryRunPageOptions = {
  pageSizeOverride?: number;
  gridModifiersOverride?: QueryGridModifiers;
  skipHistory?: boolean;
};

export type RunQueryPageFn = (
  sql: string,
  offset: number,
  runSource: string,
  options?: QueryRunPageOptions,
) => Promise<void>;
