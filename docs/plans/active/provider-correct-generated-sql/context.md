# Context — #441/#442

Source baseline: `76b120724a7542de8b587a002811ddacad28156a`.

Reuse existing dialect, no ports change. Repair newly consumed MySQL referenced-schema joins and SQLServer action spellings; declare MySQL supporting indexes before FKs. PostgreSQL typed binding prerequisite tracked separately as #533.

Detailed source pointers/alternatives and inherited limits: PLAN.md, FINDINGS.md.
