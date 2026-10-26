# Clean-Code Baseline & Repo-Wide Ratchet — Plan

State: IMPLEMENTING (release-gap item, Nhóm 2)

## Goal

Make `python3 tools/cc-scan.py . --fail-on error` a gate that is both **green at
HEAD** and **blocking for new violations**, and record, per finding, whether the
23 repo-wide errors are accepted legacy debt or named refactor work.

The gap report (`docs/release/2026-10-25-v0.1.0-audit-and-release-gap.md` § 3,
Nhóm 2, "Đồng bộ Clean-Code Baseline") states the precondition explicitly:
decide whether a repo-wide gate exists **before** freezing the baseline, and
freeze only verified-inherited violations — never to hide a new one.

## Evidence / baseline

At `d1d7dc4f` (the commit that recorded the gap report):

| Fact | Value |
|---|---|
| `.clean-code-baseline.json` | `{"version":"1.0.1","counts":{}}` — empty, so every finding counted as new |
| Repo-wide scan | `FAIL (exit 1)`, 23 errors + 129 warnings over 1083 files |
| CI clean-code step | `.github/workflows/ci.yml:123` → `clean-code-scan.sh rust --diff --ratchet --ci` |
| cc-scan in any workflow | none — the tool was a local-only command |

At HEAD before this change: 23 errors + 126 warnings (the three findings this
branch introduced in `crates/core/src/domain/audit.rs` were removed by `72feea0d`).

## Inheritance proof (the freeze precondition)

`git diff --name-only d1d7dc4f..HEAD` intersected with the set of files carrying
findings yields exactly two files:

- `crates/app/src/app/app_lifecycle.rs` — findings at 173 (`prepare_frame` ×2) and
  326 (`draw_overlays`). The branch's hunks are `@@ -76,3 +76 @@` and
  `@@ -89,0 +88,19 @@`; both sit above and outside `prepare_frame`.
- `crates/native-app/src/main.rs` — finding at 353 (`initialize_sample_sqlite`).
  The branch's only hunk is `@@ -605,0 +606,6 @@` inside `run_native_app`.

The remaining 87 finding-bearing files are untouched by the branch. Direct scan of
the branch's Rust files reports nothing in `audit.rs`, `connection.rs`,
`tunnel.rs`, `settings_system_view.rs` or the new test file. **No open finding is
authored by the release-gap work**, so all 23 errors qualify as verified legacy.

## Scope

- Freeze the 23 verified-inherited **error-severity** findings into
  `.clean-code-baseline.json`.
- Add the repo-wide ratchet step to `ci.yml`.
- Record the per-function triage and the refactor backlog in `FINDINGS.md`.
- Update the gap report checklist item.

## Non-goals

- Refactoring any of the 23 functions in this change. Splitting a 105-line SQL
  builder or a security-relevant token scanner is product-risk work and belongs in
  its own PR with its own tests, not inside a release-documentation close-out.
- Freezing the 126 warnings. They stay reported so the debt remains visible; the
  gate is `--fail-on error`.
- Adding `// cc-scan:allow` comments at the 23 sites. The baseline is the mechanism
  for repo-wide accepted debt; inline exemptions are for a specific, argued
  exception at one site.

## Acceptance criteria

1. `python3 tools/cc-scan.py . --fail-on error` exits 0 at HEAD and prints
   `(ignored 23 findings frozen in the baseline)`.
2. A synthetic function over the hard size limit in a file **not** named by the
   baseline produces `error=1` and a non-zero exit — the ratchet is live, not
   decorative.
3. `--fail-on warning` still fails, proving warnings were not silently frozen.
4. Every frozen finding is traceable to a commit at or before `d1d7dc4f`.
5. `FINDINGS.md` classifies all 23 and names the follow-up refactor order.
