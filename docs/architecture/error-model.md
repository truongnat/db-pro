# Error Model

> Status: **Implemented** (P2-02)

## Source

- Domain taxonomy: `crates/core/src/domain/error.rs`
- Transport DTO: `crates/tauri-app/src/dto.rs` (`CommandError`)
- SQL mapping: `crates/infrastructure/src/error.rs`

## Error Taxonomy

```text
DbError
├── Connection
│   ├── AuthenticationFailed
│   ├── ConnectionRefused
│   ├── ConnectionTimeout
│   ├── ConnectionLost
│   ├── DatabaseNotFound
│   ├── SslError
│   └── ConnectionFailed (generic)
├── Query
│   ├── QuerySyntax
│   ├── PermissionDenied
│   ├── QueryTimeout { timeout_ms }
│   ├── QueryCancelled
│   └── QueryFailed (generic)
├── Schema
│   ├── SchemaFailed (generic)
│   └── Unsupported
├── Data
│   ├── DataFailed (generic)
│   ├── RowIdentityRequired
│   └── ConstraintViolation
├── ReadOnlyViolation
├── Validation
│   └── ValidationFailed
├── Io
│   └── IoFailed
├── Encryption
│   └── EncryptionFailed
└── Internal
    └── Internal
```

## Transport

Every error crossing the Tauri boundary becomes a `CommandError`:

```rust
pub struct CommandError {
    pub error: String,           // Machine-readable code (e.g. "DB_AUTH_FAILED")
    pub message: String,         // Human-readable description
    pub message_id: String,      // i18n key (e.g. "error.connection.auth")
    pub details: Option<String>, // Additional context
    pub retryable: bool,         // Whether retry might succeed
}
```

## Error Codes

| Code | Message ID | Retryable | Category |
|------|-----------|-----------|----------|
| `DB_AUTH_FAILED` | `error.connection.auth` | false | Connection |
| `DB_CONNECTION_REFUSED` | `error.connection.refused` | true | Connection |
| `DB_CONNECTION_TIMEOUT` | `error.connection.timeout` | true | Connection |
| `DB_CONNECTION_LOST` | `error.connection.lost` | true | Connection |
| `DB_NOT_FOUND` | `error.connection.db_not_found` | false | Connection |
| `DB_SSL_ERROR` | `error.connection.ssl` | false | Connection |
| `DB_CONNECTION_FAILED` | `error.connection.failed` | false | Connection |
| `QUERY_SYNTAX` | `error.query.syntax` | false | Query |
| `QUERY_PERMISSION_DENIED` | `error.query.permission` | false | Query |
| `QUERY_TIMEOUT` | `error.query.timeout` | true | Query |
| `QUERY_CANCELLED` | `error.query.cancelled` | false | Query |
| `QUERY_FAILED` | `error.query.failed` | false | Query |
| `SCHEMA_OPERATION_FAILED` | `error.schema.failed` | false | Schema |
| `SCHEMA_OPERATION_UNSUPPORTED` | `error.schema.unsupported` | false | Schema |
| `DATA_OPERATION_FAILED` | `error.data.failed` | false | Data |
| `ROW_IDENTITY_REQUIRED` | `error.data.row_identity` | false | Data |
| `CONSTRAINT_VIOLATION` | `error.data.constraint` | false | Data |
| `READ_ONLY_VIOLATION` | `error.safety.read_only` | false | Safety |
| `VALIDATION_FAILED` | `error.validation` | false | Validation |
| `IO_ERROR` | `error.io` | false | I/O |
| `ENCRYPTION_FAILED` | `error.encryption` | false | Encryption |
| `INTERNAL_ERROR` | `error.internal` | false | Internal |

## Rules

1. No `Err(format!(...))` at the Tauri command boundary for errors with semantic codes.
2. Frontend must not parse raw SQL error strings for logic decisions.
3. `retryable` is set based on error semantics, not guesswork.
4. SQLSTATE-aware mapping preserves PostgreSQL error categories.
