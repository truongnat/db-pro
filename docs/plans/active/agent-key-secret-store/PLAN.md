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
