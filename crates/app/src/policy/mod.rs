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
            Self::Binary => "Binary values are read-only in the grid — use Inspect to preview/export",
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
#[path = "tests/mod_tests.rs"]
mod tests;
