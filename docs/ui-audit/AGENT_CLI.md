# UI Audit CLI — contract for coding agents

Headless UI audit + optional AI visual review for `db-pro-native`
(debug build, `--features capture`).

## Run

```bash
DB_PRO_AUDIT_JSON=/tmp/out.json \
DB_PRO_AUDIT_SCENARIO=shell-1440 \
DB_PRO_WINDOW_SIZE=1440x900 \
./target/debug/db-pro-native
```

The process renders the app off the loop, waits for a stable layout
fingerprint + warm accesskit tree, writes the report, and exits.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | clean — no FAIL rule verdicts, no Error-severity issues under policy |
| 1 | audit findings under the active policy |
| 2 | tool error — report unwritable, baseline unreadable, timeout |

## CI fail policy — `DB_PRO_AUDIT_FAIL_ON`

Comma-separated tokens; default `error`.

| Token | Fails on |
|---|---|
| `error` | FAIL rule verdicts, Error issues (always on) |
| `warning` | + Warning-severity issues |
| `screen` | + *objective* screen findings (off-screen rect, layout inversion) |

Screen **heuristic** findings and all `visual_review` (AI) output are
advisory — they can never change the exit code. The active policy is
recorded at `meta.fail_policy`.

## Report shape (schema v4)

```text
schema_version, meta{viewport,theme,scenario,fixture,frames,fail_policy,...},
signature{...widget_ids}, summary{widgets,issues,errors,warnings,verdicts},
rules[{id,verdict,coverage}], issues[{key,rule,severity,confidence,evidence}],
screen{metrics{regions,rhythm,density,balance,hierarchy},coverage,findings},
visual_review?{status,provider,model,verdict,observations[]}
```

Three verdict channels, in order of authority: `summary`/`issues` (rules,
gate the exit code) → `screen.findings` (measured, objective/heuristic
kinded) → `visual_review` (AI advisory). `null` metric = UNKNOWN, not zero.

## Scenarios — `tools/ui-audit-matrix.sh`

`bash tools/ui-audit-matrix.sh <binary> <outdir>` runs 25 named scenarios
(shell 800/1440/1920, welcome, query, tables, diagram, results-dock, agent,
gallery, dialog, palette, + good/ugly fixtures) and writes
`coverage-matrix.json`. Readiness: `DB_PRO_AUDIT_READY=<min-widgets>`.

## AI visual review — opt-in

```bash
DB_PRO_REVIEW_DIR=/tmp/review \
DB_PRO_AI_REVIEW_CMD="python3 my-provider.py" \
DB_PRO_AI_REVIEW_MODEL=gpt-4o \
DB_PRO_AI_REVIEW_TIMEOUT=60 \
DB_PRO_AUDIT_JSON=/tmp/out.json ... db-pro-native
```

- Writes `<scenario>.bundle.json` + `<scenario>.png` into the review dir.
- Provider contract: argv[1] = bundle path; stdout = review JSON
  (`provider`, `model?`, `verdict`, `observations[{category, kind:
  observation|suggestion, severity, confidence, rect?, evidence,
  recommendation?}]`). `DB_PRO_REVIEW_MODEL` is forwarded to the provider.
- Provider statuses: `completed | error | bundle_only | blocked`.
- **Privacy gate**: a screenshot only reaches a provider when the app proves
  fixture provenance (a `DB_PRO_CAPTURE_*` opener ran or a dev-tools fixture
  painted) or no live DB connection exists. A live session without
  provenance → `blocked`; the whitelisted bundle text is still written.
- Review is unreachable from interactive mode — it lives on the headless
  audit path only.
