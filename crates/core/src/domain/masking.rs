//! Column masking / anonymization for safe sample export (#253).
//!
//! Preview-only by default. In-place destructive masking is out of scope.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaskRule {
    Redact,
    Hash,
    Fixed,
    PartialReveal,
    PreserveNull,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnMask {
    pub column: String,
    pub rule: MaskRule,
    /// Used by Fixed and as salt input for keyed Hash (never log the raw key).
    pub replacement: String,
    /// Characters to keep at start/end for PartialReveal.
    pub keep_prefix: usize,
    pub keep_suffix: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MaskingProfile {
    pub name: String,
    pub schema: String,
    pub table: String,
    pub columns: Vec<ColumnMask>,
    /// Deterministic keyed hashing for join-preserving IDs.
    pub keyed: bool,
    pub key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskingPreview {
    pub columns: Vec<String>,
    pub original: Vec<Vec<String>>,
    pub masked: Vec<Vec<String>>,
    pub message: String,
}

pub fn suggest_sensitive_columns(column_names: &[String]) -> Vec<String> {
    column_names
        .iter()
        .filter(|name| {
            let n = name.to_ascii_lowercase();
            n.contains("email")
                || n.contains("phone")
                || n.contains("ssn")
                || n.contains("password")
                || n.contains("secret")
                || n.contains("token")
                || n.contains("address")
                || n.contains("dob")
                || n.contains("birth")
        })
        .cloned()
        .collect()
}

/// Mask one cell.
///
/// `value` distinguishes a real SQL `NULL` (`None`) from present text
/// (`Some`, including `Some("")`, `Some("null")`, `Some("NULL")`). A SQL `NULL`
/// has no value to mask and renders as `"NULL"`; any present text is always run
/// through the configured rule, so a literal `"null"` can never escape masking
/// (#502).
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
pub fn mask_cell(value: Option<&str>, rule: &ColumnMask, keyed: bool, key_material: &str) -> String {
    let Some(value) = value else {
        return "NULL".into();
    };
    match rule.rule {
        MaskRule::PreserveNull => value.to_owned(),
        MaskRule::Redact => "***".into(),
        MaskRule::Fixed => {
            if rule.replacement.is_empty() {
                "[masked]".into()
            } else {
                rule.replacement.clone()
            }
        }
        MaskRule::Hash => {
            let mut hasher = DefaultHasher::new();
            if keyed {
                key_material.hash(&mut hasher);
            }
            value.hash(&mut hasher);
            format!("h{:016x}", hasher.finish())
        }
        MaskRule::PartialReveal => {
            let chars: Vec<char> = value.chars().collect();
            if chars.len() <= rule.keep_prefix + rule.keep_suffix {
                return "*".repeat(chars.len().max(1));
            }
            let prefix: String = chars.iter().take(rule.keep_prefix).collect();
            let suffix: String = chars.iter().rev().take(rule.keep_suffix).rev().collect();
            let middle = chars.len() - rule.keep_prefix - rule.keep_suffix;
            format!("{prefix}{}{suffix}", "*".repeat(middle))
        }
        MaskRule::Synthetic => {
            if value.parse::<i64>().is_ok() {
                "0".into()
            } else if value.contains('@') {
                "user@example.test".into()
            } else {
                "sample".into()
            }
        }
    }
}

/// Mask a row whose cells may be SQL `NULL` (`None`) or present text (`Some`).
///
/// An unmasked column renders a SQL `NULL` as `"NULL"` so the output cell type
/// stays `String`; a masked column is delegated to [`mask_cell`].
pub fn mask_row(
    headers: &[String],
    cells: &[Option<String>],
    profile: &MaskingProfile,
    key_material: &str,
) -> Vec<String> {
    cells
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let col = headers.get(i).map(String::as_str).unwrap_or("");
            if let Some(rule) = profile.columns.iter().find(|c| c.column == col) {
                mask_cell(cell.as_deref(), rule, profile.keyed, key_material)
            } else {
                render_cell(cell.as_deref())
            }
        })
        .collect()
}

pub fn preview_masking(
    headers: &[String],
    rows: &[Vec<Option<String>>],
    profile: &MaskingProfile,
    key_material: &str,
) -> MaskingPreview {
    let original = rows.iter().map(|row| render_cells(row)).collect::<Vec<_>>();
    let masked = rows
        .iter()
        .map(|row| mask_row(headers, row, profile, key_material))
        .collect::<Vec<_>>();
    let mut message = format!(
        "preview {} rows · {} mask rule(s) · keyed={}",
        rows.len(),
        profile.columns.len(),
        profile.keyed
    );
    let unmatched = unmatched_rule_columns(headers, profile);
    if !unmatched.is_empty() {
        // #503 — surface rules that match no column instead of passing cells
        // through unmasked with no feedback. The comparison is the exact,
        // case-sensitive one `mask_row` uses; quoted PostgreSQL identifiers are
        // case-sensitive, so a rule must not silently widen its match.
        message.push_str(&format!(
            " · WARNING: {} rule(s) matched no column (names are case-sensitive): {}",
            unmatched.len(),
            unmatched.join(", ")
        ));
    }
    MaskingPreview {
        columns: headers.to_vec(),
        original,
        masked,
        message,
    }
}

/// Column names configured in `profile` that match no header.
///
/// Uses the same exact, case-sensitive comparison `mask_row` applies, so a
/// non-empty result means those rules will not mask anything in this preview.
/// Case-insensitive matching is deliberately **not** used: quoted PostgreSQL
/// identifiers are case-sensitive, and silently matching `email` to `Email`
/// could mask the wrong column.
pub fn unmatched_rule_columns(headers: &[String], profile: &MaskingProfile) -> Vec<String> {
    profile
        .columns
        .iter()
        .filter(|rule| !headers.iter().any(|header| header == &rule.column))
        .map(|rule| rule.column.clone())
        .collect()
}

/// Render a nullable cell for display: SQL `NULL` becomes the literal `"NULL"`.
fn render_cell(cell: Option<&str>) -> String {
    cell.map(str::to_owned).unwrap_or_else(|| "NULL".into())
}

fn render_cells(row: &[Option<String>]) -> Vec<String> {
    row.iter().map(|cell| render_cell(cell.as_deref())).collect()
}

/// Stream-friendly: transform a batch of transfer rows in place conceptually (returns new rows).
///
/// The transfer model stores cells as `String` and encodes SQL `NULL` as the
/// empty string (`application/table_copy.rs`, `application/export_formats.rs`),
/// so a transfer row cannot distinguish SQL `NULL` from an empty text cell. Every
/// cell is therefore treated as present text; textual `"null"` is still masked,
/// while an empty cell keeps its existing rule behavior instead of being forced
/// to a fake `"NULL"`.
pub fn mask_transfer_batch(
    headers: &[String],
    batch: &[crate::domain::transfer::TransferRow],
    profile: &MaskingProfile,
    key_material: &str,
) -> Vec<crate::domain::transfer::TransferRow> {
    batch
        .iter()
        .map(|row| {
            let cells = row.cells.iter().cloned().map(Some).collect::<Vec<_>>();
            crate::domain::transfer::TransferRow {
                cells: mask_row(headers, &cells, profile, key_material),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/masking_tests.rs"]
mod tests;
