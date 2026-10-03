# Native UI Stitch Parity Pass — Verification

Only commands actually executed are recorded, with their real output summary.

## Baseline

| Command | Commit | Result |
|---|---|---|
| `cargo test -p db-pro-ui` | `52d3600c` | 972 passed / 0 failed |

## Per-task evidence

| Task | Commit | Gates run | Runtime evidence |
|---|---|---|---|
| T1 status telemetry | `7ede34a5` | fmt · clippy `-D warnings` · ui tests | `evidence/t1-statusbar.png` — `Ping: 1 ms · UTF-8` visible |
| T2 sidebar count + snippet codes | `73340682` | fmt · clippy · ui tests | source-verified; capture folded into query-surface shots |
| T3 execution history | `153756e2` | fmt · clippy · **974 ui tests** | `evidence/t3-history-dark.png` — Today group, outcome icons, detail card with Open-as-new-query/Copy |
| T4 explorer filter workbench | `bb137c33` | fmt · clippy · ui tests (mode-aware + nav-cache) | `evidence/t4-filter.png` — accent dot on filter button, Views folder hidden when disabled |
| T5 compare summary + target lock | `34601996` | fmt · clippy · **976 ui tests** incl. 2 new lock tests · clean-code 0 fail | **visual capture pending** — windowing stopped emitting screenshots mid-session (see FINDINGS) |

Query-editor polish landed earlier in the same session and is covered by this
plan's parity intent: gutter/active-line (`90759bea`), selected-lines telemetry
(`1b1c2c00`), row-limit selector (`52d3600c`) — captures in
`evidence/query-dark-v3.png`, `query-light-v2.png`, `query-limit-dark.png`,
`query-limit-light.png`.

## Session-end gates (final run @ `34601996`+docs)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean |
| `cargo test -p db-pro-ui` | **976 passed / 0 failed** |
| `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` | 11 pass / 5 warn / 0 fail |
| `bash .skills/perf-audit/scripts/perf-scan.sh` | PASS (partial) — 4 executed checks passed; 4 not executed (need live DB / window server). Release binary 32.2 MB |
| `cargo build --release --locked -p db-pro-native` | succeeded |

## Known gaps

- T5 safety-lock visual capture pending (environmental outage; unit tests cover
  the lock semantics). `DB_PRO_CAPTURE_COMPARE` exists for the next window.
- T6 ER-diagram spec-10 diff not performed — T0 audit found no visible delta and
  the capture outage removed verification ability.
- Evidence PNGs are dark-theme only where noted; light-theme captures exist for
  table + query editor, not for history/filter/compare surfaces.
