import type { NavigatorObject } from "@/types";

export type NavigatorObjectAction =
  | "generate_select"
  | "open_data"
  | "copy_name"
  | "generate_insert"
  | "generate_update"
  | "generate_ddl";

export type NavigatorActionRequest = {
  schemaName: string;
  object: NavigatorObject;
  action: NavigatorObjectAction;
};
