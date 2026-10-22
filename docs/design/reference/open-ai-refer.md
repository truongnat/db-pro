Với app kiểu **DBeaver + Codex + AI agent**, mình sẽ không chỉ nghĩ “common UI component” theo kiểu `Button/Input/Dialog`, mà chia thành **4 tầng common**: foundation, database workspace, AI agent, và shared workflow. Nếu làm đúng từ đầu, sau này thêm PostgreSQL/MySQL/SQL Server/SQLite, query editor, schema explorer, AI agent, diff, execution log… sẽ đỡ vỡ kiến trúc.

Mình đề xuất full bộ như sau:

- **Foundation / Primitive components**
  - `Button`
  - `IconButton`
  - `Input`
  - `Textarea`
  - `SearchInput`
  - `Select`
  - `Combobox`
  - `Checkbox`
  - `RadioGroup`
  - `Switch`
  - `Label`
  - `FormField`
  - `Badge`
  - `StatusBadge`
  - `Avatar`
  - `Tooltip`
  - `Popover`
  - `DropdownMenu`
  - `ContextMenu`
  - `Dialog`
  - `ConfirmDialog`
  - `Sheet`
  - `Drawer`
  - `Tabs`
  - `SegmentedControl`
  - `Separator`
  - `Skeleton`
  - `Spinner`
  - `Toast`
  - `EmptyState`
  - `ErrorState`
  - `KeyboardShortcut`
  - `ScrollArea`
  - `ResizablePanel`
  - `SplitPane`
- **Application shell / workspace**
  - `AppShell`
  - `TitleBar`
  - `ActivityBar`
  - `PrimarySidebar`
  - `SecondarySidebar`
  - `SidebarSection`
  - `SidebarItem`
  - `SidebarTree`
  - `Toolbar`
  - `ToolbarGroup`
  - `ToolbarButton`
  - `CommandPalette`
  - `QuickOpen`
  - `Breadcrumb`
  - `StatusBar`
  - `BottomPanel`
  - `PanelHeader`
  - `PanelTabs`
  - `WorkspaceTabs`
  - `WorkspaceTab`
  - `PinnedTab`
  - `DirtyTabIndicator`
  - `ConnectionIndicator`
- **Database connection common**
  - `ConnectionCard`
  - `ConnectionForm`
  - `ConnectionSelector`
  - `ConnectionStatus`
  - `ConnectionBadge`
  - `DatabaseDriverIcon`
  - `DatabaseTypeBadge`
  - `ConnectionTestResult`
  - `ConnectionHealthIndicator`
  - `ConnectionLatency`
  - `CredentialField`
  - `SSHConfigurationForm`
  - `SSLConfigurationForm`
  - `AdvancedConnectionOptions`
  - `EnvironmentBadge`
  - `ConnectionColorMarker`
- **Database explorer / schema tree**
  - `DatabaseTree`
  - `DatabaseTreeNode`
  - `ServerNode`
  - `DatabaseNode`
  - `SchemaNode`
  - `TableNode`
  - `ViewNode`
  - `MaterializedViewNode`
  - `ColumnNode`
  - `PrimaryKeyNode`
  - `ForeignKeyNode`
  - `IndexNode`
  - `TriggerNode`
  - `FunctionNode`
  - `ProcedureNode`
  - `SequenceNode`
  - `TypeNode`
  - `ExtensionNode`
  - `TreeSearch`
  - `TreeFilter`
  - `TreeNodeActions`
  - `LazyTreeNode`
  - `TreeLoadingNode`
  - `TreeErrorNode`
- **SQL editor**
  - `SqlEditor`
  - `SqlEditorToolbar`
  - `SqlEditorStatusBar`
  - `SqlEditorTabs`
  - `EditorGutter`
  - `LineNumber`
  - `CurrentStatementHighlight`
  - `SelectionInfo`
  - `SqlFormatterButton`
  - `RunQueryButton`
  - `RunSelectionButton`
  - `RunCurrentStatementButton`
  - `ExplainQueryButton`
  - `CancelQueryButton`
  - `TransactionModeSelector`
  - `AutoCommitToggle`
  - `QueryTimeoutSelector`
  - `EditorConnectionSelector`
  - `EditorDatabaseSelector`
  - `EditorSchemaSelector`
- **Autocomplete / language intelligence**
  - `SqlAutocomplete`
  - `AutocompleteItem`
  - `FunctionSuggestion`
  - `ColumnSuggestion`
  - `TableSuggestion`
  - `KeywordSuggestion`
  - `SnippetSuggestion`
  - `HoverDocumentation`
  - `SignatureHelp`
  - `SqlDiagnostic`
  - `SqlErrorMarker`
  - `SqlWarningMarker`
- **Query result**
  - `QueryResultPanel`
  - `ResultTabs`
  - `DataGrid`
  - `DataGridHeader`
  - `DataGridCell`
  - `DataGridRow`
  - `DataGridVirtualizer`
  - `ColumnResizeHandle`
  - `ColumnSortIndicator`
  - `ColumnFilter`
  - `ColumnMenu`
  - `RowNumberCell`
  - `NullValue`
  - `BooleanCell`
  - `JsonCell`
  - `DateCell`
  - `BinaryCell`
  - `EditableCell`
  - `DirtyCellIndicator`
  - `ResultPagination`
  - `ResultLimitSelector`
  - `ResultStats`
  - `ExecutionTime`
  - `AffectedRows`
  - `ExportResultButton`
- **Data inspection**
  - `RowInspector`
  - `RecordView`
  - `KeyValueViewer`
  - `JsonViewer`
  - `JsonTree`
  - `XmlViewer`
  - `TextViewer`
  - `BinaryViewer`
  - `ImagePreview`
  - `LargeValueViewer`
  - `CopyValueButton`
  - `CellDetailPopover`
- **Table structure / object detail**
  - `ObjectDetailPage`
  - `ObjectHeader`
  - `ObjectMetadata`
  - `TableOverview`
  - `ColumnList`
  - `ColumnDetail`
  - `ConstraintList`
  - `IndexList`
  - `ForeignKeyList`
  - `TriggerList`
  - `DefinitionViewer`
  - `DDLViewer`
  - `ObjectProperties`
  - `DependencyGraph`
  - `ReferencedByList`
  - `ReferencesList`
- **Query history**
  - `QueryHistory`
  - `QueryHistoryItem`
  - `HistorySearch`
  - `HistoryFilter`
  - `ExecutionStatus`
  - `ExecutedAt`
  - `ExecutionDuration`
  - `FavoriteQuery`
  - `SavedQuery`
  - `RecentQuery`
- **Execution / logs**
  - `QueryExecutionLog`
  - `ExecutionStep`
  - `ExecutionStatusIcon`
  - `ExecutionProgress`
  - `ExecutionMessage`
  - `DatabaseError`
  - `DatabaseNotice`
  - `DatabaseWarning`
  - `LogViewer`
  - `LogEntry`
  - `LogFilter`
  - `Timestamp`
  - `Duration`
- **Explain / performance**
  - `ExplainPlan`
  - `ExplainPlanTree`
  - `PlanNode`
  - `PlanCost`
  - `ActualTime`
  - `RowsEstimate`
  - `RowsActual`
  - `QueryPerformanceSummary`
  - `BottleneckBadge`
  - `IndexRecommendation`
  - `QueryOptimizationSuggestion`
- **AI agent shell**
  - `AgentPanel`
  - `AgentChat`
  - `AgentMessage`
  - `UserMessage`
  - `AssistantMessage`
  - `AgentThinking`
  - `AgentComposer`
  - `AgentToolbar`
  - `AgentStatus`
  - `AgentAvatar`
  - `AgentModelSelector`
  - `AgentModeSelector`
  - `AgentContextIndicator`
  - `AgentStopButton`
  - `AgentRetryButton`
  - `AgentRegenerateButton`
- **AI context components**
  - `ContextChip`
  - `ContextList`
  - `ContextSource`
  - `SchemaContext`
  - `TableContext`
  - `ColumnContext`
  - `QueryContext`
  - `SelectionContext`
  - `ResultContext`
  - `FileContext`
  - `ConnectionContext`
  - `ContextPicker`
  - `AddContextButton`
  - `RemoveContextButton`
  - `ContextTokenUsage`
- **AI tool-call / agent execution**
  - `ToolCall`
  - `ToolCallHeader`
  - `ToolCallInput`
  - `ToolCallOutput`
  - `ToolCallStatus`
  - `ToolCallTimeline`
  - `ToolApproval`
  - `ToolApprovalActions`
  - `AgentStep`
  - `AgentStepStatus`
  - `AgentPlan`
  - `AgentTaskList`
  - `AgentTaskItem`
  - `AgentProgress`
  - `AgentError`
  - `AgentResult`
- **AI SQL-specific actions**
  - `GenerateSqlAction`
  - `ExplainSqlAction`
  - `FixSqlAction`
  - `OptimizeSqlAction`
  - `ConvertDialectAction`
  - `GenerateMigrationAction`
  - `GenerateSeedAction`
  - `AnalyzeSchemaAction`
  - `AnalyzeResultAction`
  - `SuggestIndexAction`
  - `DescribeTableAction`
  - `FindRelationshipAction`
- **AI diff / proposed changes**
  - `DiffViewer`
  - `DiffHeader`
  - `DiffHunk`
  - `DiffLine`
  - `DiffAddedLine`
  - `DiffRemovedLine`
  - `DiffContextLine`
  - `AcceptChangeButton`
  - `RejectChangeButton`
  - `AcceptAllButton`
  - `RejectAllButton`
  - `ApplySqlButton`
  - `PreviewSqlButton`
- **AI safe execution**
  - `SqlRiskBadge`
  - `DangerousQueryWarning`
  - `DestructiveOperationDialog`
  - `QueryPreview`
  - `ExecutionApproval`
  - `AffectedObjectsPreview`
  - `TransactionPreview`
  - `RollbackPreview`
  - `ReadOnlyBadge`
  - `ProductionWarning`
- **Transaction / mutation workflow**
  - `TransactionBar`
  - `PendingChanges`
  - `PendingChangeItem`
  - `CommitButton`
  - `RollbackButton`
  - `AutoCommitIndicator`
  - `TransactionState`
  - `UnsavedChangesDialog`
- **Migration / schema diff**
  - `SchemaDiff`
  - `SchemaDiffTree`
  - `ObjectDiff`
  - `ColumnDiff`
  - `IndexDiff`
  - `ConstraintDiff`
  - `MigrationPreview`
  - `MigrationScript`
  - `MigrationStep`
  - `MigrationStatus`
- **Search**
  - `GlobalSearch`
  - `DatabaseSearch`
  - `ObjectSearch`
  - `SearchResult`
  - `SearchResultGroup`
  - `SearchHighlight`
  - `RecentSearches`
  - `SearchFilters`
- **Developer-style utility**
  - `InlineCode`
  - `CodeBlock`
  - `CopyButton`
  - `TerminalBlock`
  - `MarkdownViewer`
  - `MarkdownEditor`
  - `CommandInput`
  - `ShortcutHint`
  - `PathBreadcrumb`
  - `StatusDot`
  - `ProgressRing`
  - `MonospaceValue`

Có một điểm rất quan trọng: **đừng biến tất cả những tên trên thành “primitive common component”**.

Mình sẽ tổ chức thành kiểu:

```text
src/
├── components/
│   ├── ui/
│   │   ├── button.tsx
│   │   ├── input.tsx
│   │   ├── dialog.tsx
│   │   ├── tabs.tsx
│   │   ├── tooltip.tsx
│   │   └── ...
│   │
│   ├── common/
│   │   ├── app-shell/
│   │   ├── toolbar/
│   │   ├── tree/
│   │   ├── data-grid/
│   │   ├── code/
│   │   ├── status/
│   │   ├── split-pane/
│   │   └── ...
│   │
│   ├── database/
│   │   ├── connection/
│   │   ├── explorer/
│   │   ├── query/
│   │   ├── result/
│   │   ├── schema/
│   │   └── explain/
│   │
│   └── agent/
│       ├── chat/
│       ├── context/
│       ├── tool-call/
│       ├── plan/
│       ├── diff/
│       └── approval/
```

Tức là có 3 mức:

```text
UI primitive
    ↓
Generic application component
    ↓
Domain component
```

Ví dụ:

```text
Button
    ↓
ToolbarButton
    ↓
RunQueryButton
```

hay:

```text
Tree
    ↓
LazyTree
    ↓
DatabaseTree
```

hay:

```text
CodeBlock
    ↓
DiffViewer
    ↓
AgentSqlProposal
```

Cách này cực hợp cho app bạn muốn làm.

## Một số component mình coi là “xương sống”

Nếu bắt đầu MVP, mình sẽ không build hết ngay. Mình ưu tiên 20 component/pattern sau trước:

```text
AppShell
ResizablePanel
WorkspaceTabs
Toolbar
CommandPalette

DatabaseTree
DatabaseTreeNode
ConnectionSelector

SqlEditor
SqlEditorToolbar

DataGrid
ResultPanel

BottomPanel
LogViewer

AgentPanel
AgentComposer
AgentMessage
ContextChip
ToolCall
DiffViewer
```

Chỉ cần các component trên được thiết kế tốt là bạn đã có thể dựng gần như toàn bộ trải nghiệm:

```text
┌─────┬───────────────┬───────────────────────────────────┐
│     │               │ Tab: query.sql               ×   │
│ ◉   │ Connections   ├───────────────────────────────────┤
│     │               │ localhost / postgres / public     │
│ 🔍  │ ▾ PostgreSQL  │ ▶ Run       Explain       AI ✦   │
│     │   ▾ postgres  ├───────────────────────────────────┤
│ DB  │     ▾ public  │                                   │
│     │       ▾ users │ SELECT *                          │
│ ✦   │         id    │ FROM users                        │
│     │         name  │ WHERE active = true;              │
│ ⚙   │               │                                   │
│     │               ├───────────────────────────────────┤
│     │               │ Results                           │
│     │               │ ┌────┬─────────────┬───────────┐ │
│     │               │ │ id │ name        │ active    │ │
│     │               │ ├────┼─────────────┼───────────┤ │
│     │               │ │ 1  │ Truong      │ true      │ │
│     │               │ └────┴─────────────┴───────────┘ │
├─────┴───────────────┴───────────────────────┬───────────┤
│ ✓ Query completed · 24 rows · 42ms          │           │
│                                             │  AI Agent │
│                                             │           │
│                                             │ Explain   │
│                                             │ this query│
│                                             │           │
│                                             │ ✦ ...     │
└─────────────────────────────────────────────┴───────────┘
```

Và đây là điểm mình nghĩ app của bạn **có thể khác DBeaver rất nhiều**:

Agent không nên chỉ là một cái “chat box bên phải”.

Nó nên có quyền hiểu toàn bộ workspace:

```text
current connection
current database
current schema
current editor
selected SQL
selected table
current result
current row
query history
execution errors
schema metadata
```

Cho nên bạn nên có một abstraction rất quan trọng:

```ts
type AgentContext = {
  connection?: ConnectionContext
  database?: DatabaseContext
  schema?: SchemaContext

  editor?: {
    content: string
    selection?: string
    cursor?: Position
  }

  object?: DatabaseObjectContext

  result?: {
    columns: ColumnMetadata[]
    sampleRows?: unknown[]
  }

  error?: DatabaseErrorContext
}
```

Sau đó UI dùng chung:

```tsx
<AgentContextBar>
  <ContextChip icon={<DatabaseIcon />}>
    localhost
  </ContextChip>

  <ContextChip icon={<TableIcon />}>
    public.users
  </ContextChip>

  <ContextChip icon={<CodeIcon />}>
    query.sql:12-28
  </ContextChip>
</AgentContextBar>
```

Agent nhìn phát là user cũng hiểu:

> “AI hiện đang biết cái gì?”

Đây là UX rất quan trọng cho sản phẩm kiểu Codex.

Một abstraction khác mình khuyên build rất sớm là **Agent Action**:

```ts
type AgentAction =
  | "explain"
  | "generate"
  | "fix"
  | "optimize"
  | "execute"
  | "inspect"
  | "modify"
```

Sau đó có common:

```tsx
<AgentAction
  type="optimize"
  context={currentQuery}
/>
```

thay vì mỗi feature tự gọi AI một kiểu.

Cuối cùng, nếu mình đặt **10 component quan trọng nhất về mặt sản phẩm** cho cái “Codex × DBeaver” của bạn, thì sẽ là:

1. `DatabaseTree`
2. `SqlEditor`
3. `DataGrid`
4. `WorkspaceTabs`
5. `ResizablePanel`
6. `AgentPanel`
7. `AgentContextBar`
8. `ToolCall`
9. `DiffViewer`
10. `ExecutionApproval`

Trong đó `ToolCall + DiffViewer + ExecutionApproval` mới là thứ làm app của bạn thực sự giống **agentic database client**, chứ không phải “DBeaver có thêm chatbot”.

Ví dụ AI nói:

```text
I found the issue.

The users.email column has no index and this query is scanning
1.8M rows.

Proposed change

+ CREATE INDEX CONCURRENTLY idx_users_email
+ ON users(email);

Impact
public.users

Risk
Low

[Preview]    [Run]
```

Đó mới chính là trải nghiệm mình nghĩ bạn nên hướng tới.