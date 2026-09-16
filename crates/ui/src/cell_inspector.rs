//! Advanced cell / record value inspector (#228).
//!
//! Presentation helpers only — mutations still flow through ChangeSet.

use super::UiCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum CellInspectorMode {
    #[default]
    Raw,
    Pretty,
    Tree,
    Hex,
    Base64,
}

impl CellInspectorMode {
    pub(crate) fn as_label(self) -> &'static str {
        match self {
            Self::Raw => "Raw",
            Self::Pretty => "Pretty",
            Self::Tree => "Tree",
            Self::Hex => "Hex",
            Self::Base64 => "Base64",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellInspectorKind {
    Text,
    Json,
    Bytes,
    Scalar,
}

pub(crate) fn classify_cell(cell: &UiCell, data_type: &str) -> CellInspectorKind {
    let normalized = data_type.to_ascii_lowercase();
    if matches!(cell, UiCell::Json(_)) || normalized.contains("json") {
        return CellInspectorKind::Json;
    }
    if matches!(cell, UiCell::Bytes(_))
        || normalized.contains("bytea")
        || normalized.contains("blob")
        || normalized.contains("binary")
    {
        return CellInspectorKind::Bytes;
    }
    if matches!(cell, UiCell::Text(value) if value.chars().count() > 120) {
        return CellInspectorKind::Text;
    }
    CellInspectorKind::Scalar
}

pub(crate) fn cell_raw_text(cell: &UiCell) -> String {
    match cell {
        UiCell::Null => "NULL".to_owned(),
        UiCell::Boolean(v) => v.to_string(),
        UiCell::Number(v) | UiCell::Text(v) | UiCell::Json(v) | UiCell::Bytes(v) => v.clone(),
    }
}

pub(crate) fn pretty_json(raw: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    serde_json::to_string_pretty(&value).ok()
}

pub(crate) fn json_tree_lines(raw: &str, max_lines: usize) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return vec!["(not valid JSON)".into()];
    };
    let mut lines = Vec::new();
    walk_json(&value, String::new(), &mut lines, max_lines);
    if lines.is_empty() {
        lines.push("(empty)".into());
    }
    lines
}

fn walk_json(value: &serde_json::Value, path: String, out: &mut Vec<String>, max_lines: usize) {
    if out.len() >= max_lines {
        return;
    }
    match value {
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                out.push(format!("{path} = {{}}"));
                return;
            }
            for (key, child) in map {
                let next = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                walk_json(child, next, out, max_lines);
            }
        }
        serde_json::Value::Array(items) => {
            if items.is_empty() {
                out.push(format!("{path} = []"));
                return;
            }
            for (idx, child) in items.iter().enumerate() {
                walk_json(child, format!("{path}[{idx}]"), out, max_lines);
            }
        }
        other => {
            out.push(format!("{path} = {other}"));
        }
    }
}

/// Decode Postgres-style `\xDEADBEEF` or raw hex / latin1-ish UiCell::Bytes payloads.
pub(crate) fn decode_bytes_payload(raw: &str) -> Result<Vec<u8>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(hex) = trimmed.strip_prefix("\\x").or_else(|| trimmed.strip_prefix("0x")) {
        return decode_hex_digits(hex);
    }
    // Already base64?
    if let Ok(bytes) = base64_decode(trimmed) {
        return Ok(bytes);
    }
    Ok(trimmed.as_bytes().to_vec())
}

fn decode_hex_digits(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return Err("hex length must be even".into());
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    let bytes = hex.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_nibble(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("invalid hex digit {}", b as char)),
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    use std::collections::HashMap;
    // Minimal base64 without extra deps — reject if alphabet looks wrong.
    let alphabet: HashMap<u8, u8> = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .iter()
        .enumerate()
        .map(|(i, b)| (*b, i as u8))
        .collect();
    let cleaned: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if cleaned.is_empty() || !cleaned.iter().all(|b| *b == b'=' || alphabet.contains_key(b)) {
        return Err(());
    }
    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for byte in cleaned {
        if byte == b'=' {
            break;
        }
        let val = *alphabet.get(&byte).ok_or(())? as u32;
        buffer = (buffer << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::from("\\x");
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

pub(crate) fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

pub(crate) fn bytes_metadata(raw: &str) -> String {
    match decode_bytes_payload(raw) {
        Ok(bytes) => format!("{} byte(s)", bytes.len()),
        Err(err) => format!("unreadable ({err})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_and_tree_json() {
        let pretty = pretty_json(r#"{"a":1,"b":[true]}"#).unwrap();
        assert!(pretty.contains('\n'));
        let lines = json_tree_lines(r#"{"a":1,"b":[true]}"#, 20);
        assert!(lines.iter().any(|l| l.contains("a = 1")));
        assert!(lines.iter().any(|l| l.contains("b[0] = true")));
    }

    #[test]
    fn hex_round_trip_metadata() {
        let raw = "\\xdeadbeef";
        let bytes = decode_bytes_payload(raw).unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(encode_hex(&bytes), raw);
        assert!(bytes_metadata(raw).contains("4 byte"));
        let b64 = encode_base64(&bytes);
        assert_eq!(decode_bytes_payload(&b64).unwrap(), bytes);
    }

    #[test]
    fn classify_json_and_bytes() {
        assert_eq!(
            classify_cell(&UiCell::Json("{}".into()), "text"),
            CellInspectorKind::Json
        );
        assert_eq!(
            classify_cell(&UiCell::Bytes("\\x00".into()), "bytea"),
            CellInspectorKind::Bytes
        );
    }
}
