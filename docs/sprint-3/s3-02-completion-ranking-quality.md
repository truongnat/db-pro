# Sprint 3 - S3-02 Completion Ranking Quality

## Goal

Improve SQL suggestion relevance with intent-aware ranking (keyword/object/column) and recency-aware boosting.

## Delivered

- Added dedicated ranking module:
  - `src/features/sql-editor/completion-ranking.ts`
- Added persistent recency tracker:
  - `src/features/sql-editor/completion-recency.ts`
  - storage key: `dbpro.sql.completionRecency.v1`
- Updated completion resolver to use intent + ranking:
  - `src/features/sql-editor/completions.ts`
- Added completion pick tracking from editor updates:
  - `src/features/sql-editor/SqlCodeEditor.tsx`

## Ranking Strategy

- Prefix quality (exact > startsWith > contains).
- Intent weighting by SQL context:
  - object-focused near `FROM/JOIN/UPDATE/INTO/...`
  - column-focused near `SELECT/WHERE/ON/SET/...` and scoped references
  - keyword-focused in early statement context
- Existing completion `boost` is preserved.
- Recency boost is applied from locally picked suggestions with freshness decay.

## Acceptance Examples

1. Type `se` at start of query:
   - `SELECT` should stay near top.
2. Type `SELECT * FROM ` and trigger completion:
   - table/view suggestions should rank above generic keywords.
3. Type `SELECT customers.`:
   - column suggestions from `customers` scope should rank highest.
4. Pick `customers` or `SELECT` repeatedly:
   - next suggestion list should prioritize recently picked item.

## Validation

- `npm run -s build` => PASS
- `cargo check --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo test --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check` => PASS (`checked=27 errors=0 warnings=0`)

Validated on 2026-03-06.
