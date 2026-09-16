//! Query document helpers: atomic export I/O, diagnostics dedupe, and SQL formatting.
use super::*;
use crate::editor::{Diagnostic, EditorSnapshot, SelectionRange, SqlDialect};

/// Distinguishes concurrent temp files of one process (see [`write_file_atomically`]).
static ATOMIC_WRITE_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Write `contents` to `path`, publishing the file in one step (#244, E-1).
///
/// `std::fs::write(path, …)` streams straight at the destination, so a failure part-way through —
/// disk full, I/O error, volume removed — leaves a **truncated file that looks like a complete
/// export**. This writes a sibling temp file first (same directory, so the rename stays on one
/// filesystem), removes it if anything fails, and only then renames it over the destination: the
/// destination either keeps its previous content or holds the complete new bytes, never a prefix of
/// them.
pub(crate) fn write_file_atomically(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;

    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => std::path::Path::new("."),
    };
    let Some(file_name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "export path has no file name",
        ));
    };
    let temp_path = parent.join(format!(
        ".{}.tmp.{}.{}",
        file_name.to_string_lossy(),
        std::process::id(),
        ATOMIC_WRITE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));

    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp_path, path)
    })();

    if result.is_err() {
        // Best effort: a failed export must not leave its scratch file behind either.
        let _ = std::fs::remove_file(&temp_path);
    }
    result
}

/// Collapse `text` to at most `max_chars` visible characters, ending in an ellipsis.
///
/// Used where a label has a fixed width contract (query header selectors, combo buttons)
/// so a 60+ character connection or schema name cannot push the primary actions off-screen.
/// Counting is by `char`, so Vietnamese/Japanese names elide the same as ASCII.
pub(crate) fn elide_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut elided: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    elided.push('…');
    elided
}

pub(crate) fn deduplicate_messages(messages: Vec<String>) -> Vec<String> {
    let mut unique = Vec::with_capacity(messages.len());
    for message in messages {
        if !unique.iter().any(|existing| existing == &message) {
            unique.push(message);
        }
    }
    unique
}

pub(crate) fn deduplicate_diagnostics(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut unique = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        let duplicate = unique.iter().any(|existing: &Diagnostic| {
            existing.source == diagnostic.source
                && existing.message == diagnostic.message
                && ranges_overlap(existing.range, diagnostic.range)
        });
        if !duplicate {
            unique.push(diagnostic);
        }
    }
    unique
}

pub(crate) fn database_error_diagnostic(
    message: &str,
    executed_sql: &str,
    executed_range: (usize, usize),
    position: Option<usize>,
    code: Option<&str>,
) -> Option<Diagnostic> {
    if executed_sql.is_empty() || executed_range.0 >= executed_range.1 {
        return None;
    }
    let (start, end) = position
        .filter(|position| *position > 0)
        .map(|position| char_position_to_byte_offset(executed_sql, position))
        .filter(|start| *start < executed_sql.len())
        .and_then(|start| {
            executed_sql[start..]
                .chars()
                .next()
                .map(|character| (start, start + character.len_utf8()))
        })
        .unwrap_or((0, executed_sql.len()));
    let local_limit = executed_range.1 - executed_range.0;
    let document_start = executed_range.0 + start.min(local_limit);
    let document_end = (executed_range.0 + end)
        .min(executed_range.1)
        .max((document_start + 1).min(executed_range.1));
    Some(match code {
        Some(code) => Diagnostic::database_with_code((document_start, document_end), message, code),
        None => Diagnostic::database((document_start, document_end), message),
    })
}

fn char_position_to_byte_offset(sql: &str, position: usize) -> usize {
    let index = position.saturating_sub(1);
    sql.char_indices()
        .nth(index)
        .map(|(offset, _)| offset)
        .unwrap_or(sql.len())
}

fn ranges_overlap(left: (usize, usize), right: (usize, usize)) -> bool {
    left.0 < right.1 && right.0 < left.1
}

pub(crate) fn format_query_document(doc: &mut QueryDocument, dialect: SqlDialect) {
    let original_selection = doc.selection;
    let had_selection = !original_selection.is_empty();
    let (start, end) = original_selection.normalized();
    let original_text = doc.buffer.slice(start, end).to_owned();
    let formatted_text = crate::query::sql_format::format_sql_for_dialect(&original_text, dialect);
    if formatted_text == original_text {
        return;
    }

    let cursor_after = if had_selection {
        if original_selection.active >= original_selection.anchor {
            start + formatted_text.len()
        } else {
            start
        }
    } else {
        start + doc.cursor.offset.saturating_sub(start).min(formatted_text.len())
    };
    let anchor_after = if had_selection {
        if original_selection.anchor <= original_selection.active {
            start
        } else {
            start + formatted_text.len()
        }
    } else {
        cursor_after
    };

    doc.buffer.replace_with_snapshot(
        start,
        end,
        &formatted_text,
        EditorSnapshot {
            cursor_offset: doc.cursor.offset,
            anchor_offset: original_selection.anchor,
        },
        EditorSnapshot {
            cursor_offset: cursor_after,
            anchor_offset: anchor_after,
        },
    );
    doc.cursor.set_offset(&doc.buffer, cursor_after);
    doc.selection = SelectionRange::new(anchor_after, cursor_after);
    doc.completion.clear();
    doc.invalidate_prediction();
    doc.reanalyze(dialect);
    doc.search.update_matches(doc.buffer.text());
    doc.dirty = true;
}

pub(crate) fn prediction_replacement_range(
    buffer: &crate::editor::buffer::TextBuffer,
    anchor: usize,
    manual: bool,
) -> (usize, usize) {
    if !manual {
        return (anchor, anchor);
    }
    let before = buffer.slice(0, anchor);
    let start = before
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_')
        .map_or(0, |(offset, ch)| offset + ch.len_utf8());
    (start, anchor)
}

#[cfg(test)]
mod elide_tests {
    use super::elide_chars;

    #[test]
    fn short_text_is_unchanged() {
        assert_eq!(elide_chars("public", 24), "public");
        assert_eq!(elide_chars("", 8), "");
    }

    #[test]
    fn long_text_is_truncated_with_ellipsis() {
        let long = "a".repeat(80);
        let elided = elide_chars(&long, 24);
        assert_eq!(elided.chars().count(), 24);
        assert!(elided.ends_with('…'));
    }

    #[test]
    fn unicode_characters_count_individually() {
        let vietnamese = "Kết nối cơ sở dữ liệu chính của phòng kế toán".repeat(3);
        let elided = elide_chars(&vietnamese, 12);
        assert_eq!(elided.chars().count(), 12);
        assert!(elided.ends_with('…'));

        let japanese = "データベース接続設定".repeat(4);
        let elided = elide_chars(&japanese, 8);
        assert_eq!(elided.chars().count(), 8);
        assert!(elided.ends_with('…'));
    }
}
