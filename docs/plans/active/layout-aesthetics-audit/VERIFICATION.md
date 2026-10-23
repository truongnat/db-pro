# Layout Aesthetics Audit — Verification

**Status: in progress.** Captures exist; scoring and the independent pass do not. Nothing here is a
finished audit.

## Source reference

```text
code under test  457cd645  (viewport/picker/chrome fixes)
plus             1b94044e  Explorer filter popover inset (audit finding F7a)
plus             working-tree helper refactor: popover_content_frame in
                 crates/ui/src/components/common/frames.rs
matrix captured  c245183e  manifest + reproducible runner
```

## What exists

| Item | State |
|---|---|
| Rubric with 0–4 anchors, eight dimensions | written — `RUBRIC.md` |
| Capture matrix | **60 cells captured** (10 surfaces × 3 widths × 2 themes) |
| Cells retained in the repository | 6 (Explorer, light+dark × 3 widths) — the rest are reproduced by `evidence/capture_matrix.py` |
| Cells visually inspected | **6 of 60** (Explorer only) |
| Cells scored | 0 |
| Independent scoring pass | not started |
| Real state captures (empty/loading/error) | not captured |
| Sub-surfaces (`table` structure, Settings → Appearance, New Connection error) | not captured |

The manifest records `retained` and `inspected` per cell. A SHA-256 proves file identity, not that
anyone looked at the image, so only cells with `inspected: true` count as reviewed.

## Capture run facts

- Runner: `evidence/capture_matrix.py`, executed against the capture binary built from this tree.
- Every cell rendered at its requested width; 1440/1920 requests render at **838 logical height** on
  this host (desktop clamp), so the matrix is not full-height verification.
- Cells whose process exits `-6` are the macOS TouchBar teardown crash that happens **after** the PNG
  is written; they are recorded as crashed, not as clean passes. The Explorer cells cover both
  outcomes: `-6` on light 1280 / dark 1440 / both 1920, exit 0 on the others, all with a PNG.

## Gates executed for the popover fix (F7a)

| Gate | Result |
|---|---|
| `cargo +nightly fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test -p db-pro-app` | 681 passed / 0 failed |
| `cargo test --workspace` | 2,272 passed / 0 failed / 65 ignored |
| `cargo build --release --locked -p db-pro-native` | exit 0 |
| `scripts/check-ui-architecture.sh` | PASS |

## Rules for this audit

- A score requires a cited image **and** the measured numbers behind it.
- Missing cells are reported as missing; they are never inferred from a neighbouring surface.
- Scores are expert judgement with anchors, not user testing.
- The audit stays `PLANNING` until scoring and the independent pass are done.

## Results

None yet.
