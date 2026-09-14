# Provider-value contract document — A3 complex/fallback slice (#53)

- Session: `v01-runtime` provider follow-up, 2026-09-15
- Tree: `main @ 9424ff8` (clean at the start of the change); fix commit recorded in `LEDGER.md`
- Issue: **#53** ([Gate 5][A3] Define INTERVAL/network/enum/domain/array/custom-type fallback
  contract) — parent workstream **#22**, parent gate **#21**
- Deliverable: **`docs/release/provider-value-contract.md`** (new; the single in-tree contract the
  A1–A4 slices were locked against)

## 1. What was missing

Re-verified on the current tree: `grep -rli fallback docs/` matched 60 files and none of them was a
per-class contract. The nearest artefacts each covered one side or one slice:

- `docs/notes/PRODUCT_CAPABILITY_MATRIX.md:94` — a provider *capability* row ("Types / Enums / Domains
  | MISSING (only `format_type()` strings)"), not a read contract;
- `docs/release/provider-capability-matrix.md:65,69` — capability/qualification status, not a read
  or fallback contract;
- `docs/release/evidence/v01-runtime/providers/32-type-aware-editability-policy.md:47-48` — the *write*
  side, and it explicitly disclaimed the read side ("the read side stays with #52/#53/#54").

So "enum/domain/array/custom fall back to raw text with no recorded contract" was accurate.

## 2. What the document records

`docs/release/provider-value-contract.md`, verified against code and tests rather than written from
memory (every line pointer in it was re-opened before commit; the two that had drifted —
`bind_params_fails_on_invalid_uuid`/`…_datetime`, now `:703`/`:710` — were corrected in the same change):

| Section | Content |
|---|---|
| §1 | the six non-negotiable rules (precision strings, no invented timezone, safe readable fallback, type-aware editability, representation decided before consumers, nothing locked on a claim) with the code pointer for each |
| §2 | the **class matrix**: 20 provider classes → domain value → canonical text/shape → UI class → editability → the test that proves the row. Covers INTERVAL, INET/CIDR, ENUM, DOMAIN (through its base type), arrays and generated columns next to the A1/A2 classes |
| §2.1 | the **fallback contract**: text format → canonical text; binary format → text only when the type is an enum or a character type; anything else → byte-exact `Bytes` (with the pre-fix mojibake/failed-query behaviour named as the reason) |
| §2.2 | what is explicitly **not** in the v0.1 contract: array element parsing, dedicated `timestamp`/`timetz`/`timestamptz` variants (#52), ranges/composites/geometric/`tsvector` as structured values, temporal mutation parameters (binders fail explicitly instead of coercing) |
| §3 | the serialization contract: the shipping in-process channel, the legacy Tauri DTO, both checked-in fixtures, and why there is no TypeScript mirror |
| §4 | what would **falsify** the document — the exact test that goes red for each kind of drift |
| §5 | the evidence index per slice |

## 3. Verification

Documentation-only change; no product code touched, and the gates were re-run on the unchanged code to
record the state the document describes:

- `cargo fmt --all -- --check` exit 0, `cargo check --workspace` exit 0,
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo test --workspace`
  **863 passed / 0 failed / 25 ignored**, `cargo build --release --locked -p db-pro-native` exit 0,
  `bash .skills/perf-audit/scripts/perf-scan.sh` `Status: PASS` (4/0/0). Delta vs the 856/0/21 session
  baseline: **+7 passed, +4 ignored, 0 failed** (all from #54/#57/#58/#59 in this session).
- Pointer check: every `file:line` cited in §2/§3 was re-opened on this tree (`grep -n` for the symbol
  names) before the commit; the two stale pointers found were fixed in the same change.

## 4. Not claimed here

- The A2 dedicated temporal variants (#52) and the A4 TypeScript mirror are still not done; the document
  says so in §2.2/§3 instead of implying a complete A1–A4.
- The document does not close #22 (Workstream A) or #21 (parent gate): both require #52's variants, and
  #21 requires the whole workstream set.
