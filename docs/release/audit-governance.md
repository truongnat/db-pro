# Governance Audit — Branch Protection, Merge Policy, Release Permissions

> Baseline SHA: `a2cf4a3`
> Issue: #130
> Source: GitHub API (repository settings, rulesets, workflows), `.github/workflows/`

## Repository settings

| Setting | Current value | Risk |
|---|---|---|
| Default branch | `main` | — |
| Branch protection on `main` | **NONE** | **P1** |
| Rulesets | **NONE** | **P1** |
| Collaborators | `truongnat` (admin) | — |
| Token permissions (current user) | admin, maintain, pull, push, triage | — |

## Branch protection gaps

**Current state**: `main` is **completely unprotected**. The API returns `"Branch not protected"`.

| Required protection | Status | Risk |
|---|---|---|
| Require PR before merge | **Not enforced** | Anyone with push can commit directly to main |
| Required status checks | **Not enforced** | CI can be bypassed |
| Required reviews | **Not enforced** | Self-merge without review |
| Dismiss stale reviews | **Not configured** | — |
| Restrict force pushes | **Not enforced** | Force push can rewrite history |
| Restrict deletions | **Not enforced** | — |
| Require signed commits | **Not enforced** | — |

## Workflow permissions

| Workflow | Trigger | Permissions | Concerns |
|---|---|---|---|
| `CI` (`ci.yml`) | push/PR to main | Default `GITHUB_TOKEN` | Reads code, reports status. No elevated permissions needed. |
| `Release Build` (`release.yml`) | Not inspected in detail | Potentially `contents: write` for releases | Must verify least-privilege |
| `VPS PR Review` (`vps-pr-review.yml`) | PR to main | Read-only VPS reviewer | Intentionally read-only per AGENTS.md |

**Finding F1 [P1]**: No branch protection on `main`. Any collaborator can push directly, bypassing CI, review, and the exact-head verification discipline described in AGENTS.md.

**Finding F2 [P2]**: No tag protection rules for `v*` tags. Any collaborator could create or delete release tags.

**Finding F3 [P2]**: No CODEOWNERS file. PR review is not routed to any specific reviewer.

**Finding F4 [P2]**: Only one collaborator (`truongnat` with admin). No separation between release writers and regular contributors (because there's only one).

## Required checks (should match CI jobs)

| CI Job Name | Should be required? |
|---|---|
| `Rust checks` | Yes |
| `Frontend checks` | Yes |

## Minimum governance policy for v0.1

| Policy | Implementation |
|---|---|
| Feature changes via PR only | Enable "Require pull request before merging" |
| CI must pass | Enable "Require status checks to pass" with `Rust checks` and `Frontend checks` |
| No direct push to main | Enforced by PR requirement above |
| No force push | Enable "Do not allow force pushes" |
| No branch deletion | Enable "Do not allow deletions" |
| Exact-head verification | Add expected-head-SHA check to merge process (manual discipline, not GitHub-enforced) |
| Release tag verification | Enable tag ruleset for `v*` pattern |

## Action items

| # | Action | Severity | Who |
|---|---|---|---|
| 1 | Enable branch protection on `main`: require PR, require CI status checks (`Rust checks`, `Frontend checks`), no force push, no deletion | **P1** | Repository admin (manual) |
| 2 | Create tag ruleset for `v*` pattern: restrict creation/deletion | P2 | Repository admin (manual) |
| 3 | Add `CODEOWNERS` if team grows beyond 1 | P3 | — |
| 4 | Verify `release.yml` uses least-privilege `GITHUB_TOKEN` permissions | P2 | Next audit pass |

## Decision record

- **D1**: Branch protection must be enabled before any public release. Current state is acceptable only for solo development.
- **D2**: GitHub connector cannot configure branch protection (API returns 404 for protection endpoints — may need classic token with `repo:admin` scope). These settings must be configured manually by the repository admin.
- **D3**: The VPS PR reviewer infrastructure (REVIEW.md, .kilo/, vps-pr-review.yml) is intentionally read-only and must not be removed per AGENTS.md.

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 1 | F1 — No branch protection on main |
| P2 | 3 | F2-F4 |

**Conclusion**: Repository governance is minimal (solo developer, no protection). The single P1 finding (no branch protection) must be resolved before public release. All settings changes are manual — the GitHub API connector lacks admin-scope tokens to configure them programmatically.
