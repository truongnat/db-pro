use crate::UiTableColumn;

/// Why a column cannot be written from the data editor.
///
/// Every message is user-facing: it is shown in the editor, in the read-only
/// context-menu entry, and as the insert error, so a blocked write always says
/// what is blocked rather than failing silently or at the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnWriteBlock {
    Binary,
    Generated,
}

impl ColumnWriteBlock {
    pub(crate) fn reason(self) -> &'static str {
        match self {
            Self::Binary => "Binary values are read-only until a binary editor is available",
            Self::Generated => "Generated columns are computed by the database and cannot be written",
        }
    }
}

/// Whether a column accepts a value from the editor.
///
/// The policy is deliberately value-class based, not type-name whitelist based:
/// every class the editor can parse and round-trip losslessly stays writable, and
/// the classes it cannot serialize safely are refused here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColumnWritePolicy {
    block: Option<ColumnWriteBlock>,
}

impl ColumnWritePolicy {
    pub(crate) fn write_block(&self) -> Option<ColumnWriteBlock> {
        self.block
    }

    pub(crate) fn is_writable(&self) -> bool {
        self.block.is_none()
    }

    /// Resolve the policy for one column.
    ///
    /// `BYTEA`/`BLOB` have no editor that can preserve the bytes, and a generated
    /// column rejects every value the database did not compute itself. `identity`,
    /// array, enum, domain, interval, inet and the temporal classes stay writable:
    /// the insert/update parsers validate them, and the parameter binder sends them
    /// as typed values rather than through a lossy conversion.
    pub(crate) fn read(column: &UiTableColumn) -> Self {
        let normalized = column.data_type.to_ascii_lowercase();
        let block = if is_binary_type(&normalized) {
            Some(ColumnWriteBlock::Binary)
        } else if column.is_generated {
            Some(ColumnWriteBlock::Generated)
        } else {
            None
        };
        Self { block }
    }
}

fn is_binary_type(normalized_type: &str) -> bool {
    normalized_type.contains("bytea")
        || normalized_type.contains("blob")
        || normalized_type.contains("binary")
        || normalized_type.contains("varbinary")
}

#[cfg(test)]
mod tests {
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
        assert!(ColumnWriteBlock::Binary.reason().contains("binary editor"));
        assert!(ColumnWriteBlock::Generated
            .reason()
            .contains("computed by the database"));
    }
}
