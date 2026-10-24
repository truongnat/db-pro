---
schema_version: 1
id: "MEM-SRC-e005112db13c4251"
title: "Source mirror: docs/user-guide/admin-files-agent.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/user-guide/admin-files-agent.md"
source_sha256: "e005112db13c425190aaff78c4c42e63b11f1577318a3c3c782e72cfcd896c0b"
mirror_body_sha256: "e005112db13c425190aaff78c4c42e63b11f1577318a3c3c782e72cfcd896c0b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Administration, Files & AI Agent

This guide covers DB Pro's administrative surfaces: **Settings**, **Server Monitor** (with audit, replication, FDW, event triggers, and server settings), **Security & Roles**, **Saved Tasks**, **Data Transfer**, the **Files workspace**, and the **Agent panel** (AI copilot).

Most admin surfaces live in the sidebar. To open them, click the **Tools & Management Hub** icon (boxes) in the left activity bar and pick a tool card:

| Group | Card | Opens |
|---|---|---|
| Schema & Migration | Schema & Data Compare, Schema Workbench, ER Diagram | See the Table Workbench guide |
| Administration & Server | **Server Monitor**, **Security & Roles**, **Execution History & Audit** | Monitor / Security / History sidebar activities |
| Data & Automation | **Data Transfer & Migration**, **Saved Tasks & Backups**, **Diagnostics & Problems** | Transfers / Tasks / Problems |

A search box at the top of the hub filters the cards. The same activities are also reachable from the **Command Palette** (`Ctrl/Cmd+Shift+P`).

The **Files** workspace shares the **SQL Scripts & Projects** rail icon; it appears in the sidebar after you open a folder (see below). The **Agent** panel is toggled with the sparkles icon in the activity bar.

---

## 1. Settings

Open **Settings** via the gear icon at the bottom of the activity bar, or via a tool/palette entry. The screen is a two-pane layout: a **SETTINGS** navigation column on the left and a scrolling content pane on the right. At the bottom of every section there is also a **DIAGNOSTICS** card (see §1.11).

Section groups in the navigation:

- **Workspace** — General, Appearance, Editor, Data Grid
- **Connections** — Connections, AI Providers
- **System** — Keybindings, Backup / Restore, Security, Advanced

All settings are persisted locally as versioned JSON (`settings-v1` schema); out-of-range values are clamped on load (font size 10–24, tab width 2–8, grid page size 25–1000). Passwords are never written to this file — they live in the secret store.

### 1.1 General

- **Language** — button row to switch the UI language.
- **Confirm destructive queries** — require confirmation before running destructive SQL.
- **Restore query tabs on startup** — reopen the previous session's query tabs.

**Workspace sessions**: named sessions save the *layout and tab references* — not SQL text, secrets, or result grids. Type a name in **Session name**, click **Save**, then manage entries with **Restore**, **Duplicate**, **Delete**. If a restore had caveats (e.g. missing tabs), notes are listed below.

### 1.2 Appearance

- **Light / Dark** radio toggle — applies immediately.
- **Reduce motion** — replaces spinners with a static status icon on loading states.

### 1.3 Editor

- **Font size** — `−` / `+` steppers, 10–24 px.
- **Tab width** — drag value, 2–8.
- **Schema completion** — toggle the completion popup.
- **Format SQL on save**.
- **AI prediction** — `Off` / `Subtle` / `Eager`. Sends the SQL around your cursor plus schema context to the configured AI provider.

**SQL Lint** — master toggle **Enable SQL lint warnings**, then per-rule switches: *Warn on `SELECT *`*, *`= NULL / != NULL`*, *`DELETE` without `WHERE`*, *`UPDATE` without `WHERE`*, *`ORDER BY` ordinal*, *comma / cartesian joins*, *duplicate projection aliases*. Lint runs locally — it never calls AI or the network. Individual rules can also be suppressed per code (e.g. `lint.comma-join`).

### 1.4 Data Grid

- **Page size** (25–1000), **Show row numbers**, **Wrap cell text**.

### 1.5 Connections

- **Reconnect last connection on startup**.
- **Prefer TLS for new server connections**.
- Note: passwords stay in secret storage, never in the settings JSON.

### 1.6 AI Providers

- **Enable Agent workspace**, **Allow Agent to auto-run read-only queries**.
- Shows the **Active provider** label. API keys are configured in the **Agent panel** (gear icon → API Key, §6) and stored as secrets.

### 1.7 Keybindings

A searchable list of every remappable command with its resolved shortcut. Type in **Search commands…** to filter by title or command id.

- Click **Edit** on a row, type a token like `mod+k`, `mod+shift+p`, `f5`, `mod+/` (accepted modifiers: `mod`/`cmd`/`ctrl`, `shift`, `alt`; keys: letters, digits, F1–F12, Enter, Escape, Delete, `/`). Press Enter or **Save**. Clearing the field or re-entering the default removes the override.
- **Reset** (per row, shown when overridden) and **Reset all to defaults**.
- Conflicting shortcuts are flagged with a warning count (`N keybinding conflict(s) detected`) and the affected rows are highlighted.

Default catalog (effective unless overridden):

| Command | Default |
|---|---|
| Quick Open / (alternate) | `mod+p` / `mod+k` |
| Command Palette | `mod+shift+p` |
| Toggle Sidebar | `mod+b` |
| Find in Editor | `mod+f` |
| Save Query / Save As | `mod+s` / `mod+shift+s` |
| Run Query / Run Script | `F5` / `mod+shift+enter` |
| Format Query | `mod+shift+f` |
| New Connection / New Query | `mod+n` / `mod+t` |
| Open Table | `mod+o` |
| Focus Results | `mod+shift+r` |
| Toggle Comment | `mod+/` |

(`mod` = Ctrl on Linux/Windows, Cmd on macOS.)

### 1.8 Backup / Restore

Available only when the active provider supports it; otherwise the card says *"Backup and restore are unavailable for the active provider."*

- **Backup destination** path field → **Choose path** (native file dialog) → **Create backup**.
- **Restore from backup** path field → **Choose file** → **Restore database** → confirm in **"Overwrite the active database?"** (**Confirm restore** / **Cancel**).

Driver notes shown in the UI:

- **SQLite** — `VACUUM INTO` snapshots (WAL-safe); restore is refused while the connection is active — disconnect first.
- **PostgreSQL** — requires `pg_dump` on PATH for backups; restores use `psql` (plain) or `pg_restore` (custom). Missing tools are detected before anything runs.
- **MySQL** — not available yet.

### 1.9 Security

- **Redact secrets in diagnostics and logs** (on by default).
- **Block secret export by default** (on by default).

### 1.10 Advanced

- **Verbose runtime logging**, **Experimental features**, and the read-only settings schema version.

### 1.11 Diagnostics (shown under every section)

Shows `DB Pro <version> · <os>/<arch>`, driver availability, connection count, active executions, and up to 5 recent structured errors.

| Button | Effect |
|---|---|
| **Copy diagnostics summary** | Copies the summary JSON to the clipboard (secrets redacted) |
| **Export support bundle** | Writes a redacted support bundle to disk; path reported in the status message |

---

## 2. Server Monitor

Open via **Tools hub → Server Monitor**. Requires a connected database ("Connect from Explorer to monitor sessions." is shown otherwise). The header shows **Connected/Disconnected**, the connection name and driver, an **Auto-refresh** checkbox (polls every ~5 s), and a **Refresh** button.

### 2.1 Health Advisor

Deterministic heuristics over the snapshot — it never mutates the database. Each finding shows a severity badge (critical/warning/info), affected objects, evidence, explanation, and a suggested action. "No findings" when clean.

### 2.2 Sessions

- **Active queries only** filter (on by default).
- An **IDLE IN TRANSACTION** block lists up to 20 idle-in-xact sessions with `pid`, `xact_age`, `backend_age`, user.
- Each session card: `pid`, `current` badge, state, `query N ms`, `xact N ms`, `idle-in-xact` badge, user/database/application, truncated query text, and actions:

| Button | Effect |
|---|---|
| **Open SQL** | Loads the session's query into the Query editor |
| **Cancel** | Confirms via *"Cancel backend pid N?"* → **Cancel backend** / **Keep running** |
| **Terminate** | Confirms via *"Terminate session?"* (disconnects the client) → **Terminate** / **Cancel** |

Your own session (`current` badge) cannot be cancelled/terminated from here.

### 2.3 Server, Locks, Sizes

- **SERVER** — version, current database, connections used/max, database size.
- **LOCKS / BLOCKERS** — blocking lock pairs (locked pid, granted/waiting, blocker pid, mode, relation), up to 30.
- **SIZE / STATS** — top 20 relations: `schema.name (kind) · size · seq/idx scans · dead tuples`.
- **LOCAL STATE** (SQLite) — journal mode, page count/size, freelist, file size.

### 2.4 Top Queries (pg_stat_statements)

Shown when the snapshot includes workload data; warns if the extension is missing.

- Sort buttons: **total time · mean time · calls · rows**.
- **Filter** box matches query text, database, or user.
- Up to 40 statements listed with calls/total/mean/rows, shared-buffer hit/read, temp r/w, and a `Δ total` delta vs. the previous snapshot.
- **Open SQL** loads the normalized statement into the editor.
- **Reset stats…** opens *"Reset pg_stat_statements?"* — clears all accumulated stats on the server (**Reset statistics** / **Cancel**).

### 2.5 Maintenance

Schema/table fields plus **VACUUM**, **ANALYZE**, **VACUUM ANALYZE** buttons. Each opens *"Run maintenance?"* warning that long-running VACUUM can take locks — confirm or cancel.

### 2.6 Auxiliary PostgreSQL surfaces (below the monitor snapshot)

| Section | What it does |
|---|---|
| **Database audit (server CSV log)** | Reads the server's CSV log if you configure the connection tag `audit:csvlog=/path/to/logfile.csv` (never auto-enabled). Filters: Text, DB, User, Severity. **Load page**, **Export selected**, per-event select/bookmark, **Open SQL**. Shows scanned bytes and truncation state. |
| **FOREIGN DATA (FDW)** | **Load FDW inventory**; create a foreign server via **Preview CREATE** → **Create (confirm)**; drop with preview (**Drop CASCADE** confirmed). Passwords/options redacted; `CREATE EXTENSION` is never auto-run. |
| **LOGICAL REPLICATION** | **Load replication inventory**: publications (with tables, owner), subscriptions (redacted conninfo), replication slots. **Preview DROP** / **Drop…** each; create a publication `FOR ALL TABLES` via **Preview CREATE** → **Create (confirm)**. `CREATE SUBSCRIPTION` is not offered (it would need secrets). |
| **EVENT TRIGGERS** | **Load event triggers**; create one (requires an existing function) via **Preview CREATE** → **Create (confirm)**; **Drop** with confirm. PostgreSQL-only, database-level DDL hooks — not table/row triggers. |
| **SERVER SETTINGS (pg_settings)** | Filter server GUCs; session `SET`/`RESET` for user-context settings; `ALTER SYSTEM` shown as preview-only. |

Non-PostgreSQL drivers show a notice instead ("MySQL has no sessions", etc.).

---

## 3. Security & Roles

Open via **Tools hub → Security & Roles**. PostgreSQL-only and capability-gated: other drivers show *"User/role management is PostgreSQL-only"*; disconnected shows *"Connect a PostgreSQL database to manage roles and privileges."*

### 3.1 Roles / Users

**Refresh roles** loads the list. Each role row shows capability badges (`login`, `super`, `createdb`, `createrole`) and a **Drop** button → dialog *"Drop role `name`? This cannot be undone."*

**Create role**: enter a name, optional **LOGIN** checkbox → **Create role**.

### 3.2 Role details (click a role)

- **ATTRIBUTES** — one-click `LOGIN`, `NOLOGIN`, `CREATEDB`, `CREATEROLE`, `NOCREATEDB`, `NOCREATEROLE` (alter-role, confirmed via dialog).
- **Password** — set a new password for the role (**Update password**).
- **Memberships** — lists `member of <role>` with **Revoke** (confirmed); grant via the *grant role name* field → **Grant membership**.
- **PRIVILEGES** — current privileges per object; revoke is confirmation-gated. **GRANT PRIVILEGE** form: pick object kind, enter schema/object/privilege → **Grant privilege**.

### 3.3 Row-Level Security

Enter **schema** and **table**:

- **Inspect RLS** — shows `enabled`, `forced`, and existing policies (`name · command · PERMISSIVE/RESTRICTIVE · roles[…] · USING(…) · CHECK(…)`).
- Table toggles: **Enable RLS**, **Disable RLS**, **Force RLS**, **No Force** — all preview-then-confirm.
- Per policy: **Drop policy**, **Load for edit** (fills the form).
- **CREATE / ALTER POLICY** form: policy name, command (`ALL/SELECT/INSERT/UPDATE/DELETE`), roles (comma-separated, empty = `PUBLIC`), **Restrictive** checkbox, `USING` and `WITH CHECK` expressions → **Preview CREATE** or **Preview ALTER** shows the exact SQL → tick *"I understand this changes data visibility immediately"* → **Apply preview SQL**.

---

## 4. Saved Tasks & Backups

Open via **Tools hub → Saved Tasks & Backups**. The scheduler runs only while DB Pro is open — there is no background daemon. Secrets stay on the connection.

Empty state: *"No saved tasks — Create a SQL, export, backup, or maintenance task to run manually."*

### 4.1 Creating a task

- **New SQL task** — drafts a task from the current query text and active connection.
- **New backup task** — drafts a backup payload (output path, optional **Custom (pg_dump -Fc) format**).

Draft form fields: **Name**, **Description**, plus payload fields:

| Payload | Fields | Run behavior |
|---|---|---|
| SQL | SQL textarea | Runs the SQL on the task's connection |
| Backup | Output path, custom format | Runs the backup command |
| Export | Table (optional), Format (`csv`/`tsv`/`json`) | Runs `SELECT` on the table and writes the result to a temp export file (`db-pro-export-<id>.<fmt>`) |
| Maintenance | Operation (`vacuum`/`analyze`), Target (optional) | Runs the maintenance command (pre-confirmed) |

**Save task** / **Cancel**.

### 4.2 Running and scheduling

Each task card shows name, kind, description, connection, last run (`status · ms · message`), and schedule state (`on/off · next <time>`).

| Button | Effect |
|---|---|
| **Run** | Runs immediately. Destructive payloads require ticking **Confirm destructive task run** → **Run destructive task** first. |
| **Schedule 60s** / **Disable schedule** | Toggles a 60-second interval schedule (app must stay open). Scheduled destructive tasks are blocked by policy. |
| **Delete** | Asks *"Delete this saved task?"* → **Delete task** / **Keep task**. |

**RECENT RUNS** at the bottom lists the last 12 executions (`OK`/`FAIL`/`CANCEL`, duration, message).

---

## 5. Data Transfer & Migration

Open via **Tools hub → Data Transfer & Migration**. Tagline: *"Streaming transfer engine · bounded batches · cancellable."*

### 5.1 Backup / Restore card

Shortcut that jumps to **Settings → Backup / Restore** (provider-aware `pg_dump`/`pg_restore` or SQLite snapshot).

### 5.2 Synthetic Table Seed (dev/test only)

Deterministic seed data generator. Pick a table from the loaded schema, set **rows**, **seed**, **null%**:

- **Preview** — shows the first 8 generated rows.
- **Export SQL → Query** — writes the `INSERT` script into the Query editor.
- **Apply INSERT (run)** — exports and executes it. On a *Production* connection you must tick **"I confirm seeding this Production database"** first.

### 5.3 Masking / Safe Sample

Preview-only by default — no in-place destructive masking.

- **cols** field: comma-separated column names (e.g. `email,phone`), or click **Suggest cols** to auto-detect sensitive names.
- Rule selector: **Redact / Hash / Partial / Fixed / Synthetic**; **Keyed hash** keeps joins consistent.
- **Preview sample** shows original → masked pairs; **Masked CSV export** runs the export job.

### 5.4 Copy / harness

Fields: **Source** / **Target** connection ids, **Tables** (`schema.table…`), **Format** (`csv` or `json`), **Conflict** (`abort` / `skip` / `replace`).

Toolbar:

| Button | Effect |
|---|---|
| **Run synthetic harness** | End-to-end streaming transfer test job |
| **Run then cancel** | Same, cancelled midway (verifies cancellation) |
| **CSV export harness** / **CSV import preview** / **JSONL export harness** / **Excel export harness** | Format round-trip harness jobs |
| **Copy tables** | Runs the DB-to-DB copy with the fields above (needs source and target connections open) |
| **Clear jobs** | Clears the job list |

### 5.5 Jobs

Each job card shows label, status badge (`pending`/`running`/`succeeded`/`failed`/`cancelled`/`partial`), progress (`read / wrote / bytes / committed batches / uncommitted / error rows / message`), and any error. Empty state: *"No transfers yet."*

---

## 6. Agent (AI Copilot)

Toggle the right-hand **Agent** panel via the sparkles icon in the activity bar. The panel is resizable (drag its left edge).

### 6.1 Default mode — Offline draft

With no API key configured, the provider badge reads **Offline draft** / **Preview**: prompts produce unexecuted draft responses — nothing is sent anywhere and nothing runs.

### 6.2 Configuring a provider

Click the gear (**Agent settings (API key)**) in the panel header:

- Paste a key (`gsk_…` for Groq, `sk-…` for OpenAI — the provider is auto-detected from the prefix) → **Save key** or `Ctrl/Cmd+Enter`. **Forget key** removes it. Keys go to the secret store, never plain text on disk.
- The disclosure states exactly what leaves the machine: *your prompts, the SQL they reference, the schema names and types around them, and up to 20 sample result rows (50 columns, 256 chars per cell) when the agent runs a query. Your database and SSH connections are the app's only other outbound connections.*
- **Auto-run read-only queries in Agent mode** checkbox mirrors Settings → AI Providers.

### 6.3 Header & context

- Mode selector: **Ask** (questions), **Edit** (propose SQL changes), **Agent** (multi-step with tools). Disabled while a run is active or a confirmation is pending.
- Icons: **Clear conversation** (when idle), **Close Agent**, **Agent settings**.
- Context chips show the live context: provider, `Auto-run Read-only` badge, connection, driver, schema, selected table, `EXPLAIN PLAN` flag.
- Quick actions (appear when relevant): **Explain query**, **Optimize** (current SQL), **Explain table** (selected table), **Investigate error** (last DB error) — each submits a prepared prompt.

### 6.4 Thread

Empty state offers suggested prompts (*"Show me the schema overview"*, *"Count rows in the active table"*, *"Explain query performance"*). Messages stream in; activities show status (`Running`, `Needs approval`, `Done`, `Failed`, `Cancelled`) and duration. Query tool results show `N rows, M cols` with an **Open in Results** / **Open sample in Results** button. Failed sessions offer **Retry**.

### 6.5 Approvals

The agent pauses on gated actions:

| Title | Approve button |
|---|---|
| Apply Agent change? (shows a −/+ SQL diff preview) | **Apply Change** |
| Run read-only query? | **Run Query** |
| Run mutation? (warning: modifies data/schema) | **Run Mutation** |
| Execute destructive query? (may drop/truncate) | **Execute Destructive Query** (destructive styling) |
| Run unclassified query? | **Run Unclassified Query** |

**Reject** declines. Read-only queries auto-run only if you enabled auto-run.

### 6.6 Composer

Bottom input: type a request and submit (or **Stop** a running generation). Agent sessions are per query document — switching tabs keeps each document's conversation.

---

## 7. Files workspace

Open a folder via **SQL Scripts & Projects** rail → the workspace card → **Open Folder** (or the rail header's folder icon). The sidebar gains a **WORKSPACE** header and a card showing the active root, path, trust state, environments, and root chips.

- **Open Folder** / recent folders (up to 8) when empty.
- Workspace card: active root name + path, **Close workspace** (✕), **Remove active root** (multi-root), trust toggle **Trust** / **Revoke** (*"Trust this workspace to enable writes and tasks"*), **Environment** selector, multi-root **Roots:** chips, schema-drift and error notes.
- Header icons: add/open folder, **Refresh**.

### 7.1 Tabs

| Tab | Contents |
|---|---|
| **Files** | File tree (`FILES · n`), **New SQL file** / **New folder** icons. Click a `.sql` to open it in the editor; right-click a folder for **New SQL here** / **Delete folder**, a file for **Open**, **Add to Agent context**, **Find references**, **Delete**. |
| **Search** | **Find in files…** + **Replace with…**: **Find**, **Preview** (shows `file · N hits` preview rows), **Replace all** → *"Replace all matches in the workspace?"* → **Confirm replace** / **Keep files**. A **Rename from / to / Refactor** row performs rename refactoring. Search hits (`path:line`) open the `.sql` on click. |
| **Migrations** | SQL files detected under `migrations/` paths with version + status; click to open. |
| **Tasks** | Run a shell command in the workspace root — requires confirmation (*"Run this command with sh -lc?"* → **Confirm shell** / **Do not run**). Shows the last result (command, exit code, ms, stdout/stderr). **Benchmark** runs a local timing check. |
| **Graph** | `file → object` dependency edges extracted from `FROM`/`JOIN` references (up to 60). |
| **Git** | **Refresh** probes git status for the active root: branch, entries `[XY] path` with **Stage** / **Unstage** / **Diff** / **Open**, a commit-message field (explicit only — never auto) and **Commit staged**. Diff shown inline (truncated at 8000 chars). Warns when a file changed on disk (*"Disk changed for … — unsaved editor buffer was kept"* → **Reload from disk** / **Dismiss**). |

### 7.2 Agent context strip (bottom of the panel)

Attach context for the Agent: **Active File**, **Selection**, **Table** buttons, removable context chips, and a **⋯** menu: **Check schema drift**, **Export schema snapshot**, **Toggle split editor**, **Clear context**.

---

## 8. Connection lifecycle notes

- **Test connection** in the connection dialog becomes **Cancel test** while a test is in flight (with a "Testing…" caption); cancelling shows *"Connection test cancelled"* and keeps the dialog open.
- **Delete connection** is confirmed via a dialog (**Cancel** aborts).
- The status bar shows **Connecting…** while a connect is pending; runtime errors surface in the status message area and in **Settings → Diagnostics**.
