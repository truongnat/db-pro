# Plan — Safe SQLite Sample Seeding

## Objective

Fix issue #492 so startup never registers an arbitrary or symlinked `/tmp` file as the writable Sample E-Commerce database.

## Target Branch

`fix/safe-sample-sqlite-seed`

## Severity

P1 — startup could silently expose a user-owned SQLite database as a writable demo connection.

## Scope

- Store the fixture under the app's resolved per-user data directory.
- Create the fixture only at a no-overwrite path; validate app id and fixture marker before reuse.
- Reject symlinks/unrecognized existing files and keep the seeded connection read-only.
- Migrate the prior built-in `/tmp/db_pro_sample.db` connection entry to the verified per-user fixture.
- Add regression coverage for existing files, symlinks, idempotence, and read-only configuration.
