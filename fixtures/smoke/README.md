# Packaged-Runtime Smoke Fixtures

Deterministic database fixtures for verifying the packaged DB Pro app's runtime behavior after `FINAL_RC_SHA`.

## Directory layout

```
fixtures/smoke/
├── postgres/
│   ├── 001_smoke_schema.sql    # DDL: 10 tables, 2 views, 8 indexes, 1 trigger, 1 function, 1 enum, 1 sequence
│   ├── 002_smoke_seed.sql      # Seed: 44 rows across all tables
│   └── 003_smoke_teardown.sql  # DROP all smoke objects
├── sqlite/
│   └── smoke_fixture.sql       # Single-file schema + seed (10 tables, 2 views, 6 indexes, 1 trigger)
├── large-er/
│   ├── generate-large-er.js    # Deterministic generator for >200 tables
│   └── large_er_fixture.sql    # Pre-generated 250-table fixture
└── verify-smoke.sh             # Verification script (object counts + row counts)
```

## Quick start

### PostgreSQL

```bash
# Setup
psql -U dbpro -d dbpro_fixture -f fixtures/smoke/postgres/001_smoke_schema.sql
psql -U dbpro -d dbpro_fixture -f fixtures/smoke/postgres/002_smoke_seed.sql

# Verify
./fixtures/smoke/verify-smoke.sh postgres postgres://dbpro:dbpro_test@localhost:5432/dbpro_fixture

# Teardown
psql -U dbpro -d dbpro_fixture -f fixtures/smoke/postgres/003_smoke_teardown.sql
```

### SQLite

```bash
# Setup
sqlite3 fixtures/smoke/sqlite/smoke.db < fixtures/smoke/sqlite/smoke_fixture.sql

# Verify
./fixtures/smoke/verify-smoke.sh sqlite fixtures/smoke/sqlite/smoke.db

# Teardown
rm fixtures/smoke/sqlite/smoke.db
```

### Large ER (250 tables)

```bash
# Generate (or use pre-generated file)
node fixtures/smoke/large-er/generate-large-er.js --tables 250 --output fixtures/smoke/large-er/large_er_fixture.sql

# Load into PostgreSQL
psql -U dbpro -d dbpro_fixture -f fixtures/smoke/large-er/large_er_fixture.sql

# Teardown
psql -U dbpro -d dbpro_fixture -c "DROP TABLE IF EXISTS smoke_large_t0000 CASCADE;"
```

## Design principles

- **Deterministic**: repeated setup produces identical object names and row counts.
- **No credentials committed**: all connection parameters are environment-specific.
- **Provider-parity**: same logical schema across PostgreSQL and SQLite (with type adaptations).
- **Smoke-oriented**: covers every introspection path (tables, views, indexes, FKs, triggers, no-PK, empty, unicode, binary).
- **Large-schema path**: >200 tables triggers the search-first flow (Gate 4 Slice B).

## Coverage matrix

| Feature                    | PG smoke | SQLite smoke | Large ER |
|----------------------------|----------|--------------|----------|
| Tables with PK             | ✓        | ✓            | ✓ (250)  |
| Tables without PK          | ✓        | ✓            | —        |
| Composite PK               | ✓        | ✓            | —        |
| Empty table                | ✓        | ✓            | —        |
| Views                      | ✓ (2)    | ✓ (2)        | ✓ (5)    |
| Indexes (B-tree)           | ✓        | ✓            | —        |
| Indexes (GIN/expression)   | ✓        | —            | —        |
| FK chain                   | ✓        | ✓            | ✓ (249)  |
| Enum type                  | ✓        | — (CHECK)    | —        |
| JSON/JSONB                 | ✓        | ✓ (text)     | —        |
| UUID                       | ✓        | ✓ (text)     | —        |
| BYTEA/BLOB                 | ✓        | ✓            | —        |
| Generated column           | ✓        | —            | —        |
| Trigger                    | ✓        | ✓            | —        |
| Function                   | ✓        | —            | —        |
| Sequence                   | ✓        | —            | —        |
| Unicode identifiers        | ✓        | ✓            | —        |
| Network types (inet/cidr)  | ✓        | ✓ (text)     | —        |
| CHECK constraint           | —        | ✓            | —        |
| Hub FK pattern             | —        | —            | ✓ (25)   |
