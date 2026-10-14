//! Tests for `../mod.rs` (moved out of the source file).
use super::*;

fn column(name: &str, data_type: &str) -> UiTableColumn {
    UiTableColumn {
        name: name.to_owned(),
        data_type: data_type.to_owned(),
        ordinal: 0,
        nullable: true,
        default: None,
        is_primary_key: false,
        is_unique: false,
        is_identity: false,
        is_generated: false,
        collation: None,
        enum_labels: Vec::new(),
    }
}

#[test]
fn binary_and_generated_columns_are_read_only() {
    let cases = [
        ("payload", "bytea", false, false, Some(ColumnWriteBlock::Binary)),
        ("thumbnail", "BLOB", false, false, Some(ColumnWriteBlock::Binary)),
        (
            "blob_data",
            "binary varying",
            false,
            false,
            Some(ColumnWriteBlock::Binary),
        ),
        ("total", "numeric", false, true, Some(ColumnWriteBlock::Generated)),
        ("search", "tsvector", false, true, Some(ColumnWriteBlock::Generated)),
    ];
    for (name, data_type, is_identity, is_generated, expected) in cases {
        let mut column = column(name, data_type);
        column.is_identity = is_identity;
        column.is_generated = is_generated;
        let policy = ColumnWritePolicy::read(&column);
        assert_eq!(
            policy.write_block(),
            expected,
            "{name} {data_type} (identity={is_identity}, generated={is_generated})"
        );
    }
}

#[test]
fn every_class_the_editor_can_round_trip_stays_writable() {
    let cases = [
        ("id", "bigserial", true),
        ("id", "integer", true),
        ("small", "smallint", true),
        ("amount", "numeric(20,6)", true),
        ("ratio", "double precision", true),
        ("active", "boolean", true),
        ("note", "text", true),
        ("code", "character varying(12)", true),
        ("uid", "uuid", true),
        ("created", "timestamp without time zone", true),
        ("created_utc", "timestamp with time zone", true),
        ("day", "date", true),
        ("clock", "time without time zone", true),
        ("zoned", "time with time zone", true),
        ("span", "interval", true),
        ("address", "inet", true),
        ("signed_at", "timestamptz", true),
        ("tags", "text[]", true),
        ("status", "order_status", true),
        ("email", "citext", true),
        ("payload", "jsonb", true),
    ];
    for (name, data_type, expected_writable) in cases {
        let policy = ColumnWritePolicy::read(&column(name, data_type));
        assert_eq!(policy.is_writable(), expected_writable, "{name} {data_type}");
    }
}

#[test]
fn identity_columns_stay_writable() {
    let mut id = column("id", "bigint");
    id.is_identity = true;
    let policy = ColumnWritePolicy::read(&id);
    assert!(policy.is_writable(), "identity columns accept explicit values");
}

#[test]
fn blocked_columns_explain_themselves() {
    assert!(ColumnWriteBlock::Binary.reason().contains("Inspect"));
    assert!(ColumnWriteBlock::Generated
        .reason()
        .contains("computed by the database"));
}
