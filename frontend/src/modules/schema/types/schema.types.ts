export interface SchemaDto {
  name: string;
}

export interface TableDto {
  name: string;
  schema: string;
  rowCount: number | null;
}

export interface SchemaColumnDto {
  name: string;
  dataType: string;
  nullable: boolean;
  defaultValue: string | null;
  isPrimaryKey: boolean;
  tableName: string;
  schema: string;
}

export interface PrimaryKeyDto {
  constraintName: string;
  columns: string[];
  tableName: string;
  schema: string;
}

export interface SchemaIndexDto {
  name: string;
  columns: string[];
  unique: boolean;
  tableName: string;
  schema: string;
}

export interface SchemaForeignKeyDto {
  name: string;
  fromTable: string;
  fromColumns: string[];
  toTable: string;
  toColumns: string[];
  schema: string;
  toSchema: string;
}

export interface ViewDto {
  name: string;
  schema: string;
  definition: string;
}

export interface TriggerDto {
  name: string;
  tableName: string;
  schema: string;
  timing: string;
  event: string;
  definition: string;
  functionDef: string;
  enabled: boolean;
}

export interface IntrospectResult {
  schemas: SchemaDto[];
  tables: TableDto[];
  columns: SchemaColumnDto[];
  primaryKeys: PrimaryKeyDto[];
  indexes: SchemaIndexDto[];
  foreignKeys: SchemaForeignKeyDto[];
  views: ViewDto[];
  triggers: TriggerDto[];
}

export interface TableInfo {
  table: TableDto;
  columns: SchemaColumnDto[];
  primaryKey: PrimaryKeyDto | null;
  indexes: SchemaIndexDto[];
  foreignKeys: SchemaForeignKeyDto[];
}

export type DetailTab =
  "columns" | "indexes" | "foreignKeys" | "ddl" | "ddlEditor" | "generateCrud" | "triggers";

export interface TreeNode {
  id: string;
  type: "schema" | "table" | "view";
  label: string;
  schemaName?: string;
  tableName?: string;
  children?: TreeNode[];
}

export function buildTreeData(result: IntrospectResult, searchQuery: string): TreeNode[] {
  const query = searchQuery.toLowerCase().trim();

  const tablesBySchema = new Map<string, TableDto[]>();
  for (const table of result.tables) {
    const list = tablesBySchema.get(table.schema) ?? [];
    list.push(table);
    tablesBySchema.set(table.schema, list);
  }

  const viewsBySchema = new Map<string, ViewDto[]>();
  for (const view of result.views) {
    const list = viewsBySchema.get(view.schema) ?? [];
    list.push(view);
    viewsBySchema.set(view.schema, list);
  }

  const schemaNames = result.schemas.map((s) => s.name);
  const allSchemaNames = new Set([
    ...schemaNames,
    ...tablesBySchema.keys(),
    ...viewsBySchema.keys(),
  ]);

  const nodes: TreeNode[] = [];

  for (const schemaName of [...allSchemaNames].sort()) {
    let tables = [...(tablesBySchema.get(schemaName) ?? [])].sort((a: TableDto, b: TableDto) =>
      a.name.localeCompare(b.name),
    );
    let views = [...(viewsBySchema.get(schemaName) ?? [])].sort((a: ViewDto, b: ViewDto) =>
      a.name.localeCompare(b.name),
    );

    if (query) {
      tables = tables.filter((t: TableDto) => t.name.toLowerCase().includes(query));
      views = views.filter((v: ViewDto) => v.name.toLowerCase().includes(query));
    }

    if (tables.length === 0 && views.length === 0) continue;

    const children: TreeNode[] = [
      ...tables.map((t): TreeNode => ({
        id: `table:${t.schema}:${t.name}`,
        type: "table",
        label: t.name,
        schemaName: t.schema,
        tableName: t.name,
      })),
      ...views.map((v): TreeNode => ({
        id: `view:${schemaName}:${v.name}`,
        type: "view",
        label: v.name,
        schemaName,
        tableName: v.name,
      })),
    ];

    nodes.push({
      id: `schema:${schemaName}`,
      type: "schema",
      label: schemaName,
      schemaName,
      children,
    });
  }

  return nodes;
}

export function sortColumnsForDisplay(columns: SchemaColumnDto[]): SchemaColumnDto[] {
  return [...columns].sort((a, b) => {
    if (a.isPrimaryKey !== b.isPrimaryKey) return a.isPrimaryKey ? -1 : 1;
    return a.name.localeCompare(b.name);
  });
}

export interface SchemaDiff {
  tablesOnlyInSource: string[];
  tablesOnlyInTarget: string[];
  columnDiffs: TableColumnDiff[];
  indexesOnlyInSource: string[];
  indexesOnlyInTarget: string[];
}

export interface TableColumnDiff {
  schema: string;
  table: string;
  columnsOnlyInSource: string[];
  columnsOnlyInTarget: string[];
  typeMismatches: ColumnTypeMismatch[];
}

export interface ColumnTypeMismatch {
  column: string;
  sourceType: string;
  targetType: string;
}

export interface DataDiff {
  schema: string;
  table: string;
  sourceRowCount: number;
  targetRowCount: number;
  rowCountDiff: number;
}

export interface ObjectDependency {
  objectType: string;
  objectName: string;
  dependsOnType: string;
  dependsOnName: string;
}

export interface PartitionInfo {
  schema: string;
  table: string;
  partitionStrategy: string;
  partitions: PartitionChild[];
}

export interface PartitionChild {
  name: string;
  boundExpr: string;
}

export interface TablespaceInfo {
  name: string;
  owner: string;
  location: string;
}
