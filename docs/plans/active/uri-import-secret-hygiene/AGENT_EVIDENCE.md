# URI Import Secret Hygiene — Agent Evidence

- Issue: #479, confirmed that valid URI credentials were copied into dedicated connection fields while the source URI remained in a plaintext `Input` backed by `UiConnectionDraft.cloud_snippet`.
- Corrected issue claim: inspected code did not support draft serialization, backend propagation, or log exposure. Established risk is on-screen plaintext retention.
- Implementation: successful `apply_connection_snippet` now clears `cloud_snippet`; parsing failures leave the original input intact for correction. The password remains in the existing dedicated password field.
- Regression evidence: `cargo test -p db-pro-app mapper_` passes 3/3, including success clears and failure preserves input.
- UI/design review: no layout, type, or palette change. Existing compact connection form and `DbProTheme` remain unchanged. Runtime screenshots were produced for the New Connection dialog at 1280×800, 1440×900, 1920×1080, light and dark. Reviewed dark/light 1280×800 and dark 1920×1080 for centered, unclipped modal layout; all six files have expected dimensions. Runtime capture does not display a post-import fixture; behavior is asserted in tests.
- Build/format/diff gates: see `VERIFICATION.md`; native release and capture-feature builds pass.
