# Secret Fallback Coverage

State: COMPLETED

## Goal

Cover the development encrypted secret fallback and crypto primitives with
deterministic tests so corruption, wrong keys, persistence and deletion do not
remain unverified.

## Scope

- Argon2 key derivation determinism;
- AES-GCM round-trip and authentication failures;
- fallback blob validation;
- fallback store round-trip, missing/delete/reopen and corrupt-file behavior.

## Non-goals

- changing the fallback algorithm or production key policy;
- platform keyring qualification;
- native UI runtime verification.

## Acceptance

- crypto and fallback tests are deterministic and secret-safe;
- malformed/corrupt data returns an error;
- persistence across reopen and deletion are covered;
- targeted tests and repository quality gates pass.
