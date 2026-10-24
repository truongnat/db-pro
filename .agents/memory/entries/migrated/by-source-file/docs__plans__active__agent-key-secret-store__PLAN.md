---
schema_version: 1
id: "MEM-SRC-a0a429663c59bab7"
title: "Source mirror: docs/plans/active/agent-key-secret-store/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/agent-key-secret-store/PLAN.md"
source_sha256: "a0a429663c59bab76cf16e2c023f0854afe381140755231a4fff046cb83202ba"
mirror_body_sha256: "a0a429663c59bab76cf16e2c023f0854afe381140755231a4fff046cb83202ba"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Agent API Key Secret Store

## Goal

Keep the Agent API key lifecycle inside the runtime `SecretStore`: load it at
startup, persist it on configuration, and delete it on “Forget key”.

## Scope

- replace native-app direct keyring access with runtime secret-store calls;
- preserve the existing key name for compatibility;
- make configuration fail before provider activation when persistence fails;
- make forgetting remove the secret from all configured stores;
- add focused tests for provider selection and command/event routing.

## Non-goals

- platform packaged-runtime qualification;
- changing the public DB Pro identity or secret-store backend policy;
- adding a new Agent provider or changing network request behavior.

## Acceptance

- no native-app code constructs or mutates a keyring entry for the Agent key;
- startup reads the saved key through `SecretStore`;
- save persists through `SecretStore` before activation;
- forget deletes through `SecretStore` and deactivates the provider only on success;
- persistence failures are surfaced and do not leave the UI busy;
- focused automated tests pass.
