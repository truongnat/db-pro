# Findings: QA-P1-14 PostgreSQL Type Mapping

## Defect Summary
- **Defect ID**: `QA-P1-14`
- **Severity**: P1
- **File**: `crates/infrastructure/src/postgres/query_mapper.rs`
- **Source Confirmed**: Yes

## Detailed Analysis
The existing `map_row` implementation matches on column type names:
```rust
match col.data_type.as_str() {
    "BOOL" => ...
    "INT2" | "INT4" | "INT8" => ...
    "FLOAT4" | "FLOAT8" => ...
    "UUID" => ...
    "TIMESTAMPTZ" | "TIMESTAMP" => ...
    "JSON" | "JSONB" => ...
    "BYTEA" => ...
    _ => {
        let v: String = row.try_get(i).map_err(crate::error::from_sqlx)?;
        CellValue::Text(v)
    }
}
```

### Risk & Impact
1. `_ => row.try_get::<String>(i)` returns an error if `sqlx` does not support direct `String` conversion for unhandled OIDs or types.
2. An error on a single cell fails the entire `map_row` function and aborts the whole `QueryResult`.
3. Types like `NUMERIC`, `DECIMAL`, `DATE`, `TIME`, `TIMETZ`, `INTERVAL`, `INET`, `CIDR`, `MACADDR`, `MONEY`, `OID`, `BIT`, `VARBIT`, `VARCHAR`, `TEXT`, `CHAR`, `BPCHAR`, `NAME`, `CITEXT` need explicit handling or safe string fallback.

## Remediation Strategy
1. Handle common PostgreSQL text, numeric, network, and temporal type names explicitly.
2. For fallback or decoding failure, attempt to convert raw cell value to string or return `CellValue::Text("<unsupported type: DATA_TYPE>")` or string representation rather than returning a top-level `Err`.
