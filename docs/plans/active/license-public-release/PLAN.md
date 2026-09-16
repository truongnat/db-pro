# DB Pro license and public-release policy

## Goal

Record the owner-approved MIT policy and remove the undefined-license state
from the current release documentation and Rust package metadata.

## Scope

- add the MIT license text;
- declare MIT metadata for every workspace package;
- record the DB Pro trademark boundary;
- add the third-party notice inventory index;
- reconcile current release/readiness documents.

## Non-goals

- renaming source identifiers, binaries, bundle IDs, or persisted paths;
- changing release runtime support, signing, or GUI-smoke decisions;
- legal clearance of the DB Pro trademark;
- generating a final platform-specific binary notice bundle.

## Acceptance

- `LICENSE` exists and is the selected MIT policy;
- all workspace packages inherit `license = "MIT"`;
- current release documents no longer say the license is undecided;
- remaining runtime/platform blockers remain accurately open;
- tests/gates actually executed are recorded in `VERIFICATION.md`.
