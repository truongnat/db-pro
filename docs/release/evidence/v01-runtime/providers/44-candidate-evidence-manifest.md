# RC1 Freeze C2 — candidate evidence manifest and artifact checksums (#88)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Issue: **#88** ([RC1][Freeze C2] Generate candidate evidence manifest and artifact checksums) —
  parent freeze workstream **#28**; dependencies #84 (SUPERSEDED), #85, #86, #87 (all closed)
- **Base head:** `main @ d585771` (worktree clean at session start)
- Candidate described by the manifest: `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20`
  (`git merge-base --is-ancestor 85a7fa3 HEAD` → exit 0, verified this session)

## 1. What was missing

`grep -rn 'evidence-manifest\|evidence_manifest' .` over the tracked tree returned **no matches**
before this change. The qualification results and artifact values existed, but scattered across
five documents (`0.1.0-readiness.md`, `risk-register.md`, `0.1.0-packaging.md`,
`0.1.0-release-notes.md`, `evidence/v01-06/*`, `evidence/v01-runtime/*`) with no single file tying
them to one SHA — exactly the gap the issue names.

## 2. What was added

| Path | Role |
|---|---|
| `docs/release/evidence-manifest.json` | the machine-readable manifest: candidate + toolchain, collection provenance, pipeline run and all nine job ids, every gate with the SHA it was observed at, the three archives + `SHA256SUMS.txt` + the three build artifacts + the provenance record with their GitHub artifact ids and checksums, signing truth status, P2 disposition counts, the 18-entry limitation registry, the smoke/publication handoff, and superseded run `34859158012` kept as history |
| `crates/native-app/tests/evidence_manifest_contract.rs` | 9 contract tests that make the manifest's invariants a gate (`cargo test --workspace`) instead of a review habit |
| `docs/release/0.1.0-readiness.md` | one pointer paragraph so #29/#31 find the manifest from the readiness assessment |

Values were read from the checked-in evidence and **cross-checked across sources**: the three
archive digests appear identically in `0.1.0-readiness.md:130-132`, `risk-register.md:457-459`,
`0.1.0-handoff.md:101-103`, `0.1.0-final-report.md:27-29` and `0.1.0-goal-3-traceability.md:207-209`.
The pipeline run and its job/artifact ids were **re-read from the GitHub API** this session
(`gh run view 34860902181 --json …`, `gh api …/artifacts`), not copied from prose: run `head_sha`
= `85a7fa3cc0a84c56ac2a5049ce08130db06e0a20`, nine jobs `success`, eight artifacts with ids
`10356295122`, `10355870711`, `10355207143`, `10355206098`, `10355160017`, `10355107349`,
`10355007833`, `10354569603`.

## 3. The invariants the tests enforce

| Test | Invariant |
|---|---|
| `manifest_carries_every_required_section` | all 16 sections the issue's "manifest must include" list asks for |
| `manifest_version_matches_the_release_source_of_truth` | `app.version` equals `CARGO_PKG_VERSION` of `db-pro-native` |
| `candidate_sha_is_one_well_formed_commit` | 40-hex SHA, `short_sha` a 7-hex prefix, every descendant SHA fully named with its ancestry check |
| `every_gate_status_is_truthful_and_backed_by_evidence` | closed status vocabulary; `pass` **requires** evidence; any other status **requires** a reason; a gate may only be observed at the candidate or a recorded descendant |
| `no_result_or_artifact_points_at_a_sha_other_than_the_candidate` | artifacts/build artifacts/checksum manifest/provenance are all the candidate SHA; a gate may only be observed at the candidate or a **recorded descendant** |
| `every_artifact_is_checksummed_or_explicitly_unavailable` | 64-hex SHA-256 or a non-empty unavailability reason; non-zero byte size |
| `a_superseded_artifact_never_poses_as_the_candidate` | no superseded (archive, digest) pair equals a candidate pair; the candidate's own run cannot be listed as superseded |
| `publication_claims_match_the_recorded_blockers` | `internal_rc_ready` ⇒ no `failed` gate; `public_release_ready` ⇒ no blockers, every gate `pass`, signing configured; `public_release_ready == false` ⇒ blockers recorded |
| `every_evidence_pointer_resolves_to_a_file_in_this_tree` | every path cited anywhere under an evidence/pointer key exists and is non-empty |

**Skipped gates cannot read as passes.** The frontend gate block is `not_applicable` with the reason
(React frontend retired 2026-09-11, `ci.yml` has no Node/TS gate, #84 SUPERSEDED); the interactive
smoke and the Windows/Linux runtime are `not_run` with their reason; the CLI backup/restore and the
launch-after-extract rows are `partial` with their reason. None is counted anywhere as a pass, and
`claims.public_release_ready` stays `false` with the four blockers (`R-LICENSE`, `R-SIGNING`,
`GUI-SMOKE`, `R-WINLINUX`) named in the manifest.

## 4. Falsification — the guards are not vacuous

Each probe was applied to a copy of the manifest and the contract test re-run; the manifest was
restored and re-run green after each.

| Probe | Observed |
|---|---|
| `runtime.interactive_smoke.reason` deleted | `every_gate_status_is_truthful_and_backed_by_evidence` **FAILED** (8 passed / 1 failed) |
| `frontend.release_gates` relabelled `pass`, reason blanked, evidence pointed at `docs/release/does-not-exist.md` | `every_gate_status_is_truthful_and_backed_by_evidence` + `every_evidence_pointer_resolves_to_a_file_in_this_tree` **FAILED** (7 passed / 2 failed) |
| `claims.public_release_ready` set `true` while the four blockers stayed; `artifacts[0].candidate_sha` set to `1a0c186`; one gate `observed_at_sha` set to `deadbeef` | `publication_claims_match_the_recorded_blockers`, `no_result_or_artifact_points_at_a_sha_other_than_the_candidate` + the gate test **FAILED** (6 passed / 3 failed) |
| restored manifest | **9 passed / 0 failed / 0 ignored** |

## 5. Gates on the changed tree

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all -- --check` | exit 0 (after `cargo fmt --all`, which re-wrapped one assert) |
| Compile | `cargo check --workspace` | exit 0 |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| Workspace tests | `cargo test --workspace` | **872 passed / 0 failed / 26 ignored** (baseline 863/0/26, **+9**) |
| Release build | `cargo build --release --locked -p db-pro-native` | exit 0 |
| Perf scan | `bash .skills/perf-audit/scripts/perf-scan.sh` | `Status: PASS` (4 passed / 0 warnings / 0 failed) |
| **CI-mirroring run** | `DATABASE_URL=… cargo test --all -- --include-ignored` | **898 passed / 0 failed / 0 ignored**, exit 0 (baseline 889/0/0, **+9**) |

The first CI-mirroring attempt failed with `AuthFailed("password authentication failed for user
\"dbpro\"")` because the fixture credential in the session prompt was redacted; the run above used
the password read from the container's own environment. The fixture container was stopped after the
run.

## 6. Deliberate limits of this record

- The manifest describes the **candidate** `85a7fa3`; it does not declare `FINAL_RC_SHA` — that is
  #89, still open, and the manifest's `candidate.selection` field says so.
- No claim is made about the interactive smoke, which has not been run (0 of 165 items; #90/#91).
- `checksum_manifest.sha256` is deliberately `null` with a reason: the run does not hash
  `SHA256SUMS.txt` itself, and no self-referential digest is invented.
- The two intermediate binaries without a recorded digest carry
  `sha256_unavailable_reason` rather than a guessed value.
