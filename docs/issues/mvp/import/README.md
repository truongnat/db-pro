# MVP Issue Import Files

## Files
- `linear-import.csv`: CSV aligned to Linear CSV import fields (`Title, Description, Priority, Status, Assignee, Created, Completed, Labels, Estimate`).
- `github-import.csv`: CSV for GitHub issue import tooling (for example `aboutcode-org/github-import-issues-csv`) with placeholders for account/repo.

## Notes
- Replace placeholders in `github-import.csv`:
  - `<ACCOUNT_NAME>`
  - `<REPO_NAME>`
- Labels are prefilled as `mvp,sprint-x,Sx-yy`.
- Source tickets: `docs/issues/mvp/S1-01.md` ... `S3-06.md`.

## One-Command Import (GitHub)
- Dry-run preview (safe default):
  - `./scripts/import_github_issues.sh --csv docs/issues/mvp/import/github-import.csv --repo <owner>/<repo>`
- Create issues for real:
  - `./scripts/import_github_issues.sh --csv docs/issues/mvp/import/github-import.csv --repo <owner>/<repo> --apply`
- Optional limit (smoke-test first 2 issues):
  - `./scripts/import_github_issues.sh --csv docs/issues/mvp/import/github-import.csv --repo <owner>/<repo> --limit 2 --apply`
