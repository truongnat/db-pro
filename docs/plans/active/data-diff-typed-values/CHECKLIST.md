# Checklist — #457 / #485

Baseline: `76b120724a7542de8b587a002811ddacad28156a`. State: REVIEW; parent execution evidence recorded in VERIFICATION.md.

## Implemented
- [x] Trace service/API/worker/UI consumers; preserve public DTO field types.
- [x] Borrow typed row values; use structural typed key tuples, never sentinels or delimiters as identity.
- [x] Compare actual NULL separately from text `∅`; expose NULL as DTO None.
- [x] Generate null-aware key predicates and typed INSERT/UPDATE values.
- [x] Add adjacent regression cases for NULL/text changes, distinct NULL/text keys, null predicates, sentinel INSERT/UPDATE, composite separator collisions, intended WHERE boundaries and precision/quoting.
- [x] Preserve basic diff and genuine duplicate-key tests.
- [x] Supply real SQLite and independent PostgreSQL public-service round-trip probes; execute each preview, assert one affected row, recompare final state.
- [x] Document inherited binary/MySQL literal limitations explicitly.

## Integration owner gates
- [x] Focused core data-diff tests: 11 passed / 0 failed / 0 ignored after final parameter-count correction.
- [x] SQLite independent public-service smoke PASS.
- [x] PostgreSQL independent public-service smoke PASS; permanent ignored provider case explicitly executed (1/0/0).
- [x] Combined formatting/check/workspace tests executed; independent reviewer ACCEPT; inherited lint limits reported separately.
- [ ] Publish exact commit and close #457/#485 with individual evidence.
