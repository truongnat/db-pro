export type QuickOpenItemKind = "tab" | "db-object" | "schema" | "connection";

export interface QuickOpenTabItem {
  kind: "tab";
  tabId: string;
  title: string;
  connectionId: string | null;
  connectionName: string;
  resourceKey: string;
  searchText: string;
}

export interface QuickOpenDbObjectItem {
  kind: "db-object";
  connectionId: string;
  connectionName: string;
  schema: string;
  objectName: string;
  objectType: "table" | "view" | "function" | "sequence" | "type";
  resourceKey: string;
  searchText: string;
}

export interface QuickOpenSchemaItem {
  kind: "schema";
  connectionId: string;
  connectionName: string;
  schema: string;
  resourceKey: string;
  searchText: string;
}

export interface QuickOpenConnectionItem {
  kind: "connection";
  connectionId: string;
  connectionName: string;
  resourceKey: string;
  searchText: string;
}

export type QuickOpenItem =
  QuickOpenTabItem | QuickOpenDbObjectItem | QuickOpenSchemaItem | QuickOpenConnectionItem;

export type DbObjectType = "table" | "view" | "function" | "sequence" | "type";

export interface RecentResource {
  resourceKey: string;
  kind: "db-object" | "query" | "connection" | "schema-workspace";
  connectionId: string;
  schema?: string;
  objectName?: string;
  objectType?: DbObjectType;
  openedAt: string;
}
