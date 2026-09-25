# Security — implementation roadmap

- Source baseline SHA: `b0500b9a7ecbe37b454f3d917881154c5f7a403c`
- Inputs: [security-baseline.md](security-baseline.md), [security-feature-research.md](security-feature-research.md), [AGENT_EVIDENCE.md](AGENT_EVIDENCE.md)
- Evidence boundary: implementation statements refer to source at the baseline SHA. P1/P2 severities are carried forward from research, not re-triaged; no source-level risk below is presented as an observed incident.

## Decision

Treat Security as connection/database administration under Explorer; make RLS reachable from table context and separate it from role administration. Preserve current reachable capabilities during navigation changes. Do not call this complete until password confidentiality, reviewed access-change semantics, audit/outcome handling, and independent provider/native-UI verification are complete. PostgreSQL is the only implemented provider in these paths.

## Current state

### Implemented in source

- Native top-level Security rail activity; role list/details, selected role attributes, password update, memberships, object privileges and table RLS source paths exist (`security-baseline.md` lines 12–18).
- `UserService` enforces PostgreSQL-only routing and rejects configured read-only mutations; adapter quotes identifiers and allowlists privilege tokens (`security-baseline.md` lines 20–31).
- RLS has SQL preview/acknowledgment and applies through common DDL safety/read-only policy (`security-baseline.md` lines 16–18, 27).

### Not implemented / incomplete

- Explorer-scoped role administration and table-scoped RLS navigation are a recommendation, not current source behavior (`security-feature-research.md` lines 7–12, 22–35).
- General role/grant change-set preview, target-bound confirmation, service active-role-drop guard and durable Security mutation audit are absent (`security-feature-research.md` lines 18–20, 50, 53).
- Privilege results can be partial without marker; membership/effective-access view is narrower than PostgreSQL semantics; direct grants are not effective-permission analysis (`security-feature-research.md` lines 18, 41–43, 51–53).
- SQLite and MySQL/SQL Server are unsupported; they must be represented honestly, not as empty role lists (`security-feature-research.md` lines 37–45).

### Needs fix (source-derived, not runtime-reproduced)

- **P1:** password is composed into `ALTER ROLE ... PASSWORD ...` statement text and sent with no bind args. Dollar quoting prevents syntax break-out but not statement-text secrecy; PostgreSQL docs warn on cleartext transmission/logging (`security-baseline.md` lines 29–33; `security-feature-research.md` line 49).
- **P2:** direct role/grant/membership/password actions lack reviewed apply semantics; role-drop lacks service-level active-role guard (`security-feature-research.md` line 50; baseline lines 15–16, 29).
- **P2:** secondary privilege query failures are ignored, so partial results appear complete (`security-feature-research.md` line 51; baseline lines 25–27).
- **P2:** role/membership representation omits attributes/options and effective access; additional high-risk capabilities require explicit product scope (`security-feature-research.md` line 52).
- **P2:** no durable app-level audit record in inspected mutation flow; operation feedback must distinguish unsupported, denied, partial, and failed outcomes (`security-feature-research.md` line 53; baseline lines 31–33).
- **P2:** Phase E/current capability docs and current UI/RLS scope conflict (`security-feature-research.md` line 54; baseline lines 35–40).

## Ordered V3 backlog

| Priority | Type | Evidence | Concrete change / outcome | Dependencies | Observable acceptance criteria |
|---|---|---|---|---|---|
| P1 (inherited) | fix | Plaintext password embedded in SQL statement text: baseline lines 29–32; research line 49 | Replace password mutation transport with a supported mechanism that does not send plaintext as ordinary SQL statement text where provider protocol permits; redact secrets from command/debug/error paths; document protocol/config limits and do not claim server-log guarantees unsupported by evidence. | Establish supported PostgreSQL versions/connector protocol; first | Provider observation/logging test verifies password absent from statement text and app/runtime errors/logs; password update succeeds for supported PostgreSQL configuration; failure contains no secret. |
| P2 (inherited) | fix | Unreviewed direct mutations, simple drop confirmation, missing active-role guard: baseline lines 15–16, 29; research line 50 | Add service-level prohibition for dropping active connection role; stage role/grant/membership/sensitive attribute changes with before/after target, connection/database/environment, preview, explicit apply confirmation and refresh. Preserve existing read-only/routing checks. | P1 secret boundary for password changes; define preview representation | Service rejects current-role drop independent of UI; each high-impact mutation shows exact target and effect before apply; after apply UI refreshes authoritative role/grant state or presents failure. |
| P2 (inherited) | fix | Ignored secondary grant query errors: baseline lines 25–27; research line 51 | Return typed partial/error status with category/query failure and provide explicit retry/refresh; never present incomplete grants as a complete list. | Role/grant state UI contract | Simulated secondary-query failure produces visible partial/failed state and preserves known rows with category labels; successful retry replaces the partial state. |
| P2 (inherited) | missing | Narrow model and UI: research lines 18, 41–43, 52; baseline lines 15, 25 | Define initial supported role/membership semantics (direction, options, direct vs inherited/ownership/PUBLIC); expose effective access only if computed from authoritative source. Add high-risk attributes only by explicit product decision and privilege gating. | Product decisions in unresolved section; partial-result model | UI labels direct/effective distinctions accurately; membership direction/options shown for supported semantics; omitted capabilities are not implied as absent grants. |
| P2 (inherited) | missing | No app audit writer; audit reader is separate: baseline lines 31–33; research line 53 | Persist secret-safe audit outcomes: actor, target connection/database, operation, role/object, result, timestamp; exclude password and secret-bearing SQL. Present distinct denied/unsupported/partial/execution failure feedback. | Apply/result correlation and audit-store policy | Each applied or rejected mutation has an inspectable audit outcome; audit fields contain no password or full secret-bearing SQL; failures remain distinguishable from success. |
| P2 (inherited) | missing | Recommended Explorer/table IA differs from current rail mixed surface: research lines 7–12, 22–35; baseline lines 14–18 | Move roles/membership/grants to connection/database Security context; expose RLS from selected table and provide route to role administration; keep all existing capability paths reachable during cutover. | Navigation contract; no behavior cutover before actions safely preserved | Explorer connection/database exposes role workflows; table context exposes RLS; no current role/RLS operation becomes unreachable; top-level rail removal follows route verification. |
| P2 (inherited) | verification | Current docs stale and runtime evidence absent: baseline lines 35–46; research lines 54, 58–62 | Reconcile Phase E/current-state and capability docs; separate source, automated, PostgreSQL runtime, unsupported-provider, and native UI proof. | After behavior/IA changes | Docs describe implemented source truth and scoped RLS; lifecycle claims remain partial until independent gates are recorded. |
| Proposed P1 | verification | Password risk source-derived only; baseline lines 31, 42–46; research lines 49, 62 | Verify password secrecy and successful password change against supported PostgreSQL protocol/configurations without logging a production secret. | P1 password transport implementation | Test harness/provider inspection confirms secret absent from SQL text, wire/debug/error artifact; no secret value retained in test output. |
| Proposed P2 | verification | Research reports no build/test/UI/provider scenario: lines 60–62 | Add focused automated service/adapter tests and perform native UI and PostgreSQL runtime verification; verify SQLite/MySQL/SQL Server show explicit unsupported states without mutation dispatch. | Corresponding work complete | Automated tests cover service rejection and partial results; UI reflects scope/results; PostgreSQL scenarios separately prove role, membership, grant, RLS and read-only behavior; unsupported providers do not dispatch. |

## Rollout / dependency order

1. Resolve PostgreSQL password transport/confidentiality before expanding password workflows.
2. Add service-level mutation invariants, target-bound review/apply, refresh semantics and secret-safe audit/result records.
3. Make partial permission-query results explicit; define the first-release role/membership semantics before adding broader attributes or effective-access claims.
4. Move role and RLS navigation to their respective Explorer/table contexts without dropping reachable operations.
5. Reconcile current-state documentation and run automated, PostgreSQL runtime, unsupported-provider and native UI gates independently. No status promotion on source evidence alone.

## Provider/support matrix

| Workflow | PostgreSQL | SQLite | MySQL / SQL Server |
|---|---|---|---|
| Role administration | Source supports listing/create/drop/selected attributes/password, memberships and object grants; password secrecy and partial-result issues remain. Runtime unverified. | Explicitly unsupported by UserService; no role model in this path. | Unsupported by this API; no implementation evidence. |
| Membership / privileges | Source supports individual target kinds, but membership and privilege display/query may be partial; no effective privilege analyzer. Runtime unverified. | Unsupported. | Unsupported. |
| RLS | State and preview/apply paths exist; keep table-scoped and subject to DDL/read-only checks. Runtime unverified. | Explicitly unsupported. | Unsupported by this API. |

Support claims are only for the inspected source path; no provider runtime was exercised (`security-baseline.md` lines 20–33, 42–46; `security-feature-research.md` lines 37–45).

## Verification gates still needed (not run)

- Automated coverage of password non-disclosure and mutation safety/read-only/current-role boundaries.
- Automated partial-result/error-state checks and audit secret-redaction behavior.
- PostgreSQL runtime checks for password update mechanism, role/membership/grant/RLS workflow, privileges, read-only behavior and resulting state; do not use real secret values in artifacts.
- SQLite and MySQL/SQL Server checks confirm explicit unsupported UX and no PostgreSQL mutation dispatch.
- Native UI review of Explorer/table placement, preview/confirmation, error states and audit outcome.
- Inputs report no tests/build, native UI launch or provider runtime (`security-baseline.md` lines 42–46; `security-feature-research.md` lines 60–62).

## Out of scope / unresolved decisions

- SQLite, MySQL or SQL Server role/RLS implementations; no provider-specific authorization contract is present.
- Whether SUPERUSER, REPLICATION, BYPASSRLS, connection limits, validity and ownership actions are in initial scope; these are explicitly unresolved/high risk (`security-feature-research.md` line 52; `AGENT_EVIDENCE.md` lines 67–70).
- Exact password-setting protocol and supported PostgreSQL versions/configurations; app audit scope versus database/server audit policy (`security-feature-research.md` line 58; `AGENT_EVIDENCE.md` line 70).
- Effective-access analysis and default privileges until authoritative semantics and scope are specified.
