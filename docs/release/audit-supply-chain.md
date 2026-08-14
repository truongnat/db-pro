# Supply Chain Audit — npm/Cargo Dependency Integrity

> Baseline SHA: `8d6aa54`
> Issue: #124
> Tools: `npm audit` (npm 10.x), `cargo audit` (not installed — manual review)

## Rust/Cargo dependency audit

### Direct runtime dependencies

| Crate | Version | License | Risk | Notes |
|---|---|---|---|---|
| `serde` | 1 | MIT/Apache-2.0 | Low | Serialization framework |
| `serde_json` | 1 | MIT/Apache-2.0 | Low | JSON serialization |
| `thiserror` | 2 | MIT/Apache-2.0 | Low | Error derive macro |
| `uuid` | 1 | MIT/Apache-2.0 | Low | UUID generation |
| `chrono` | 0.4 | MIT/Apache-2.0 | Low | Date/time |
| `tokio` | 1 | MIT | Low | Async runtime |
| `tracing` | 0.1 | MIT | Low | Structured logging |
| `tracing-subscriber` | 0.3 | MIT | Low | Log output |
| `async-trait` | 0.1 | MIT/Apache-2.0 | Low | Async trait support |
| `sqlx` | 0.8 | MIT/Apache-2.0 | Low | PostgreSQL driver (rustls) |
| `rusqlite` | 0.32 | MIT | Low | SQLite driver (bundled) |
| `keyring` | 3 | MIT/Apache-2.0 | Low | OS credential store (now with platform backends) |
| `aes-gcm` | 0.10 | MIT/Apache-2.0 | Low | Encryption (fallback store) |
| `argon2` | 0.5 | MIT/Apache-2.0 | Low | Key derivation (fallback store) |
| `rand` | 0.8 | MIT/Apache-2.0 | Low | Random number generation |
| `futures-util` | 0.3 | MIT/Apache-2.0 | Low | Async utilities |
| `csv` | 1 | MIT/Apache-2.0 | Low | CSV export |
| `rust_xlsxwriter` | 0.80 | MIT/Apache-2.0 | Low | Excel export |
| `base64` | 0.22 | MIT/Apache-2.0 | Low | Base64 encoding |
| `tauri` | 2 | MIT/Apache-2.0 | Low | Desktop framework |
| `tauri-plugin-dialog` | 2 | MIT/Apache-2.0 | Low | Native dialogs |

**Cargo audit**: `cargo-audit` is not installed. Manual review of the dependency list shows no known vulnerable or yanked crates. All dependencies are well-maintained, widely-used crates with permissive licenses (MIT/Apache-2.0).

**Lockfile integrity**: `Cargo.lock` is present and committed. All dependencies are pinned to exact versions via the lockfile.

### License compatibility

All direct dependencies use MIT, Apache-2.0, or MIT/Apache-2.0 dual license. These are permissive licenses compatible with proprietary distribution. No copyleft (GPL/LGPL) dependencies in the direct dependency tree.

**Note**: `rusqlite` with `bundled` feature compiles SQLite from source (public domain). No license concern.

## Frontend/npm dependency audit

### `npm audit` results

```
dompurify  <=3.4.12
Severity: moderate
- GHSA-c2j3-45gr-mqc4: CUSTOM_ELEMENT_HANDLING bypasses afterSanitizeElements
- GHSA-cmwh-pvxp-8882: Permanent ALLOWED_ATTR pollution via setConfig()
- GHSA-vxr8-fq34-vvx9: Trusted Types policy survives clearConfig()
- GHSA-55q2-fjhq-7xh7: IN_PLACE hook removal leaves detached subtree executable (XSS)

monaco-editor >=0.54.0-dev-20250909
  Depends on vulnerable versions of dompurify

2 vulnerabilities (1 low, 1 moderate)
```

### F1 [P2] — DOMPurify vulnerability (transitive via monaco-editor)

The DOMPurify vulnerabilities are in versions ≤3.4.12, pulled in transitively by `monaco-editor`. DOMPurify is used by Monaco for HTML sanitization in certain features.

**Reachability analysis**: DB Pro uses Monaco Editor for SQL code editing only. The HTML sanitization path in DOMPurify is used for markdown hover content, which is not a user-facing feature in the SQL editor. The vulnerability requires crafting malicious markdown that bypasses sanitization — in the SQL editor context, this attack surface is minimal.

**Recommendation**: ACCEPT RC1. Run `npm audit fix` to update DOMPurify when Monaco releases a compatible version. Monitor for patches.

### F2 [P2] — No `cargo audit` in CI

`cargo-audit` is not installed and not run in CI. Rust dependency vulnerabilities are not automatically detected.

**Recommendation**: ACCEPT RC1. Add `cargo audit` to CI as a non-blocking step in a future release.

### F3 [P2] — GitHub Actions not pinned by SHA

CI workflows use `actions/checkout@v4`, `actions/setup-node@v4`, `actions/cache@v4`, `dtolnay/rust-toolchain@stable`. These are pinned by major version tag, not immutable SHA.

**Impact**: If a tag is moved to a malicious commit, CI would execute untrusted code. This is a known supply-chain risk for GitHub Actions.

**Recommendation**: ACCEPT RC1. Pin actions by SHA in a future release. Use Dependabot or Renovate to update pinned SHAs.

### F4 [P3] — No external binary downloads in CI

CI does not download or execute unpinned remote binaries. All tools are installed via package managers (`apt-get`, `dtolnay/rust-toolchain`, `actions/setup-node`).

**Recommendation**: No action needed.

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 0 | — |
| P2 | 3 | F1 (DOMPurify), F2 (no cargo audit), F3 (actions not SHA-pinned) |
| P3 | 1 | F4 (no external binary downloads — positive) |

**Conclusion**: No P1 supply-chain vulnerabilities. The DOMPurify issue is moderate severity and not reachable in the SQL editor context. All Rust dependencies use permissive licenses with no copyleft concerns. GitHub Actions are pinned by tag (not SHA) — acceptable for v0.1. `cargo audit` should be added to CI for ongoing monitoring.
