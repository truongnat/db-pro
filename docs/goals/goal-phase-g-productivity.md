# DB Pro — Phase G Goal: Productivity, Search & Personalization

- Doc ID: `GOAL-PHASE-G`
- Phase: G — Productivity / Search
- Release targets: v0.2 (G01), v0.4 (G02–G04)
- Priority: P1 (G01), P2 (G02–G04)
- Authority: `docs/goals/goal-full-product.md` (master goal)
- Depends on: K01 (Settings Center) for G04; the meta-store migration pattern (§4.6 of the
  master goal) for G02/G03; nothing for G01.
- Status: PLANNING. No implementation has started.
- Companions: `docs/goals/goal-phase-a-object-crud.md` (search index must know new object
  kinds), `docs/goals/goal-phase-h-ai.md` (agent context lookup reuses the index).

---

## 1. Problem

DB Pro is fast at execution and slow at *finding*. This is the phase where the product can
become better than mature clients instead of merely equal to them.

Observed facts (source-verified):

1. **There is no Search activity.** The `Activity` enum is
   `{ Explorer, Queries, History, Transfers, Monitor, Settings, Diagram }`
   (`crates/ui/src/app.rs:114-123`) — no Search variant, no search view, no index. The topbar
   search box is a palette launcher only: it is painted with the placeholder "Search commands,
   tables, schemas..." and on click it calls `open_palette(PaletteMode::QuickOpen)`
   (`crates/ui/src/navigation_view.rs:174-225`).
2. **The palette is narrow.** Quick Open covers tables up to a cap of 100 plus static items
   (`palette_view.rs:150-163`). It does **not** search: columns, views, routines, saved-query
   SQL text, history SQL text, settings, or the newer commands this roadmap adds. The
   capability matrix rates "Search activity (dedicated)" as `MISSING` with the highest-value
   gap being exactly this findability problem.
3. **Organization features are absent or partial:**
   - Favorites: the *connection* domain supports folders/tags/favorites/color, but the UI DTO
     drops them (`UiConnectionSummary`; `crates/native-app/src/translate.rs:41-42`) — so the
     capability exists in the model and is invisible.
   - Pinned objects: absent everywhere.
   - Recent: only the reverse-chronological query history list.
   - Snippets: two hardcoded buttons ("SELECT table", "UPDATE by primary key") in
     `crates/ui/src/query_view.rs:276-288`; no library, no persistence, no variables. (The
     completion engine does emit `CompletionItemKind::Snippet` items for JOIN clauses —
     `query/schema_completion.rs:316,808` — but that is editor completion, not a library.)
   - Scratch SQL: absent; an unsaved query document is the closest thing.
   - Saved-query tags and move-between-folders: absent (folders exist —
     `meta/saved_query_repo.rs` has 7 methods — but there is no tag support and no move).
4. **Editor/grid/persistence settings are ad hoc.** Editor flags live in a per-query overflow
   menu; only the prediction mode is persisted (`dbpro.native.prediction-mode`). Grid layout
   persists (`dbpro.native.grid-layouts`) but grid *behavior* settings do not. The
   `RunConfigRepository` exists (`meta/run_config_repo.rs`) with no native UI wiring (no
   `UiCommand`/`RuntimeCommand` variants). Connection timeouts/max-rows exist per connection
   but there is no defaults surface.
5. **Keybindings are hardcoded** in `crates/ui/src/events.rs` (shortcut handling is a static
   match), with a documented inconsistency: `Ctrl+N` is advertised in the palette hint
   (`navigation_view.rs:584`, `workspace_view.rs:533-536`) but is not wired
   (`events.rs:1102-1160`) — the capability matrix calls this out as a paper cut.
6. **Some grid/editor paper cuts remain**: copying ignores staged values in one path, the JSON
   viewer has no tree mode, `DEFAULT` has no apply affordance (placeholder text only),
   and query parameters have no dialog while a binary-cell hint references them
   (`table_editor_view.rs:798`).

Why this matters: a database IDE is used for hours a day. Findability (jump to any object or
query in two keystrokes) and reusability (snippets, templates, favorites) determine whether it
feels professional. These are also the features that make the Agent more useful, because a
searchable index is the agent's context lookup.

## 2. Scope

### 2.1 In scope

| Group | Content | Milestone |
|---|---|---|
| Palette expansion | Columns, views, routines, sequences, types (from the active connection), saved-query SQL text, history SQL text, settings entries, all registered commands; grouping, keyboard navigation, type icons | G01 |
| Search activity | First-class activity with a unified index across connections, filters, preview pane, and actions | G02 |
| Unified search index | Connections, databases, schemas, tables, columns, views, materialized views, routines, sequences, types, saved queries, history, commands, agent actions, settings | G02 |
| Index lifecycle | Incremental population from introspection, invalidation on schema-change events, bounded size, secret-free by construction | G02 |
| Favorites / Pinned / Recent | Objects, connections, and queries; surfaces in Search and in the relevant activity | G03 |
| Snippet library | CRUD, folders, variables with typed prompts, insertion into the editor, import/export | G03 |
| Scratch SQL | Persisted scratch documents (multiple, named), separate from saved queries | G03 |
| Saved-query organization | Tags, move between folders, favorites | G03 |
| Connection organization UI | Folders, tags, favorites, color (wiring the existing domain through the UI DTO) | G03 |
| Editor settings | Font size, tab size, word wrap, completion on/off, prediction mode, format-on-run, diagnostics verbosity | G04 |
| Data-grid settings | Default row height, NULL display, copy format, date/number formats, page size, sticky header | G04 |
| Connection defaults | Default timeout, max rows, default schema, default read-only, default environment | G04 |
| Keybindings | Remappable bindings, conflict detection, cheatsheet, reset to defaults | G04 |
| Query parameters dialog | Typed parameter entry bound as `QueryParam` (never interpolated); unblocks the binary-cell hint | G04 |
| Grid/editor paper cuts | Staged-value copy correctness, JSON tree viewer, `DEFAULT` apply affordance | G04 |

### 2.2 Out of scope

- Cross-connection federated queries.
- Cloud sync of snippets/favorites/settings.
- Semantic/embedding search.
- A plugin/marketplace for snippets.
- Full keybinding theme/import/export (remapping + reset is enough for this phase).
- Localization/i18n of the UI (the message-id infrastructure exists in errors; a full
  localization framework is not in this roadmap).

## 3. Non-goals

- **No index that contains secrets or data.** The index stores identifiers, metadata, and query
  *text* (which the user typed), never credentials, connection strings with passwords, bound
  parameter values, or result rows. This is asserted by test and is a security boundary, not a
  preference.
- **No background crawler that queries the database on a timer.** Index population happens on
  introspection and on explicit refresh; a schema-change event invalidates and repopulates. The
  app must not become a source of background load on the user's database.
- **No "search the whole database contents"** (full-text over table data) — that is a different
  product decision with its own permission and performance implications.
- **No snippet execution without review.** Inserting a snippet places SQL in the editor; it
  never runs.
- **No settings that silently weaken safety.** Read-only defaults, destructive confirmations, and
  environment behavior are not user-disableable through ordinary settings (they may be relaxed
  only through the connection's own explicit flags, per master goal §10).
- **No new editor.** The snippet/parameter features reuse the existing `CodeEditor`.

## 4. Current implementation (code-verified baseline)

| Element | Reality | Evidence |
|---|---|---|
| Activity rail | 7 activities, no Search | `ui/src/app.rs:114-123`; `navigation_view.rs:412-460` |
| Topbar search box | Palette launcher only | `navigation_view.rs:174-225` |
| Palette | Quick Open (tables ≤100 + static items) + Commands; `PaletteAction`/`PaletteItem` | `palette_view.rs:150-163, 190-241, 294`; `app.rs:131-159` |
| Search index | **Absent** | no index type, no meta-store table |
| Snippets | 2 hardcoded buttons; no persistence | `query_view.rs:276-288`, insert at `:1518` |
| Scratch SQL | **Absent** | — |
| Favorites / pinned / recent | Favorites: connection domain only, dropped at DTO; pinned/recent-object: absent | `translate.rs:41-42`; `capability matrix §13` |
| Saved queries + folders | Works (7 repo methods) | `meta/saved_query_repo.rs`; `query_service.rs:410-479` |
| Query history | Local dual (20 in-memory + 500 persisted) | `ui/src/app.rs:286-288`; `dbpro.native.query-history-v1` |
| Editor flags | Per-query overflow menu; only prediction persisted | `query_view.rs` overflow; `dbpro.native.prediction-mode` |
| Grid behavior settings | Layout only | `dbpro.native.grid-layouts`, `grid-widths`, `grid-widths-customized` |
| Connection defaults | Per-connection fields exist; no defaults surface | `domain/connection.rs:104-111` |
| `RunConfigRepository` | Exists, unwired to native | `meta/run_config_repo.rs`; `query_service.rs:483-509`; absent from `UiCommand`/`RuntimeCommand` |
| Keybindings | Hardcoded match; `Ctrl+N` advertised/unwired | `events.rs:1102-1160`; `navigation_view.rs:584`; `workspace_view.rs:533-536` |
| Query parameters | `QueryParam` exists in core; **no UI** | `domain/query.rs:24-51`; binary-cell hint at `table_editor_view.rs:798` |
| JSON viewer | Expanded cell editor + pretty copy; no tree | `result_grid_view.rs` cell editor |
| `SearchInput` component | Exists and is used | `components/input.rs` |
| Persisted-but-unread keys | `dbpro.native.connections-pane-height`, `dbpro.native.schemas-pane-height` are saved and loaded but never read by a render path | `app.rs:432,475`; `app_state.rs:62,285` |

## 5. UX

### 5.1 Global search (`⌘K` / `Ctrl+K`)

One surface, two modes (the palette and the Search activity share an index and a result model):

```text
┌ Search ──────────────────────────────────────────────────────────┐
│ 🔍 customer_id                                    [ All ▾ ]     │
├──────────────────────────────────────────────────────────────────┤
│ COLUMNS                                                          │
│   customer_id          orders.customer_id        local-pg/public │
│   customer_id          invoices.customer_id      local-pg/public │
│ TABLES                                                           │
│   customers            public.customers          local-pg/public │
│ SAVED QUERIES                                                    │
│   Monthly revenue      … WHERE customer_id = $1  Queries/Reports │
│ HISTORY                                                          │
│   SELECT … customer_id …  (2h ago)                               │
│ COMMANDS                                                         │
│   New query on local-pg                                          │
└──────────────────────────────────────────────────────────────────┘
   ↑↓ navigate · ↵ open · ⌘↵ open in new tab · ⇥ filter type
```

Rules:

- Results are grouped by type with keyboard navigation across groups; `Tab`/`⇧Tab` cycles the
  type filter.
- Every result shows its **target context** (connection · database · schema) — never a bare name.
- Actions are explicit and typed: "Open table data", "Open structure", "Open definition",
  "Explain current SQL", "New query on \<connection\>", "Search column customer_id"
  (which filters to columns), "Export current results", "Show active queries" (Phase D
  entry, disabled/absent when the provider lacks it), "Run command".
- The index is connection-scoped by default (the active connection) with an "all connections"
  toggle; indexing every connection's full catalog eagerly is explicitly avoided.
- Empty state shows recent and favorites rather than a blank panel.
- No result, no "did you mean" guessing: an empty result says "no matches in \<scope\>" with the
  scope and a link to widen it.

### 5.2 Search activity (G02)

The rail gains `⌕ Search`. The sidebar shows result groups and filters (type, connection,
favorites only); the workspace area shows the result list with a preview pane:

- Preview for an object: its metadata summary + DDL (via `DDLViewer`).
- Preview for a saved query/history: the SQL with "Open" and "Open in new tab".
- Preview for a command: a short description and its keybinding.
- Preview for an agent action: what it will do and its risk class.

### 5.3 Queries activity organization (G03)

```text
QUERIES
  Open Queries          (existing documents)
  Saved Queries         (folders → queries; tags as chips; drag or menu to move)
  Favorites             (queries + objects marked as favorite)
  Recent                (objects and queries, most recent first)
  Query History         (existing, with filter + favorites)
  Snippets              (library: folders → snippets; insert into editor)
  Scratch SQL           (named scratch documents)
  Templates             (parameterized starting points; share the snippet model)
```

### 5.4 Snippets (G03)

```text
SELECT * FROM {{table}}
WHERE {{column}} = {{value}}
LIMIT {{limit:100}};
```

- Variables are declared inline with optional defaults and optional type hints; insertion opens
  a small prompt form (or inserts placeholders as a block selection when no values are supplied).
- **Values are inserted as text into the editor, never executed, and never sent to the database
  as literals by the snippet system.** When the user runs the query, the existing classifier and
  safety rules apply.
- Snippets are shareable via clipboard/export (a small JSON document) and importable with
  validation.

### 5.5 Settings-driven personalization (G04)

Delivered by K01's Settings Center; G04 fills in the behavioral settings and the keybinding
editor:

- **Editor**: font size, tab size, insert spaces, word wrap, show line numbers, completion
  (on/off/manual), prediction mode, format on run, diagnostics verbosity.
- **Data Grid**: default row height, page size, NULL display string, copy format (CSV/TSV/JSON/
  INSERT), date/number format, sticky header, zebra rows.
- **Connections**: default timeout, default max rows, default read-only, default environment,
  default schema.
- **Keybindings**: a table of commands with their bindings; conflict detection; reset to
  defaults; a searchable cheatsheet.

### 5.6 Paper-cut fixes (G04)

| Paper cut | Fix |
|---|---|
| Copy ignores staged values in one path | Make all copy paths respect the staged `ChangeSet` (the capability matrix notes the inconsistency) |
| JSON viewer has no tree | Add a read-only tree view beside the text editor for `Json` cells |
| `DEFAULT` has no apply affordance | Add an explicit "set to DEFAULT" cell action where the column has a default |
| `Ctrl+N` advertised but unwired | Wire it (new query) or remove the hint — no third option |
| Persisted-but-unread pane-height keys | Wire or remove |
| Query parameters have no dialog | G04's parameter dialog (also used by Phase B's routine execution form) |

## 6. Architecture

### 6.1 The unified searchable item model

```rust
pub enum SearchItemKind {
    Connection, Database, Schema, Table, View, MaterializedView, Column,
    Routine, Sequence, Type, SavedQuery, HistoryQuery, Command, AgentAction, Setting,
}

pub struct SearchItem {
    pub id: SearchItemId,             // stable within a connection scope
    pub kind: SearchItemKind,
    pub title: String,                // primary display text
    pub subtitle: Option<String>,     // e.g. "public.orders"
    pub scope: ItemScope,             // connection_id, database, schema
    pub keywords: Vec<String>,        // identifiers, aliases, tags
    pub payload: ItemPayload,         // typed reference (ObjectRef, SavedQueryId, CommandId…)
    pub flags: ItemFlags,             // favorite, pinned, recent_rank
}
```

Index storage: a meta-store table (`search_items`) with a **normalized search column**
(lowercased title + subtitle + keywords) and a scope column, plus a `search_meta` row per
connection holding the last-indexed introspection generation. Ranking is deliberately simple and
explainable: exact match > prefix match > word-prefix > substring, with favorites/pinned boosted
and recency as a tiebreaker. Fuzzy/substring matching is done in SQL over the normalized column
with a bounded result count; when a connection has more items than the scan budget, the query
narrows by kind first (the UI's type filter is passed down).

Why not an in-memory index: the item count can reach tens of thousands of columns across
connections, and the meta store is already the durable home for cross-session data. Why not a
background push of every row into memory: startup cost and memory churn for a feature used
intermittently.

### 6.2 Index lifecycle

```text
Introspection completes (SchemaApi::introspect / introspect_summary)
   → SearchService::reindex_connection(connection_id, IntrospectResult)
       - upsert items for schemas/tables/columns/views/routines/sequences/types
       - mark removed items (delete by scope + missing generation)
   → search_meta.generation = introspection generation

Schema-change event (Phase A apply, Phase F apply, any DDL execution)
   → invalidate scope → reindex on next introspection (not eagerly)

Saved query / history / settings changes
   → upsert the specific item (no full reindex)

Connection deletion
   → delete all items in that scope
```

Rules:

- Client-side (meta store) indexing only; **no database queries** beyond what introspection
  already performs.
- Bounded: the feature declares a per-connection item budget (for example 50,000 items) and
  degrades by indexing tables/views/routines always and columns for the first N tables, with a
  visible "column search is partial for this connection" note rather than silent truncation.
- Secret-free: `SearchService` accepts only the introspection result and user-authored text; a
  test asserts that no connection secret, password, or bound parameter value can be inserted
  into the index (the insert API does not accept secret-bearing types).

### 6.3 Command registry

Commands become data so that palette, Search, keybindings, and the cheatsheet share one source:

```rust
pub struct CommandDescriptor {
    pub id: CommandId,
    pub title: String,
    pub description: String,
    pub category: CommandCategory,
    pub default_binding: Option<KeyBinding>,
    pub availability: CommandAvailability,   // capabilities + context predicates
    pub risk: CommandRisk,                   // None | Mutation | Destructive (for display)
    pub action: CommandAction,               // constructs a UiCommand
}
```

The palette and the Search index are both generated from this registry, so a new command is
discoverable and bindable in one change. Availability is capability- and context-aware (for
example "Show active queries" is unavailable on SQLite), which also fixes the current situation
where commands exist without any discoverable description.

### 6.4 Settings model (with K01)

G04 does not create a second settings system. It consumes the K01 model:

- Behavior settings → `Settings` (meta store) read by the runtime and pushed to the UI.
- View/layout state → eframe storage (existing pattern, versioned).
- Per-connection defaults → `ConnectionConfig` (domain) with defaults applied at creation.
- Keybindings → `Settings` (a map of `CommandId` → binding) resolved at event time, with the
  hardcoded table retained as defaults only.

`RunConfigRepository` is decided in K01: either it becomes the per-connection run-configuration
surface behind the settings UI, or it is deleted. Leaving it unwired is not an acceptable end
state (master goal §2.5 C13).

## 7. Domain model

New module `crates/core/src/domain/workspace.rs` (product-organizational entities; keep names
distinct from `domain/history.rs`, which already owns `SavedQuery`/`QueryHistory`):

```rust
pub struct Favorite { pub subject: FavoriteSubject, pub created_at: OffsetDateTime }
pub enum FavoriteSubject { Object(ObjectRef), Query(SavedQueryId), Connection(ConnectionId) }

pub struct Pin { pub subject: FavoriteSubject, pub ordinal: u32 }

pub struct RecentEntry { pub subject: FavoriteSubject, pub last_used: OffsetDateTime, pub use_count: u32 }

pub struct Snippet {
    pub id: SnippetId,
    pub name: String,
    pub folder: Option<String>,
    pub body: String,                       // text with {{variable}} placeholders
    pub variables: Vec<SnippetVariable>,    // parsed from body, editable
    pub tags: Vec<String>,
    pub updated_at: OffsetDateTime,
}

pub struct SnippetVariable { pub name: String, pub default: Option<String>, pub hint: Option<String> }

pub struct ScratchDocument { pub id: ScratchId, pub name: String, pub sql: String,
                             pub connection_id: Option<ConnectionId>, pub updated_at: OffsetDateTime }

pub struct QueryTag { pub name: String, pub color: Option<String> }
```

Repository ports: `WorkspaceRepository` already exists in a minimal form
(`ports/workspace_repository.rs`: save/list/delete) — extend it or add
`WorkspaceOrganizationRepository` with `favorites`, `pins`, `recents`, `snippets`,
`scratch_documents`, `query_tags`. Decide in G03 and document; do not leave two overlapping
ports.

## 8. APIs, services, ports

### 8.1 SearchService

```rust
impl SearchService {
    pub async fn query(&self, request: SearchRequest) -> Result<SearchResponse, DbError>;
    pub async fn reindex_connection(&self, connection_id: ConnectionId, result: &IntrospectResult) -> Result<ReindexStats, DbError>;
    pub async fn invalidate_scope(&self, scope: ItemScope) -> Result<(), DbError>;
    pub async fn upsert_query_item(&self, item: SearchItem) -> Result<(), DbError>;
    pub async fn recent(&self, scope: Option<ItemScope>, limit: usize) -> Result<Vec<SearchItem>, DbError>;
    pub async fn favorites(&self, scope: Option<ItemScope>) -> Result<Vec<SearchItem>, DbError>;
}

pub struct SearchRequest {
    pub text: String,
    pub kinds: Vec<SearchItemKind>,      // empty = all
    pub scope: SearchScope,               // ActiveConnection | Connection(id) | AllConnections
    pub favorites_only: bool,
    pub limit: usize,                     // bounded, default 50
}
```

### 8.2 CommandRegistry (UI-side but shared)

Lives in `crates/ui/src/commands.rs` (new) because a command constructs a `UiCommand`:
`CommandDescriptor` list, `availability(context, capabilities)`, and `binding(settings)`.
The palette, Search, keybinding editor, and cheatsheet all read it. No command executes a
database operation directly — it constructs a `UiCommand` like any other UI action.

### 8.3 Runtime/UI wiring

```text
UiCommand::SearchQuery          { request_id, request }
UiCommand::SearchRecent         { request_id, scope, limit }
UiCommand::SearchFavorites      { request_id, scope }
UiCommand::ReindexSearch        { request_id, connection_id }         // manual refresh
UiCommand::ListSnippets         { request_id }
UiCommand::SaveSnippet          { request_id, snippet }
UiCommand::DeleteSnippet        { request_id, id }
UiCommand::ListScratchDocuments { request_id }
UiCommand::SaveScratchDocument  { request_id, doc }
UiCommand::SetFavorite          { request_id, subject, on }
UiCommand::SetPinned            { request_id, subject, on, ordinal }
UiCommand::RecordRecent         { request_id, subject }               // fire-and-forget
UiCommand::ListRunConfigs       { request_id, connection_id }         // unwire/dewire decision from K01
UiEvent::SearchResults          { request_id, response }
UiEvent::SnippetsLoaded / ScratchDocumentsLoaded
UiEvent::FavoriteChanged / PinnedChanged
```

`RecordRecent` is emitted on object/query open (not on every keystroke) and debounced.

### 8.4 Keybinding resolution

`handle_shortcuts` (`events.rs:1102-1160`) becomes a lookup:

```text
KeyEvent → binding table (Settings) → CommandId → CommandDescriptor → availability → UiCommand
```

Conflicts are detected at settings-save time and reported; a conflict leaves the previous
binding intact rather than silently dropping one.

## 9. Provider behavior

| Concern | PostgreSQL | SQLite |
|---|---|---|
| Indexable object kinds | schemas, tables, views, matviews, columns, routines, sequences, types | tables, views, columns, indexes (no schemas/routines/sequences/types) |
| Index source | `IntrospectResult` (existing) + Phase A catalog additions | `IntrospectResult` (existing) |
| Connection organization | provider-neutral | provider-neutral |
| Snippets/scratch/parameters | provider-neutral; the parameter dialog binds through `QueryParam` (PG `$n`) | provider-neutral; `?` placeholders — the parameter dialog must be placeholder-aware (`numbered_parameters` vs `positional_parameters` flags exist at `capabilities.rs:31-37`) |
| Monitoring command availability | available after Phase D | unavailable, with reason in the command registry |
| Capability flags | none new | none new |

No new provider capability flags are needed; the existing query-parameter flags already describe
the binding difference, and the command registry's availability predicates use the existing
capability struct.

## 10. UI structure

### 10.1 Components

| Need | Component | Status |
|---|---|---|
| Search input | `SearchInput` (`components/input.rs`) | reuse, add scope + type filter |
| Result list with groups | `ObjectList`/`ObjectTree` | reuse/extend |
| Result preview | `PropertyGrid` + `DDLViewer` + `CodeBlock` | reuse (Phases A/E) |
| Command palette | promote `palette_view.rs` logic into a `components/` widget | promote |
| Keybinding capture widget | new small widget (records a chord, validates) | new |
| Settings pages | K01's settings shell | reuse |
| Tags/chips | `tag_chip` (`components/legacy.rs`), `Badge` | reuse |
| Snippet editor | `CodeEditor` + a variables panel | reuse |

### 10.2 Screens

1. **Search activity + palette** (shared result model).
2. **Queries activity** with the new sections (§5.3).
3. **Snippet library** (list, editor with variable parsing, insert action, import/export).
4. **Scratch SQL manager**.
5. **Settings → Editor / Data Grid / Connections** (K01 shell, G04 content).
6. **Settings → Keybindings** with capture, conflicts, reset, cheatsheet.
7. **Query parameters dialog** (editor action and inline hint fix).
8. **Context-menu additions**: Favorite/Pin on objects, queries, connections; "Move to folder";
   "Add tag".

### 10.3 Interaction rules

- The palette opens instantly (index query is bounded) and never blocks the frame; the first
  keystroke is responsive even before results arrive (a small "searching" state, not a freeze).
- Search results are keyboard-first: `↑↓` move, `↵` primary action, `⌘↵` secondary action
  ("open in new tab" where meaningful), `Esc` closes.
- Recently used items appear before typing (the empty state), so `⌘K ↵` reopens the last thing.
- Every favorite/pin/recent action is immediate and shows in the relevant list without a
  refresh action.
- Snippet insertion never replaces a selection silently: insert at cursor, or wrap the selection
  when the snippet has a single variable and the selection is non-empty (with the behavior stated
  in the snippet editor).

## 11. Safety model

| Action | Class | Notes |
|---|---|---|
| Search / browse results / open object | `ReadOnly` | No database writes |
| Reindex | `ReadOnly` | Reads local introspection only; explicit user action |
| Snippet insert | `ReadOnly` | Inserts text; **never executes** |
| Favorite / pin / recent | `ReadOnly` | Local meta-store writes only |
| Scratch document save | `ReadOnly` | Local only |
| Commands that mutate | Follow the target operation's class | The registry displays the risk but the target flow enforces it |
| Parameter dialog | `ReadOnly` (it only prepares a statement) | Values are bound as `QueryParam`; never interpolated into SQL text |

Additional rules:

- **Index secrecy**: the index must never contain passwords, secrets, connection strings with
  credentials, bound parameter values, or result data. Enforced by the insert API's types plus a
  test that attempts to index a `ConnectionSecret` and fails to compile / is rejected.
- **Query text is user content**: saved-query and history text are indexed because the user wrote
  them for reuse; the UI must not index anything the user did not author (for example, it must
  not index query text from an agent tool call that the user never accepted).
- Settings that affect safety (read-only defaults, destructive confirmations) are not relaxed by
  personalization; they are connection flags (I01) or not configurable at all.
- Keybinding remapping must not allow binding a destructive command to a single unmodified key:
  the editor refuses single-key bindings for commands whose risk is `Destructive`.

## 12. Testing strategy

### 12.1 Unit

- Ranking: exact > prefix > word-prefix > substring; favorite/pinned/recent tiebreakers; stable
  ordering for equal scores; bounded results.
- Index lifecycle: upsert, delete-by-scope, generation tracking, invalidation on schema change,
  item budget degradation with the partial-coverage flag.
- **Secret-free assertion**: the index insert path cannot accept secret-bearing types; a test
  writes a curated item set and asserts no field contains a secret marker.
- Command registry: every registered command has a unique id, a title, a category, and a valid
  availability predicate; every `PaletteAction` is reachable from the registry.
- Keybinding resolution: binding lookup, conflict detection, single-key refusal for
  `Destructive` commands, reset to defaults.
- Snippet variable parsing: `{{name}}`, `{{name:default}}`, repeated variables, escaping, invalid
  syntax reporting.
- Parameter binding: numbered vs positional placeholders per driver; values never appear in the
  SQL string (assertion on the generated statement).

### 12.2 Service / integration

- SQLite (always-run): reindex a fixture, query it, invalidate after a DDL change, re-query;
  favorites/pins/recents/snippets/scratch round-trip through the meta store; migration idempotency.
- PostgreSQL (live): reindex a real schema (including routines/sequences/types where Phase A
  exists), query by column name, verify `NotCompared`-style gaps are surfaced when a kind is not
  indexed.
- Scale: index a 1,000-table fixture and assert query latency stays within the interactive
  budget; assert the item budget degradation message appears when configured low.

### 12.3 UI

- Palette: keyboard-only flow, grouping, filters, empty/no-match state, `⌘↵` behavior.
- Search activity: scope switching, result preview, actions.
- Snippet insert: variable prompt, insert-at-cursor, wrap-on-selection.
- Settings: each editor/grid/connection setting persists and takes effect; reset works.
- Keybindings: capture, conflict, reset, and the destructive-single-key refusal.
- Parameters dialog: type widgets, NULL, binding per driver, error rendering.
- Paper cuts: staged-value copy correctness, JSON tree, `DEFAULT` action, `Ctrl+N` consistency.

### 12.4 Regression

- No secret in the index (security test, repeated at the service level).
- No settings change can disable the read-only policy or destructive confirmations.
- Existing palette commands keep working after the registry refactor (command-coverage test).

## 13. Runtime verification

**G01**

1. `⌘K` → type a column name → capture the grouped result with target context → open it.
2. `⌘K` → find a saved query by a fragment of its SQL → open it.
3. `⌘K` → find a history entry by SQL text → open it.
4. `⌘K` → run a command from the registry (for example "New query on \<connection\>") → capture
   the result.
5. Capture the no-match state showing the scope and the widen action.

**G02**

1. Capture the Search activity with type filters and a preview pane for an object, a query, and a
   command.
2. Run DDL (Phase A) to change the schema; capture that the index reflects the change after
   re-introspection without a manual reindex.
3. With a synthetic large schema, capture the bounded result behavior and (with a low budget
   configured) the partial-coverage note.

**G03**

1. Favorite a table, a query, and a connection; capture them in the Favorites filter and in the
   connection list.
2. Pin an object; capture it at the top of the relevant list after restart.
3. Create a snippet with two variables; insert it into the editor with a prompt; capture the
   inserted SQL (not executed).
4. Create two scratch documents; restart the app; capture them restored.
5. Tag a saved query; move it between folders; capture the result.
6. Wire connection folders/tags/favorites/color (the DTO fix): create a folder, tag two
   connections, favorite one, set a color; restart; capture the preserved organization.

**G04**

1. Change editor font size, grid NULL display, and default connection timeout; restart; capture
   the applied values in a real editor/grid/new connection.
2. Rebind a command; capture the cheatsheet and the conflict warning for a deliberate conflict.
3. Attempt to bind a destructive command to a single key; capture the refusal.
4. Open the query parameters dialog; execute a parameterized query on both providers with typed
   values including NULL; capture the bound execution and the absence of interpolation.
5. Capture the paper-cut fixes: staged-value copy, JSON tree view, `DEFAULT` action, `Ctrl+N`.

## 14. Milestone order

| Order | ID | Title | Release | Pri | Prerequisites | Rationale |
|---:|---|---|---|---|---|---|
| 1 | G01 | Palette & Quick Open Search Expansion | v0.2 | P1 | none | No prerequisites, high visibility, and it validates the searchable-item model cheaply before the index and activity exist |
| 2 | G02 | Search Activity + Unified Index | v0.4 | P2 | G01, §4.6 meta store, Phase A object kinds | Needs the item model proven by G01, the durable index, and the object kinds that Phase A adds |
| 3 | G03 | Workspace Organization (favorites, pinned, recent, snippets, scratch, tags, connection organization) | v0.4 | P2 | G02 (index reuse) | Favorites/pinned/recent are filters over the index; snippets/scratch are independent but share the meta-store work, so one migration covers them |
| 4 | G04 | Editor, Grid & Connection Customization (+ paper cuts, parameters dialog) | v0.4 | P2 | K01 | Consumes the Settings Center; also the natural home for the parameter dialog shared with Phase B |

Ordering rules: G01 is intentionally first and independent — it ships a visible win in v0.2
without waiting for the Phase G infrastructure. G02 and G03 share a meta-store migration; if
they are split across PRs, the migration must land with G02 and be additive for G03. G04 must
not start before K01 exists (otherwise it recreates the ad-hoc settings problem it is meant to
solve).

## 15. Definition of done

1. **Plan folder** per milestone; `docs/plans/STATUS.md` matches.
2. **P0 = 0, P1 = 0.**
3. **Search finds the six required classes**: tables, columns, views, routines, saved-query SQL,
   history SQL — plus commands, with target context on every result. Verified by test and runtime
   evidence.
4. **The Search activity is real** and the topbar box is no longer the only entry point; no
   placeholder or unimplemented rail icon is introduced (master goal §3.3).
5. **Index is durable, bounded, and incremental**: rebuilt from introspection, invalidated by
   schema-change events, degraded with a visible note at the budget limit, and never populated by
   a background timer.
6. **Index is secret-free** — proven by a test that asserts no credential/bound-value/result data
   can enter the index.
7. **Commands are data**: the palette, Search, keybindings, and cheatsheet all read one registry;
   every command has a description and an availability predicate; the `Ctrl+N` inconsistency is
   resolved.
8. **Keybindings are remappable** with conflict detection and reset; destructive commands cannot
   be bound to a single unmodified key.
9. **Organization features persist across restart**: favorites, pins, recents, snippets, scratch
   documents, query tags and folder moves, and connection folders/tags/favorites/color.
10. **Settings take effect**: editor, grid, and connection defaults are persisted, validated,
    resettable, and applied without a restart where feasible.
11. **Parameters dialog** binds values as `QueryParam` per provider placeholder style with no
    interpolation (asserted), and the misleading binary-cell hint is resolved.
12. **Paper cuts fixed**: staged-value copy, JSON tree, `DEFAULT` action, unwired pane-height
    keys, and the `Ctrl+N` hint.
13. **No safety regression**: no setting or personalization path can weaken the read-only policy
    or destructive confirmations; a test asserts it.
14. **Quality gates executed and recorded** (fmt, clippy with stated tauri scope,
    `cargo test --workspace`, SQLite suite, PostgreSQL live suite where the milestone touches PG
    behavior).
15. **Docs updated**: capability matrix §13/§14 rows move off MISSING, master goal §9 and the
    component contract (§4.5) reflect the promoted components, and the `PRODUCT_ROADMAP.md`
    invented-component names are replaced by the real contract names (master goal §2.5 C9).
16. **Non-goals respected**: no content search, no background crawling, no snippet execution, no
    cloud sync, no third settings mechanism.

Phase G is complete when a user can find any object, column, query, or command in two
keystrokes with its target context; organize work with favorites, pins, snippets, scratch
documents, and tags that survive restarts; see connection folders/tags/favorites that the domain
already supported; and tune the editor, grid, connections, and keybindings through a single
settings surface — with the search index proven to contain no secrets and no setting able to
weaken the product's safety guarantees.
