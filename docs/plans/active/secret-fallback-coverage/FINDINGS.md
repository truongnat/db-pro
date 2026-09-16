# Findings

## P2 — Crypto and fallback modules had no direct tests

At baseline `11665f6ac756636ebac7fbe36c11691941bc9b80`,
`crates/infrastructure/src/secret/encryption.rs` and `fallback.rs` expose the
development encrypted-file path but contain no `#[test]` coverage. The parent
secret lifecycle audit explicitly listed wrong-key, corruption, restart and
delete behavior as remaining gaps.

No production behavior change is required; the smallest fix is focused unit
coverage at the existing module seams.
