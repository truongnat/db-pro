# Agent evidence template — claim, progress, handoff, review

> Canonical contract for every coding, research, audit and review agent working in this repository.
> Location of this template is referenced from `AGENTS.md` ("Agent evidence contract") and from
> issue #113. One file per task, placed where the task's plan lives
> (`docs/plans/active/<feature>/` or the issue-ledger evidence path the task names). Fill the
> sections that apply to the task's lane; never delete a section header — write `n/a` instead.

**Non-negotiable rules**

- An **exact SHA is mandatory wherever source behaviour is asserted**. Line anchors without a SHA,
  or "the current tree", are not evidence.
- A skipped or failed gate must be reported as skipped/failed. "Passed" for a command that was not
  run is a contract violation.
- Review outcomes separate **inherited baseline failures** from **regressions introduced by the
  reviewed SHA**.
- Normalized task states: `Todo / Ready / In Progress / Review / Blocked / Done / Deferred`.
- Normalized PR states: `Draft / Ready / Merged`.

---

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | `<agent-id / lane>` |
| Issue(s) | `#…` |
| Task state | `In Progress` (claimed) |
| Baseline SHA | `<full 40-char sha>` — everything below is asserted against this tree |
| Branch / PR | `main` / `feature/<slug>` / `fix/<slug>` · PR `#…` (`Draft`/`Ready`/`Merged`) |
| Scope interpretation | one or two sentences on what this task covers |
| Out of scope | explicit list; `none` only when literally nothing is excluded |

## 2. Progress checkpoint

- Current HEAD: `<sha>`
- Completed acceptance rows: `[x]` list copied from the issue
- Remaining acceptance rows: `[ ]` list
- Findings / risks: severity-tagged (`P0/P1/P2`), each with file:line at the reviewed SHA
- Tests already run: exact command + `N passed / M failed / K ignored` + exit status
- Dependency / blocker changes: what was blocked at claim time and what changed

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `<full 40-char sha>` |
| Commit list | one line per commit: short sha + subject |
| File / surface inventory | every file changed + what changed in it |
| Acceptance mapping | issue row → code/test/document that satisfies it |
| Commands and counts | exact commands with raw pass/fail/ignored numbers and exit status |
| CI run IDs / status | workflow run id + status, or `not run` |
| Known limitations | honest list; `none` only if none |
| Migrations / config implications | persisted-state, env vars, keys, paths affected |
| Out-of-scope changes | `none` or an explicit list of what was deliberately not touched |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | `<full 40-char sha>` (a reviewer never changes the branch in the same pass) |
| Verdict | `ACCEPT` / `BLOCK` / `ACCEPT WITH P2` |
| P0 / P1 / P2 counts | introduced-by-this-SHA vs inherited |
| Findings | each with file:line or issue link, and the verdict consequence |
| CI disposition | run id + status at the reviewed SHA, or which gates were not executed |
| Next task(s) unblocked | list or `none` |

## 5. Research / audit handoff

- Source date: when the sources were read / fetched
- Source URLs / references: complete list (repo paths count as references)
- Factual findings: observed statements, with the pointer that proves each
- Inference: clearly separated from findings — nothing inferred is reported as fact
- Decision / recommendation: options considered + the recommended one, or `PENDING owner decision`
  with the decision owner named
- Unresolved questions: list
- Downstream tasks activated: issue numbers

## 6. Tổng kết (Vietnamese summary)

Per issue #113 every implementation/review handoff ends with a short Vietnamese summary
(`Tổng kết bằng tiếng Việt`) of what was done, what remains, and what the next agent must know.

---

## Example — coding task (filled)

```markdown
# Agent evidence — #56 PostgreSQL temporal decoders

## 1. Claim
| Agent identity | zcode · provider follow-up lane |
| Issue(s) | #56 |
| Task state | In Progress |
| Baseline SHA | 5749d8e… |
| Branch / PR | main (no PR) |
| Scope interpretation | explicit decoder branches for temporal classes; no timezone invention |
| Out of scope | dedicated timestamp/timetz/timestamptz domain variants (#52/#54) |

## 2. Progress checkpoint
- Current HEAD: f7c61c9d…
- Completed acceptance rows: [x] explicit decode branch per temporal OID, [x] TIMESTAMP keeps local semantics, [x] TIMESTAMPTZ keeps instant + canonical Z, [x] deterministic fixtures
- Remaining acceptance rows: none for this issue
- Findings / risks: none open
- Tests already run: cargo test -p db-pro-infrastructure --test pg_integration -- --ignored → 20 passed / 0 failed / 0 ignored, exit 0
- Dependency / blocker changes: none

## 3. Implementation handoff
| Exact SHA | f7c61c9dd… |
| Commit list | f7c61c9d fix(postgres): decode temporal value classes explicitly (Gate 5 B2) (#56) |
| File / surface inventory | crates/infrastructure/src/postgres/query_mapper.rs — two formatters + decoder arms; crates/infrastructure/tests/pg_integration.rs — two provider tests |
| Acceptance mapping | "TIMESTAMP cannot carry invented Z/offset" → pg_timestamp_without_time_zone_keeps_wall_clock_value |
| Commands and counts | cargo test --workspace → 841 passed / 0 failed / 21 ignored, exit 0 |
| CI run IDs / status | not run (no CI trigger on main push in this session) |
| Known limitations | none for the fixed path |
| Migrations / config implications | none |
| Out-of-scope changes | none |

## 4. Review outcome — n/a (self-verified + owner-closed; recorded in LEDGER.md)

## 6. Tổng kết
Đã sửa #56: bộ giải mã timestamp không còn gắn timezone; 2 test provider + 4 unit test; toàn bộ gate xanh (841/0/21). Chưa đụng đến các variant DTO chuyên biệt (#52/#54).
```

## Example — research/audit task (filled)

```markdown
# Agent evidence — #121 persisted identifier inventory

## 1. Claim
| Agent identity | zcode · audit lane |
| Issue(s) | #121 |
| Task state | In Progress |
| Baseline SHA | 6c698bc… |
| Branch / PR | main (docs only) |
| Scope interpretation | classify every persisted identifier that a product rename could break |
| Out of scope | the identity decision itself (#101/#30); display-string rename (#102 §2 is the sibling) |

## 2. Progress checkpoint
- Completed acceptance rows: [x] inventory, [x] classification, [x] migration rules, [x] keyring + app-data paths covered
- Remaining acceptance rows: none
- Findings / risks: P1-risk rows: keyring service, eframe app id — both classified PERSISTED COMPATIBILITY
- Tests already run: cargo check --workspace, exit 0 (docs-only change)
- Dependency / blocker changes: none

## 5. Research / audit handoff
- Source date: 2026-09-15
- Source URLs / references: crates/runtime/src/lib.rs:62, crates/native-app/src/main.rs:128-200, crates/ui/src/app.rs:445-471, docs/release/brand-rename-inventory.md
- Factual findings: every row of the inventory table is a grep/read at the baseline SHA
- Inference: the migration grace period (one release) is a recommendation, not a repo fact
- Decision / recommendation: PENDING owner decision on identity; migration strategy recommended for the MIGRATE rows
- Unresolved questions: whether the legacy .db-pro-data lookup should be removed after v0.1
- Downstream tasks activated: #103 (implementation checklist)

## 6. Tổng kết
Đã kiểm kê toàn bộ định danh lưu trữ theo SHA cơ sở; phân loại thành PERSISTED COMPATIBILITY / MIGRATE WITH FALLBACK / INTERNAL TECH ID; bàn giao checklist cho #103. Quyết định tên thương hiệu để lại cho chủ sở hữu.
```
