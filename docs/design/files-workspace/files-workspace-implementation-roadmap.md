# Files Workspace — implementation roadmap

## 1. Source baseline and inputs

- **Exact source baseline SHA:** `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- **Input documents:** [files-workspace-baseline.md](files-workspace-baseline.md), [files-workspace-feature-research.md](files-workspace-feature-research.md).
- Source facts below describe this SHA, not an exercised UI. Product proposals and priorities are recommendations from research; no provider/database runtime claim is implied. Research assigns P1 to trust/mutation and reviewable replace safety; other feature priorities retain their stated conditions (`files-workspace-feature-research.md:49-70,81-143,145-215`).

## 2. Decision

Treat Files as a local SQL-project workspace, not a database migration executor or general-purpose IDE. First make local filesystem mutations reviewable and errors visible while preserving trust/sandbox boundaries; then build reviewable search/replace. Decide whether multi-root workspaces are session-local or persisted before adding persistence assumptions. Keep Migrate, Graph and Refactor labels honest about their narrow file-name/keyword/literal behavior. Workspace trust is not filesystem authorization: browsing/opening and writing/deleting/executing/sending data must have explicit, distinct policies. Never allow adding an untrusted root to silently elevate another root. (Baseline: `files-workspace-baseline.md:47-63,71-87,91-124`; research: `files-workspace-feature-research.md:45-70,94-103,123-143,236-265`.)

## 3. Current state

### Implemented at source baseline (source fact, not runtime proof)

- Workspace shell supports recent folders, multiple roots, a root switcher, trusted/untrusted indicator, environment labels and six sub-tabs (`files-workspace-baseline.md:49-63`).
- File tree can open SQL, create file/folder, delete, attach Agent context and search references; scanner has explicit extension, skip, depth and entry limits (`files-workspace-baseline.md:65-69`).
- Search is literal/case-insensitive with bounded hits; replace preview counts hits by file; replace-all and text “Refactor” require Trusted (`files-workspace-baseline.md:78-87`).
- Migrate lists filename/path-detected SQL files; Tasks runs a free-form command via shell in trusted root and stores bounded last output; Graph scans SQL keywords; optional Git adapter supports local status/stage/diff/commit; Agent context and schema helper actions exist (`files-workspace-baseline.md:89-111`).

### Not implemented / incomplete (source fact or absent evidence)

- No workspace manifest is visible in `WorkspaceFilesState`; stable root identity, persisted trust/selection/expansion and ignore configuration are not established (`files-workspace-baseline.md:56-63`; research `files-workspace-feature-research.md:123-143`).
- Search lacks scoped include/exclude, regex, whole-word and match-case controls; preview lacks exact before/after diff and per-match selection, and apply does not freeze/revalidate file versions or dirty editor buffers (`files-workspace-feature-research.md:81-103`; baseline `files-workspace-baseline.md:80-87`).
- Migrate has no database history lookup or execution; Tasks has no catalog, streaming, cancel or history; Graph is not resolved SQL dependency analysis; Git view lacks history/branch/remotes/hunk staging/conflict resolution (`files-workspace-baseline.md:91-105`).
- Environment labels are not connected database profiles; schema snapshot output is table-name/column-count comments, not executable DDL (`files-workspace-baseline.md:58-59,107-111`).
- No UI runtime/destructive operation/task/live Git or database migration scenario was run (`files-workspace-baseline.md:126-128`; research `files-workspace-feature-research.md:274-276`).

### Needs fix / safety dispositions (retain research priorities)

- **P1 — Tree mutation confirmation and truthful errors.** Delete recursively invokes local deletion without confirmation in the view; create-under-directory/delete result errors are discarded (`files-workspace-baseline.md:71-76`; research `files-workspace-feature-research.md:49-79`). Require exact-target confirmation for destructive delete and surface create/delete failures with paths. Distinguish file and directory.
- **P1 — Safe, reviewable bulk replacement.** Sequential Replace All writes have count-only preview and no version/buffer revalidation; partial failure must not be reported as full success (`files-workspace-baseline.md:78-87`; research `files-workspace-feature-research.md:81-121,236-255`). Do not claim atomicity without an actual staging/transaction protocol.
- **P1 — Explicit capability/trust policy before broadening.** Existing trust gates are inconsistent by operation; trust toggle is boolean although model includes Restricted; operation classes and root-level trust behavior need a defined contract (`files-workspace-baseline.md:58,71-76`; research `files-workspace-feature-research.md:49-70`). Trust must not silently become permission to arbitrary filesystem writes or process execution.
- **P2 conditional — multi-root identity/persistence.** If Files is a durable project workspace, stable IDs and manifest/trust rules are needed; if session-local, do not imply persistence (`files-workspace-feature-research.md:123-143`).
- **P2 — Do not mislabel heuristics as semantics.** Literal replace is not SQL-aware rename; path detection is not migration status; keyword scan is not resolved graph. Preserve those limits until their respective contracts are implemented (`files-workspace-baseline.md:87,93,99-102`; research `files-workspace-feature-research.md:202-215,257-265`).

## 4. Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 | fix | Unconfirmed recursive delete and discarded results: `files-workspace-baseline.md:71-76`; research `files-workspace-feature-research.md:49-79` | Route delete through request/confirmation; show exact root/path and file-vs-folder; surface success/failure path for delete and all create operations. Keep trust separate from filesystem permission and preserve path containment within selected root. | Define destructive-operation confirmation and root containment contract. | Delete cannot occur before explicit confirmation showing the exact target; cancel leaves it intact. Missing/permission-denied create/delete reports the path and failure; UI never displays success after an error. A selected operation cannot escape its root. |
| P1 | fix | Trust model and operation inconsistencies: `files-workspace-baseline.md:58,71-76`; research `files-workspace-feature-research.md:49-70` | Specify operation policy for browse/open, write/replace/delete, task execution and Agent egress; make Restricted/Trusted/untrusted presentation and multi-root trust behavior explicit. | Product/security decision; should precede bulk replace/task expansion. | Each operation class has a documented allow/deny/confirmation behavior; untrusted root cannot run tasks or bulk writes; adding/trusting one root does not change another root's trust without explicit action; blocked action explains why. |
| P1 | missing | Literal search and count-only preview; no revalidation: `files-workspace-baseline.md:78-87`; research `files-workspace-feature-research.md:81-121` | Build Reviewable SQL Workspace Search/Replace: scoped search, grouped hits, exact before/after preview, selectable changes, dirty-buffer/disk-version conflict detection, per-path written/skipped/failed outcome. Label text replacement honestly; keep semantic rename separate. | P1 operation policy; root/path scoping and file identity. | Two roots with same filename remain distinct; exclude patterns prevent reads/writes outside scope; preview matches exact write; changed-on-disk or dirty-buffer targets are not silently overwritten; partial apply reports each path and does not claim total success. |
| Proposed P1 | verification | Research explicitly did not run destructive, task, Git or UI scenarios: `files-workspace-feature-research.md:274-276`; baseline evidence boundary `files-workspace-baseline.md:126-128` | Exercise native UI for trust transitions, confirmed delete/cancel, permission errors, cross-root replace conflicts and task gating; retain artifacts. | Safety and replace implementations; temp workspaces with controlled permissions. | Evidence demonstrates exact target/confirmation, cancel non-effect, root containment, dirty/conflict rejection, partial-failure reporting and untrusted task denial. Source inspection alone is not completion. |
| Proposed P2 (conditional) | missing | No manifest; root IDs follow add position; source does not show persisted workspace state: `files-workspace-baseline.md:56-63`; research `files-workspace-feature-research.md:123-143` | Product owner decides session-local vs durable. If durable, implement manifest with stable root ID, path, display name, explicit trust policy and selected/expanded state; disambiguate same-name files across roots. If session-local, explicitly present/reset state accordingly instead. | Product decision; operation policy for persisted trust. | Restart behavior matches declared model; root order/add/remove does not change surviving root identity; same-name paths are uniquely identifiable; no persisted trust escalation occurs. |
| P2 (conditional) | missing | Migrate is path-derived `Unknown` and has no DB lookup/execute: `files-workspace-baseline.md:91-94`; research `files-workspace-feature-research.md:145-162` | Only if Files is approved as migration review surface, show source root, explicit connection/schema target, version/order/checksum and database-backed status using existing Schema Compare/Migration Planner and safety confirmation. Do not add a second executor. | Target DB/history contract; shared migration planner; provider capability matrix and confirmation review. | Status is based on database history, not filename alone; opening/refreshing never executes; preview/impact and explicit confirmation precede apply; unsupported provider behavior is explicit. |
| P2 | missing | Free-form `sh -lc` task, last bounded output only: `files-workspace-baseline.md:95-98`; research `files-workspace-feature-research.md:164-181` | After trust policy, add named project tasks with visible command/cwd/environment, explicit confirmation, bounded streaming/status/duration, cancel/timeout and limited history; never auto-run on open/refresh. | Trust/risk classification and process lifecycle/error model. | No task starts on workspace open; run shows exact command context and requires policy approval; exit status and bounded output visible; cancellation/timeout truthful; untrusted workspace cannot execute. |
| P2 | upgrade | Local Git supports status/stage/diff/commit only: `files-workspace-baseline.md:103-105`; research `files-workspace-feature-research.md:183-200` | Improve local change review first: distinguish staged vs working-tree diff, line navigation and SQL review before commit. Do not add remotes/history/hunk-stage until adapter contract/use case is approved. | Existing local Git adapter; error states surfaced. | User can identify staged vs unstaged content and review exact changed SQL before commit; errors are actionable; no unimplemented remote capability is implied. |
| P2 | upgrade | Keyword graph and string-pattern diagnostics are heuristic: `files-workspace-baseline.md:99-102`; research `files-workspace-feature-research.md:202-215` | When source ranges/context are available, parse/tokenize SQL, ignore comments/strings, and distinguish resolved/unresolved/ambiguous references; retain heuristic label until then. | SQL parser/tokenizer source ranges and validated connection/schema context. | Comments/string literals do not create edges; resolved references identify their source and context; unresolved/ambiguous items are not presented as resolved; PostgreSQL semantics are not inferred from SQLite. |

## 5. Rollout and dependency order

1. Define path/root containment and operation-specific trust policy; fix confirmation and surfaced errors for create/delete first.
2. Implement reviewable search/replace only on top of those guarantees, including immutable/revalidated preview targets and truthful partial-result behavior. Do not call it semantic refactor.
3. Resolve session-local versus persistent workspace intent; implement stable identity/manifest only if persistence is approved, with trust migration designed explicitly.
4. Independently approve database-project extensions: migration review must reuse the shared planner; Tasks must use the trust/process contract; Git improvements remain local review first.
5. Treat Graph/diagnostics as heuristics until SQL semantics/context are validated. Never let heuristic output block safe work or claim resolved dependencies prematurely.

## 6. Provider/support matrix

Not database-facing for the current local Files operations. Search/replace, filesystem trust, tasks and local Git do not imply PostgreSQL or SQLite support. The proposed migration status/review feature would be database-facing only if approved; no provider behavior is established by these inputs.

| Proposed migration surface (conditional) | PostgreSQL | SQLite |
|---|---|---|
| Database history/status, preview and apply via shared planner | **Not verified.** Must establish provider-specific history/preview/apply capabilities independently before claiming support (`files-workspace-feature-research.md:145-162,274-276`). | **Not verified.** Must establish independently; do not infer from PostgreSQL (`files-workspace-feature-research.md:145-162,274-276`). |

## 7. Verification gates still needed (not run)

- **Not run:** native Files UI traversal for trust, roots, destructive confirmation, errors, search conflict/partial-write and tasks.
- **Not run:** local Git live repository matrix, including external file changes and commit review.
- **Not run:** PostgreSQL or SQLite migration status/review/apply scenarios; no DB runtime behavior is established.
- **Not run:** process cancellation/timeout/streaming behavior for Tasks.
- **No tests/build/lint/formatter were run for this roadmap.** The baseline and research explicitly report source inspection rather than UI/destructive/task/Git/database runtime verification (`files-workspace-baseline.md:126-128`; `files-workspace-feature-research.md:274-276`).

## 8. Out of scope / unresolved decisions

- No source, baseline/research docs, plans or evidence files changed. No implementation authorization is implied.
- Unresolved: whether workspaces persist; whether trust is per root or workspace-wide; exact filesystem path-sandbox semantics; whether migration status belongs in Files; project-task configuration format; whether later Git remote features have a supported adapter/use case (`files-workspace-feature-research.md:123-162,164-200`).
- No arbitrary workspace scripts on open/refresh, unreviewed bulk mutation, semantic-rename claim for literal replace, separate migration executor, false database environment profiles, or premature Git remotes/merge workflows (`files-workspace-feature-research.md:257-265`).
