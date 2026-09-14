DB Pro – Final v0.1 Release Closure

Current baseline:
- V01-01 Native Visual Verification: PASS
- V01-02 Query Editor Runtime Verification: PASS
- V01-03 Large-Schema ER Runtime Verification: PASS
- V01-04 Schema Runtime Verification: PASS
- V01-05 Integrated RC1 Runtime Smoke: PASS

Long-term product goals are now defined in:
- docs/goals/goal-full-product.md
- docs/goals/goal-phase-a-object-crud.md
- docs/goals/goal-phase-b-routines.md
- docs/goals/goal-phase-c-transfer.md
- docs/goals/goal-phase-d-monitoring.md
- docs/goals/goal-phase-e-security.md
- docs/goals/goal-phase-f-compare-migration.md
- docs/goals/goal-phase-g-productivity.md
- docs/goals/goal-phase-h-ai.md

DO NOT begin Phase A yet.

This task is exclusively:

V01-06 Cross-Platform Release Build
+
V01-07 Final Release Sign-off / Release Candidate preparation.

==================================================
0. FIRST — AUDIT CURRENT RELEASE STATE
==================================================

Read:

docs/notes/V0_1_CLOSURE_PLAN.md
docs/plans/STATUS.md
docs/release/0.1.0-readiness.md
docs/release/0.1.0-manual-smoke.md
docs/release/risk-register.md
docs/release/provider-capability-matrix.md
docs/plans/FEATURE_LIFECYCLE.md
.github/workflows/*
Cargo.toml
crates/native-app/Cargo.toml
README.md
CHANGELOG.md

Also inspect latest commits:

53e89f5
3e0b077

Confirm that the current HEAD includes:
- V01-01..05 runtime evidence
- full product goals
- no uncommitted Phase A production code.

Before doing release work, report internally:

- current HEAD
- current branch
- dirty/clean worktree
- package name/version
- current Rust toolchain
- supported targets
- release workflow status.

==================================================
1. VERIFY V01-01 → V01-05 EVIDENCE CONSISTENCY
==================================================

Do not rerun the entire native QA matrix unless needed.

Audit evidence recorded by commit 53e89f5.

Verify that PASS claims have actual corresponding evidence files.

Specifically:

V01-01:
- visual verification docs
- light/dark traversal
- viewport matrix
- independent review evidence.

V01-02:
- Query Editor runtime verification
- PG + SQLite
- cancellation
- multi-result
- completion
- diagnostics
- draft/history.

V01-03:
- ER 1000-table evidence
- timing evidence
- pan/zoom
- LOD
- BFS
- worker lifecycle.

V01-04:
- Schema introspection
- PG
- SQLite
- columns
- indexes
- relations
- triggers
- DDL.

V01-05:
- RC1 manual smoke
- connection lifecycle
- table mutation
- conflict resolution
- readonly/destructive safety
- export
- backup
- restore
- workspace recovery
- Agent preview.

If any PASS is documentation-only without evidence:
DO NOT silently keep PASS.

Mark:
EVIDENCE_GAP

and record exactly what is missing.

Do not fabricate screenshots, metrics, provider runs or reviewer approval.

==================================================
2. EXACT-HEAD QUALITY GATES
==================================================

Run on current HEAD:

cargo fmt --all -- --check

cargo check --workspace

cargo clippy --workspace --all-targets -- -D warnings

cargo test --workspace

cargo build --release --locked -p db-pro-native

bash .skills/perf-audit/scripts/perf-scan.sh

Record EXACT:

- commit SHA
- cargo version
- rustc version
- test total
- passed
- failed
- ignored
- clippy warning count
- release binary size
- perf scan result.

Do not copy previous counts.

If docs currently say:
808
809
571
etc.

replace stale counts where those docs claim CURRENT HEAD.

Historical counts can remain if clearly labelled historical.

==================================================
3. RELEASE BINARY AUDIT
==================================================

Inspect current native release output.

Record:

- executable name
- path
- size
- linked runtime/system libraries where relevant
- startup result
- architecture.

On macOS current host:

file target/release/db-pro-native

codesign info if available

otool -L if useful.

Launch release build, not debug.

Verify minimum smoke:

- app opens
- native window renders
- Settings accessible
- SQLite fixture opens
- Query executes
- app exits normally.

This is artifact smoke, not feature QA.

==================================================
4. CROSS-PLATFORM RELEASE WORKFLOW AUDIT
==================================================

Inspect .github/workflows release/build workflows.

Expected matrix:

macOS
- Apple Silicon
- Intel if intentionally supported.

Windows
- x86_64 MSVC.

Linux
- x86_64 GNU.

Determine actual support contract.

Do not claim targets that CI does not build.

For every platform determine:

- runner
- Rust target
- binary artifact
- packaging format
- artifact retention
- checksum generation
- signing status.

Create a truthful matrix.

Example:

Platform      Compile      Artifact      Installer      Signing
macOS ARM64   YES          binary        ?              ?
macOS x64     YES/NO       binary        ?              ?
Windows x64   YES          .exe          ?              ?
Linux x64     YES          binary        ?              ?

Do not conflate:
binary artifact
with
installer.

==================================================
5. PACKAGING CONTRACT DECISION
==================================================

Previous docs mention combinations such as:

.dmg
.msi
.deb
AppImage
tarball

But actual native packaging support must be audited.

Choose an explicit v0.1 packaging contract.

A valid v0.1 decision can be:

Option A:
portable native binaries only.

Option B:
platform installers.

Option C:
mixed:
macOS .app/.dmg
Windows .exe
Linux tar.gz

Do not implement five packaging ecosystems simply because old docs listed them.

Decision criteria:

- reliable
- reproducible
- CI-friendly
- native app launchable
- appropriate for v0.1.

Document intentional deferrals.

==================================================
6. NATIVE APP METADATA
==================================================

Audit:

app name
version
bundle identifier
window title
binary name
description
repository
authors
license metadata
icons/resources.

Ensure user-facing version is consistent:

0.1.0

or release-candidate representation if project policy requires:

0.1.0-rc.1

Do not randomly change semantic version.

Check:

Cargo workspace version
native crate version
README
CHANGELOG
release docs.

==================================================
7. MACOS RELEASE
==================================================

For macOS supported architectures:

build via CI or locally where possible.

Verify:

- release binary launches
- correct architecture
- no missing resources
- no dev-only path dependency
- SQLite works
- filesystem persistence works.

If producing .app bundle:

verify structure:

DB Pro.app/
Contents/
MacOS/
Resources/
Info.plist

Check:
CFBundleIdentifier
CFBundleShortVersionString
CFBundleVersion
minimum macOS target if defined.

If .dmg is NOT implemented:
state explicitly:
DEFERRED / NOT PART OF v0.1 CONTRACT.

Do not mark packaging PASS based solely on raw binary if docs promise DMG.

Either:
implement agreed contract
or
update contract.

==================================================
8. WINDOWS RELEASE
==================================================

Use CI target:

x86_64-pc-windows-msvc

Verify compile.

Artifact:
db-pro-native.exe

Check:
- artifact uploaded
- no runtime DLL omitted if needed
- no Unix-path assumptions
- no shell-command assumptions
- SQLite local paths use platform-safe APIs.

Inspect backup/export/save paths for Windows path correctness.

Installer:
if MSI/NSIS absent and not release contract:
document deferred.

Do not invent manual Windows runtime evidence if no Windows host/run exists.

Use:
BUILD_VERIFIED

vs:
RUNTIME_VERIFIED

as separate statuses.

==================================================
9. LINUX RELEASE
==================================================

Target:

x86_64-unknown-linux-gnu

Verify required native dependencies.

egui/eframe/winit may require system libs.

Audit workflow install dependencies such as:

X11
Wayland
libxcb
OpenGL
fontconfig
etc.

Verify compiled artifact.

If CI can run headless smoke safely:
perform minimal startup or help/version smoke.

Do not claim graphical runtime verification without evidence.

Package:
tar.gz is acceptable for v0.1 if explicitly chosen.

.deb/AppImage only if project intentionally supports them now.

==================================================
10. RELEASE ARTIFACT NAMING
==================================================

Standardize artifacts.

Example:

db-pro-v0.1.0-macos-aarch64.tar.gz
db-pro-v0.1.0-macos-x86_64.tar.gz
db-pro-v0.1.0-windows-x86_64.zip
db-pro-v0.1.0-linux-x86_64.tar.gz

Inside:
- executable / app bundle
- README or install note if needed
- LICENSE only if license decision exists.

Generate SHA256 checksums:

SHA256SUMS.txt

Do not include:
debug binaries
incremental artifacts
API keys
test DB credentials
local settings.

==================================================
11. LICENSE BLOCKER
==================================================

Audit current repository license.

Old status documents mention:
"Public project license is not defined."

This matters before publishing a release.

Determine current state.

If no LICENSE exists:

DO NOT choose a license autonomously.

Mark governance blocker:

R-LICENSE

User decision required.

Update risk register.

Release can be technically ready but must not be represented as publicly licensed without a decision.

==================================================
12. BRAND / NAMING
==================================================

Audit risk register for brand naming.

Check:

DB Pro
binary names
app titles
package names.

If unresolved branding risk exists:

record:
ACCEPT
DEFER
BLOCK

according to current project policy.

Do not rename application without explicit instruction.

==================================================
13. CODE SIGNING / NOTARIZATION
==================================================

Audit current status for:

macOS signing
macOS notarization
Windows signing.

Separate:

unsigned artifact usable
vs
signed artifact ready for public distribution.

For v0.1 choose/document contract.

If unsigned:

state:
UNSIGNED

and expected OS warning.

Do not pretend signing is configured.

Never add secrets to repo.

==================================================
14. RELEASE WORKFLOW
==================================================

Harden GitHub Actions only if required for V01-06.

Desired flow:

on:
- workflow_dispatch
- optionally tag v*

jobs:
quality
macos
windows
linux

Quality:
fmt
check
clippy
tests

Build jobs:
cargo build --release --locked -p db-pro-native

Artifacts:
named by platform.

Optional final job:
checksums / release assembly.

Do not introduce unrelated CI refactors.

==================================================
15. REPRODUCIBILITY
==================================================

Verify release uses:

Cargo.lock
--locked

No:
git dependency floating branch
network-downloaded runtime assets at launch
local absolute paths.

Audit embedded assets.

Ensure fresh checkout can build.

Document required native packages.

==================================================
16. SECRETS / CREDENTIAL RELEASE AUDIT
==================================================

Search repository/worktree for accidental:

API keys
database passwords
SSH keys
fixture secrets
provider tokens
local connection metadata.

Fixture credentials such as:
postgres/postgres

are okay only if clearly test-only.

Never package:
user config
user DB files
credentials
agent provider keys.

Check generated artifacts if possible.

==================================================
17. RELEASE DATA / MIGRATION SAFETY
==================================================

Audit app state compatibility.

Current v0.1 install/update must not unexpectedly destroy:

connections
saved queries
query history
workspace tabs
settings.

Check workspace/meta schema version migration.

If this is first public release:
document baseline migration version.

If upgrading from previous native dev builds:
verify migration behavior where reasonable.

==================================================
18. STARTUP FAILURE HANDLING
==================================================

Release binary should fail gracefully if:

state DB corrupted
font unavailable
workspace state invalid
connection missing
provider key missing.

No startup panic.

Do targeted smoke around current recovery paths.

Do not open a new hardening project unless blocker found.

==================================================
19. RELEASE NOTES
==================================================

Prepare:

docs/release/0.1.0-release-notes.md

Sections:

DB Pro 0.1.0

Highlights:
- Native egui application
- PostgreSQL + SQLite
- Connection Explorer
- Query Editor
- Table Data Editor
- Schema workbench
- ER Diagram
- Agent Preview
- Export / Backup subset.

Safety:
- readonly enforcement
- destructive confirmations
- mutation rollback/conflict resolution.

Known limitations:
use capability matrix honestly.

Explicitly mention not included:
- Monitoring
- Data Import
- full Object CRUD
- Routine editing
- Users/Roles UI
- Schema Migration
- MCP
- advanced AI.

Do not market unimplemented goals as shipped features.

==================================================
20. README RELEASE ALIGNMENT
==================================================

Review README.

Ensure README distinguishes:

Available now

vs

Roadmap.

Do not show roadmap feature as current capability.

Link:

PRODUCT_ROADMAP
goal-full-product
release notes.

Keep README concise.

==================================================
21. CHANGELOG
==================================================

Update CHANGELOG.md.

Add:

## 0.1.0

or
## [0.1.0] - YYYY-MM-DD

Include meaningful user-facing changes.

Do not dump every commit.

Sections:

Added
Changed
Fixed
Security/Safety
Known limitations if project format supports it.

==================================================
22. STATUS TRANSITIONS
==================================================

Read FEATURE_LIFECYCLE.md carefully.

For each v0.1 feature whose required runtime evidence is now complete:

decide whether it may move:

RUNTIME_VERIFY
→ COMPLETED

Do not mass-complete blindly.

Candidate features:

Core Safety Hardening
Table Data Editor Hardening
Query Editor Intelligence
Agent Workflow (may remain Preview even when verified)
P1 Large-Schema ER Architecture
RC1 Full Product QA
Native Visual Redesign
S1–S7 schema work.

For every transition record:
- evidence
- commit
- provider
- runtime state.

If lifecycle demands REVIEW before COMPLETED:
follow lifecycle exactly.

==================================================
23. ACTIVE PLAN ARCHIVAL
==================================================

Only when feature is COMPLETED:

move:
docs/plans/active/<feature>
→
docs/plans/completed/<feature>

Do not archive:
- plans with pending evidence
- roadmap goals
- post-v0.1 plans.

Keep goals under docs/goals, not plans/completed.

==================================================
24. RC1 P2 ITEMS
==================================================

RC1 currently historically recorded 25 P2 findings.

Audit dispositions.

For each:

FIXED
ACCEPTED
DEFERRED
OBSOLETE
STILL_OPEN.

P2 does not automatically block v0.1.

But identify any P2 that is actually:
- crash
- data loss
- credential leak
- destructive SQL misclassification.

Those must be promoted to P0/P1 and fixed before release.

Do not mass-fix visual polish.

==================================================
25. RISK REGISTER
==================================================

Update:

docs/release/risk-register.md

Every open risk must have:

ID
description
severity
owner/decision
status
release disposition.

Allowed release disposition:

FIXED
ACCEPTED
DEFERRED
BLOCKING.

Focus:

license
signing
branding
SSH cross-platform
packaging
Agent Preview
provider limitations.

==================================================
26. PROVIDER CAPABILITY MATRIX
==================================================

Update only if runtime evidence changed.

Ensure matrix correctly represents:

PostgreSQL
SQLite.

Distinguish:

SUPPORTED
SUPPORTED + QUALIFIED
SUPPORTED + NOT YET QUALIFIED
UNSUPPORTED
DEFERRED.

Do not mark:
PostgreSQL feature supported
because SQLite passed.

==================================================
27. V01-06 RESULT
==================================================

At end of V01-06 report:

Quality gates:
PASS / FAIL

macOS ARM64:
BUILD
RUNTIME
PACKAGE
SIGNING

macOS x64:
BUILD
RUNTIME
PACKAGE
SIGNING

Windows x64:
BUILD
RUNTIME
PACKAGE
SIGNING

Linux x64:
BUILD
RUNTIME
PACKAGE
SIGNING

Artifacts:
list exact names.

Checksums:
PASS/FAIL.

Remaining blockers:
list exactly.

Update:
docs/notes/V0_1_CLOSURE_PLAN.md

V01-06:
PASS
FAIL
or
PARTIAL.

Do not start V01-07 if blocking build issue exists.

==================================================
28. V01-07 — FINAL SIGN-OFF PREPARATION
==================================================

If V01-06 is PASS or only contains explicitly accepted governance risks:

prepare final release candidate state.

Do not push a public release tag automatically unless repository instructions explicitly authorize it.

Prepare:

docs/release/0.1.0-readiness.md

with:

READY_FOR_RELEASE:
YES / NO

Candidate SHA:
<exact sha>

Quality gates:
<results>

Runtime QA:
<results>

Providers:
PG
SQLite

Artifacts:
<list>

Known limitations:
<list>

Accepted risks:
<list>.

==================================================
29. RELEASE CANDIDATE TAG STRATEGY
==================================================

Determine from project policy whether final tag should be:

v0.1.0

or first:

v0.1.0-rc.1.

Recommendation if no prior release qualification:

v0.1.0-rc.1

then host-install smoke,
then v0.1.0.

But do not change existing release policy without documenting decision.

==================================================
30. FINAL INSTALL SMOKE
==================================================

For host platform:

Test artifact from packaged output,
not target/release directly.

Steps:

install/extract
launch
create SQLite connection
run:
SELECT 1;

open Explorer
open Query
open Settings
exit
relaunch
verify persisted state.

If PG test fixture available:
connect
SELECT version();
disconnect.

No deep QA repetition necessary.

==================================================
31. NO FEATURE EXPANSION
==================================================

Forbidden in this task:

Phase A object CRUD
Functions editor
Monitoring
Import
Users/Roles UI
Schema Compare
Migration
Global Search
Advanced AI
Agent new tools
ER edit mode
new visual redesign.

If you encounter an enhancement idea:
record it in roadmap.

Do not implement it.

==================================================
32. PRODUCTION CODE CHANGES POLICY
==================================================

Production code may only change for:

- release build blocker
- portability blocker
- packaging blocker
- startup crash
- data-loss bug
- credential/security bug
- current v0.1 contract regression.

Every code fix must include:
- root cause
- focused regression test where possible
- quality gates rerun.

Do not refactor unrelated modules.

==================================================
33. DOC CONSISTENCY
==================================================

After all work search for contradictions in:

STATUS.md
07-current-status.md
0.1.0-readiness.md
V0_1_CLOSURE_PLAN.md
PRODUCT_CAPABILITY_MATRIX.md
README
CHANGELOG.

Examples to eliminate:

"Agent production excluded"
vs
"Agent Preview shipped"

"Data insert deferred"
vs
current Table Data Editor supports insert

"571 tests"
vs
current exact test count

"React frontend"
as current UI.

Historical docs can remain unchanged only when explicitly marked historical.

==================================================
34. LONG-TERM GOALS MUST REMAIN UNTOUCHED
==================================================

Do not alter feature scope in:

goal-full-product.md
goal-phase-a-object-crud.md
goal-phase-b-routines.md
goal-phase-c-transfer.md
goal-phase-d-monitoring.md
goal-phase-e-security.md
goal-phase-f-compare-migration.md
goal-phase-g-productivity.md
goal-phase-h-ai.md

unless correcting a factual contradiction.

These are post-v0.1 product authority.

Do not mark Phase A as IMPLEMENTING.

==================================================
35. RELEASE HANDOFF DOCUMENT
==================================================

Create:

docs/release/0.1.0-handoff.md

Include:

Candidate SHA

Build matrix

Artifacts

Quality gate result

Runtime verification summary

Known limitations

Accepted risks

Blocking risks

Install smoke

Tag/release steps

Rollback instructions.

This should allow another engineer to release without reading all plan docs.

==================================================
36. RELEASE CHECKLIST
==================================================

Create compact final checklist:

[ ] exact HEAD identified
[ ] clean worktree
[ ] fmt PASS
[ ] check PASS
[ ] clippy PASS
[ ] tests PASS
[ ] release build PASS
[ ] perf PASS
[ ] macOS artifact
[ ] Windows artifact
[ ] Linux artifact
[ ] host install smoke
[ ] runtime V01-01..05 evidence
[ ] provider matrix current
[ ] license disposition
[ ] signing disposition
[ ] risk register current
[ ] README aligned
[ ] CHANGELOG updated
[ ] release notes ready
[ ] READY_FOR_RELEASE decision
[ ] candidate tag decision.

==================================================
37. COMMIT STRATEGY
==================================================

Prefer separate logical commits if production/CI fixes are needed:

fix(release): ...
ci(release): ...
docs(release): ...

If only documentation + release workflow:

ci(release): qualify cross-platform native artifacts

docs(release): prepare DB Pro v0.1 release candidate

Do not squash unrelated fixes into giant opaque commit if avoidable.

==================================================
38. FINAL RESPONSE
==================================================

At end report exactly:

HEAD:
<sha>

V01-06:
PASS / PARTIAL / FAIL

V01-07:
READY / BLOCKED

Quality gates:
fmt:
check:
clippy:
tests:
release:
perf:

Tests:
N passed / N failed

Artifacts:
macOS:
Windows:
Linux:

Host install smoke:
PASS/FAIL

License:
status

Signing:
status

Open P0:
N

Open P1:
N

Accepted P2/P3:
N

READY_FOR_RELEASE:
YES/NO

Next required human decision:
<if any>

Do not say "done" if READY_FOR_RELEASE=NO.

==================================================
39. STOP CONDITION
==================================================

STOP after release candidate preparation.

DO NOT:
- create Phase A branch
- implement A01
- add object CRUD
- implement Monitoring
- add Import
- start v0.2 work.

The next task after a clean v0.1 closure will explicitly start:

A01 — Typed Object Mutation Framework.

Until v0.1 closure is confirmed, Phase A remains PLANNING.
