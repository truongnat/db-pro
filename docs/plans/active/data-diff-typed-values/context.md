# Context — #457/#485

Source baseline: `76b120724a7542de8b587a002811ddacad28156a`.

Borrow typed comparison values/keys. INSERT/predicates use typed cells; UPDATE consumes exact nullable scalar change values (None vs Some), never sentinel decoding. Preserve DTO types and existing binary/MySQL limits.

Detailed source pointers/alternatives and inherited limits: PLAN.md, FINDINGS.md.
