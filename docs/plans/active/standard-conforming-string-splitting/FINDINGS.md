# Driver-aware SQL String Splitting — Findings

## P1 — PostgreSQL single-statement gate bypass via backslash quote

**Evidence:** Issue #477 reports that `skip_quoted` unconditionally consumed `\\` plus the next byte in every `'…'` string. With PostgreSQL's default `standard_conforming_strings=on`, the payload `SELECT 'x\\'; DROP TABLE important; --` contains a real statement boundary after the first literal, but the baseline splitter considered it one statement. `reject_multi_statement` could therefore permit a destructive second statement.

**Decision:** Select quoting behavior from `DriverType`. PostgreSQL standard strings do not treat backslash as an escape; `E'…'` does. MySQL's default dialect keeps backslash escapes in both quote styles. SQLite and SQL Server use standard quoted-string behavior. Keep PostgreSQL-standard semantics as the default where no driver is available.

**Disposition:** Fixed in this branch. Regression tests assert both the split result and rejection by the single-statement policy. No live DB was used as evidence.

## Limitations

- MySQL `NO_BACKSLASH_ESCAPES` and PostgreSQL `standard_conforming_strings=off` session overrides are not introspected; documented behavior uses the respective common defaults.
- Live provider behavior and independent review remain pending.
