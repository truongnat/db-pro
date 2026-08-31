# FINDINGS — PostgreSQL Non-Basic Provider Type Mapping (QA-P1-14)

## QA-P1-14 Details

- **Severity**: P1 (Provider-specific correctness / query result handling defect)
- **Impact**: Queries returning NUMERIC, DECIMAL, MONEY, ARRAY, TIMETZ, or INTERVAL columns in PostgreSQL could fail to decode rows or corrupt UTF-8 text representation fallback.
- **Evidence**: `query_mapper.rs` only handled basic scalar types explicitly; fallback to `String` decoding failed for complex types and binary protocol payloads.
