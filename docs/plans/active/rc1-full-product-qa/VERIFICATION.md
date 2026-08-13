# RC1 Full Product QA — Verification Matrix

Audit baseline: `6e0a04ad675eaa85cae08bbe1a066270596a18db`

Source review identifies defects; it does not close runtime evidence. Each remediation PR must record tests against its own head SHA, then integrated verification must run again on `main`.

## Mandatory automated commands

### Frontend

```bash
cd frontend
npm run typecheck
npm run lint
npm run format:check
npm run check:tokens
npm run test
npm run build
```

### Rust

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## P1 Regression Matrix

| ID | Automated proof required | Runtime proof required |
|---|---|---|
| QA-P1-01 | exact i64 serialization tests + frontend exact representation + update/delete identity | PG + SQLite row with PK `9007199254740993` displays and mutates exact row |
| QA-P1-02 | workspace/store integration test: staged A → preview B cannot inherit revisions | single-click A/edit/single-click B; no cross-table change bar/apply |
| QA-P1-03 | close guard tests for single/many/preview/reassign | stage edit then close/close-all; explicit discard prompt |
| QA-P1-04 | SQLite table-data column metadata tests + codec tests | SQLite INTEGER/BLOB/text table editor shows correct capabilities |
| QA-P1-05 | ConnectionEditor provider form test | new SQLite connection saves/tests without password |
| QA-P1-06 | SSH toggle model test | edit SSH conn → disable → save/reopen; SSH remains off |
| QA-P1-07 | SSH default-state serialization test | enable SSH, never touch port, use host/user/key → submitted port is exactly 22 |
| QA-P1-08 | fake delayed mutation test with dialog generation/session | close in-flight create, reopen New, later submit creates new record only |
| QA-P1-09 | restore coordinator call-count tests | startup restores once; normal connection-list refresh never reconnects |
| QA-P1-10 | orphan close guard test | orphan dirty query Close requires confirmation |
| QA-P1-11 | provider/resource reassignment tests | PG public.users → SQLite cannot silently retain invalid public.users context |
| QA-P1-12 | 500/1000 fixture initial visible-node/layout count | office 500+ schema opens into bounded/search-first state |
| QA-P1-13 | first-render node tier test | no full-column first-paint freeze on large schema |
| QA-P1-14 | PostgreSQL provider type integration matrix | representative DB columns query/render without precision loss/crash |

## Precision Test Fixtures

### PostgreSQL

```sql
CREATE TABLE qa_bigint_identity (
  id BIGINT PRIMARY KEY,
  amount NUMERIC(38, 18),
  label TEXT NOT NULL
);

INSERT INTO qa_bigint_identity (id, amount, label) VALUES
  (9007199254740991, 12345678901234567890.123456789012345678, 'safe-boundary-minus'),
  (9007199254740992, 0.000000000000000001, 'safe-boundary'),
  (9007199254740993, 99999999999999999999.999999999999999999, 'unsafe-js-number'),
  (9223372036854775807, 1.000000000000000001, 'i64-max');
```

Required assertions:

- IDs display exactly as database text.
- Copy/export returns exact decimal string.
- Selecting `9007199254740993` never aliases to `9007199254740992`.
- Update label by PK affects exactly one intended row.
- Delete by PK removes exactly intended row.
- NUMERIC text remains exact; no scientific notation/float rounding unless user explicitly requests formatting.

### SQLite

```sql
CREATE TABLE qa_bigint_identity (
  id INTEGER PRIMARY KEY,
  value_text TEXT
);

INSERT INTO qa_bigint_identity VALUES
  (9007199254740991, 'a'),
  (9007199254740992, 'b'),
  (9007199254740993, 'c'),
  (9223372036854775807, 'd');
```

Same identity assertions as PostgreSQL.

## Staged Mutation Safety Scenarios

### Preview replacement

1. Open table `qa_a` by single click.
2. Edit row PK=1 but do not Apply.
3. Confirm Change Bar shows one pending update.
4. Single-click `qa_b`.
5. Expected: either preview is promoted/protected and A remains visible, or user receives explicit discard decision.
6. Forbidden: B opens using same pending change state.
7. Apply must never issue A's revision against B.

Repeat via:

- Explorer single clicks
- SearchView
- Quick Open keyboard preview navigation

### Close lifecycle

Run each with a staged edit and a staged delete:

- close button
- Ctrl/Cmd+W
- tab context menu Close
- Close Others
- Close to Right
- Close All
- preview replacement
- orphan Close
- change/reassign connection
- restart/session restore

Expected invariant: pending mutation is either visibly preserved with its exact resource identity or explicitly discarded by user. It is never silently hidden/rebound/applied.

## Connection Lifecycle Scenarios

### SQLite form

- New SQLite shows file/database path workflow only; no password requirement.
- Browse cancel leaves current path unchanged.
- Browse failure returns a visible error.

### SSH model

- Existing enabled → uncheck → Save → reload → disabled.
- New enabled → host/user/key entered, port untouched → model port 22.
- Toggle enabled/disabled multiple times does not resurrect old tunnel unexpectedly.

### Stale dialog callback

Use delayed fake backend:

1. Open New Connection A.
2. Submit create; promise remains pending.
3. Close dialog.
4. Resolve A create.
5. Open New Connection B.
6. Submit B.
7. Expected: create B.
8. Forbidden: update A.

### Session restore

Instrument `service.connect` call counts:

- initial hydrate with N persisted active IDs → exactly N intended restore attempts
- connection-list invalidation after normal connect → zero restore calls
- rename/favorite/update/list refresh → zero restore calls
- manually disconnected connection must not be restored until intended next session policy says so

## ER Large-Schema Test Matrix

Fixture sizes: 20 / 100 / 500 / 1000 tables.

### Automated structural budgets

For 500 and 1000 tables assert:

- initial visible nodes are bounded below full table count unless explicit Show All
- initial node data is not full-detail tier
- initial Dagre input is bounded/search-neighborhood, not entire schema
- search/neighborhood BFS is deterministic
- no relayout on edge highlight
- MiniMap disabled when intended

### Office runtime

Record on a real >500 table schema:

| Check | Expected |
|---|---|
| Open ER | interaction available without long freeze |
| Initial state | search-first/bounded, not 500 detailed cards |
| Search | responsive; explicit result selection |
| Neighborhood | selected table + correct 1/2-hop relations |
| LOD low | table-level summary only |
| LOD high | detail appears only where intended |
| Edge anchors | remain valid through zoom tier changes |
| Pan/zoom | no repeated multi-hundred-ms freezes |
| Show All | explicit warning/intent and remains cancellable/usable |
| Memory | repeated open/close does not grow without bound |

Capture DevTools performance trace for at least initial open and pan/zoom.

## PostgreSQL Type Matrix

Create/query representative columns:

```sql
CREATE TYPE qa_status AS ENUM ('new', 'done');

CREATE TABLE qa_type_matrix (
  id BIGINT PRIMARY KEY,
  n NUMERIC(38,18),
  d DATE,
  t TIME,
  tz TIMETZ,
  ts TIMESTAMP,
  tstz TIMESTAMPTZ,
  i INTERVAL,
  ip INET,
  u UUID,
  j JSONB,
  b BYTEA,
  e qa_status,
  ia INT[]
);
```

For each type record one of:

- `FULL`: lossless display + supported editing/mutation
- `READ_ONLY`: lossless display, editing deliberately disabled with reason
- `UNSUPPORTED`: explicit error/limitation without unrelated row/query crash where feasible

Never silently coerce NUMERIC/BIGINT through float/number.

## P2 Interaction Verification

### Tabs
- Pin C in `[A,B,C]`; visual order, Ctrl+Tab order, drag order and Close to Right must agree.
- macOS labels use `⌘`; Windows/Linux labels use `Ctrl`.

### Column picker
- Clicking row toggles exactly once.
- Clicking checkbox toggles exactly once.
- Keyboard Space toggles exactly once.

### Read-only grid
- Connection marked readonly presents grid as non-editable before user begins editing.
- Backend still independently blocks mutation.

### Context menu
- Right-click bottom/right viewport edge: menu remains visible.
- Escape closes.
- outside click closes.
- keyboard focus moves through actions.

### Query export
- SQL typed, never executed → export disabled.
- result exists → export enabled.
- clear editor while result remains → product policy explicit; if result remains exportable, export stays enabled.
- failed/cancelled result does not masquerade as exportable success.

### Connection test result
- Test success → edit host/password/db → old success disappears/becomes stale state.
- backend detailed error is surfaced safely.

## Visual / UX Smoke

Run both light and dark:

- shell surfaces/borders/focus rings
- shadcn dropdown/popover/dialog/tooltip/checkbox/select
- Connection Dialog
- Explorer/Search
- Query editor and result dock
- Data Grid + Change Bar
- Schema/ER
- Agent Preview panel

Platforms:

- macOS: traffic-light inset present and aligned
- Windows/Linux: no mac-only blank inset
- shortcut labels match platform

## Evidence Template

```text
SHA:
Platform:
Provider:
Database/version:
Finding IDs verified:
Automated gates:
Runtime scenarios:
Passed:
Failed:
New findings:
Screenshots/trace:
Reviewer:
Date:
```

## Closure Rule

A finding is CLOSED only when:

1. fix exists on current PR head;
2. focused regression proves the exact failure scenario;
3. full applicable quality gates pass;
4. Kilo/Cubic findings against latest head are classified;
5. runtime-required items have runtime evidence;
6. integrated `main` is re-verified after merge.
