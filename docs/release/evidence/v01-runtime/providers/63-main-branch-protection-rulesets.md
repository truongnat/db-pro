# Protect `main` and release tags (#146)

- Session: 2026-09-15
- Issue: **#146** ([P1][RC1][Governance] Protect main and require release CI checks)
- Mechanism: GitHub **repository rulesets** (classic branch protection remains unused / 404)

## 1. Before

| Check | Result |
|---|---|
| `GET …/branches/main/protection` | HTTP 404 — Branch not protected |
| Ruleset `main` (id `20631004`) | `enforcement=active`, but `conditions.ref_name.include=[]` (matched **no** branch), rules only `deletion` + `non_fast_forward` |
| Required PR / status checks | absent |
| Tag policy for `v*` | absent |

## 2. After (API evidence)

### Branch ruleset `main` (id `20631004`)

```json
{
  "enforcement": "active",
  "conditions": { "ref_name": { "include": ["refs/heads/main"], "exclude": [] } },
  "rules": ["deletion", "non_fast_forward", "pull_request", "required_status_checks"],
  "bypass_actors": [{ "actor_id": 5, "actor_type": "RepositoryRole", "bypass_mode": "always" }],
  "current_user_can_bypass": "always"
}
```

| Requirement | Setting |
|---|---|
| Require a pull request | `pull_request` with `dismiss_stale_reviews_on_push=true`, approving-review count `0` (exact-head discipline keeps the process-level SHA check; reviewers are not the RC1 gate) |
| Required status checks | **`Rust checks`** — the live job name in `.github/workflows/ci.yml:15` |
| Strict status checks | `strict_required_status_checks_policy=true` |
| Force push | blocked (`non_fast_forward`) |
| Branch deletion | blocked (`deletion`) |
| Admin bypass | retained (`RepositoryRole` admin, `bypass_mode=always`) so release/emergency ops remain possible |

**Frontend checks:** the issue text still names `Frontend checks`. That job no longer exists — the React frontend was archived 2026-09-11 and `ci.yml` has no Node/TS gate. Requiring a ghost check would soft-lock every PR. The required check is therefore the real shipping gate: `Rust checks`.

### Tag ruleset `release-tags` (id `23461211`)

```json
{
  "enforcement": "active",
  "conditions": { "ref_name": { "include": ["refs/tags/v*"], "exclude": [] } },
  "rules": ["deletion", "non_fast_forward", "creation"],
  "bypass_actors": [{ "actor_id": 5, "actor_type": "RepositoryRole", "bypass_mode": "always" }]
}
```

`v*` tags may only be created/updated/deleted by the admin bypass role — matches the issue's "release-tag creation/deletion policy is explicitly defined".

## 3. Verification commands

```bash
gh api repos/truongnat/db-pro/rulesets/20631004 --jq '{name,enforcement,conditions,rules:[.rules[].type],bypass:.bypass_actors}'
gh api repos/truongnat/db-pro/rulesets/23461211 --jq '{name,enforcement,conditions,rules:[.rules[].type]}'
gh api repos/truongnat/db-pro/branches/main/protection   # still 404 — rulesets are the mechanism
```

## 4. Process note

The workstream's documented main-only push path continues to work for repository admins via the retained bypass. Non-admin contributors must open a PR whose head is green on `Rust checks`.
