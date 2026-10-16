# SSH transport lifecycle — Plan

State: REVIEW (parent public process/provider smoke and integrated tests PASS; independent source review ACCEPT; #400 remains partial)
Branch: `main` (owner-authorized direct edits)
Issues: #435, #451, #400, #475
Baseline: `76b120724a7542de8b587a002811ddacad28156a`

## Requirements / context

The selected SSH route must not disappear between draft translation and connect/test/save. Password authentication must put `SSHPASS` on the process actually spawned, never in argv. A tunnel belongs to the provider connection through its awaited disconnect, including failure. Preflight must not contact the direct database or report an SSH handshake it did not perform.

At the baseline SHA above, `ssh/tunnel.rs:69-92` attaches the password to an unused command, `connector.rs:321-329` drops the tunnel before provider close, and native `translate_cmd.rs:25-35` discards network-driver tunnel intent except PostgreSQL. In contrast `connector.rs:332-342` already exercises SSH before TestConnection and keeps its handle through the test. The complete #400 diagnostic-stage acceptance is not already satisfied.

## Scope / architecture

- Infrastructure SSH command construction: one shared tunnel command builder, password environment on `sshpass`, omit empty identity arguments for start/test.
- Composite disconnect: remove the whole active record under the lock; release lock; await provider close; then drop tunnel on success/error.
- Native draft translation: preserve SSH for all network providers; reuse domain validation to reject unsupported SQL Server SSH on Test/Create/Update.
- Core diagnostics: defer tunneled preflight to the real connection test; omit unobserved SSH success and direct DB probes.
- Adjacent behavioral tests and this one plan directory.

## Non-goals

No SSH library, credential persistence changes, host-key relaxation, new command/event protocol, UI layout/panel redesign, or mandatory network access on Save. Parent owns final tests/build/format/review/commit/push/issue closure. No claim of real SSH DB authentication from local fixtures.

## Provider matrix

| Provider | Intent | Required evidence |
|---|---|---|
| PostgreSQL | supported generic SSH transport | password process + disconnect sockets + failed-hop routing; live SSH/DB pending |
| MySQL | preserve requested SSH and use shared transport | Test/Create/Update translation/domain validation + failed-hop routing; live SSH/MySQL pending |
| SQL Server | unsupported by existing domain validator | translated Test/Create/Update reaches explicit `ssh_tunnel` validation error, not direct connect |
| SQLite | existing local-file path unchanged | existing SQLite translation test remains authoritative; no SSH support added |

## Definition of done / acceptance

- Spawned password process receives environment secret, no secret argv, no empty `-i`.
- Socket-owning tunnel process remains alive across provider disconnect await and dies after success/error; provider error preserved.
- MySQL tunnel intent preserved for Test/Create/Update; SQL Server intent is not erased before existing validation.
- Connect/Test never contact a reachable direct DB port after the SSH hop fails.
- Diagnostics do not probe direct DB endpoints or invent SSH success for tunneled configs.
- Parent executes final gates and records results before closing independently proven #435/#451/#475.
- #400 remains OPEN: full-chain path already exists and gains failure-route regression, but separate handshake/bind/TCP/auth stage evidence and live SSH/DB authentication are not provided by this change.
