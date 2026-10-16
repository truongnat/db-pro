# Requirements

- [ ] R1 #499: valid bare/qualified zero-argument function references create a working PG event trigger; reject malformed references.
- [ ] R2 #508: exact requested PG relation dependency identity, unaffected by search_path/wildcards/quotes/decoys.
- [ ] R3 #452: MySQL credentials and database names containing URL-reserved characters connect/test correctly without changing TLS policy.
- [ ] R4 #486: verify BOM/non-BOM delimited import header/data semantics; repair only a reproduced failure.

Owner approved bounded four-ticket continuation. Out of scope: other PR ownership, UI redesign, unrelated global lint debt, SQLServer runtime.
