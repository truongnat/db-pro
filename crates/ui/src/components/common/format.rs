/// Safely truncates a string to at most `max_chars` characters, appending `…` if truncated.
///
/// This avoids splitting unicode multi-byte character boundaries and provides consistent
/// ellipsis formatting across all UI views and widgets.
pub fn truncate_ellipsis(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_owned();
    }
    let keep_chars = max_chars.saturating_sub(1);
    let truncated: String = text.chars().take(keep_chars).collect();
    format!("{truncated}…")
}

/// Formats a byte size into human-readable representation (e.g. `120 B`, `4.2 KB`, `18.5 MB`, `1.2 GB`).
pub fn format_bytes(bytes: u64) -> String {
    const KIBIBYTE: u64 = 1024;
    const MEBIBYTE: u64 = 1024 * KIBIBYTE;
    const GIBIBYTE: u64 = 1024 * MEBIBYTE;
    const TEBIBYTE: u64 = 1024 * GIBIBYTE;

    if bytes < KIBIBYTE {
        format!("{bytes} B")
    } else if bytes < MEBIBYTE {
        format!("{:.1} KB", bytes as f64 / KIBIBYTE as f64)
    } else if bytes < GIBIBYTE {
        format!("{:.1} MB", bytes as f64 / MEBIBYTE as f64)
    } else if bytes < TEBIBYTE {
        format!("{:.2} GB", bytes as f64 / GIBIBYTE as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TEBIBYTE as f64)
    }
}

/// Formats execution or latency durations in milliseconds into readable text (`15ms`, `1.24s`, `2m 14s`).
pub fn format_duration_millis(milliseconds: u64) -> String {
    const MILLISECONDS_PER_SECOND: u64 = 1_000;
    const MILLISECONDS_PER_MINUTE: u64 = 60 * MILLISECONDS_PER_SECOND;

    if milliseconds < MILLISECONDS_PER_SECOND {
        format!("{milliseconds}ms")
    } else if milliseconds < MILLISECONDS_PER_MINUTE {
        format!("{:.2}s", milliseconds as f64 / MILLISECONDS_PER_SECOND as f64)
    } else {
        let minutes = milliseconds / MILLISECONDS_PER_MINUTE;
        let seconds = (milliseconds % MILLISECONDS_PER_MINUTE) / MILLISECONDS_PER_SECOND;
        format!("{minutes}m {seconds}s")
    }
}

/// Formats an item count with singular/plural suffix (e.g. `1 row`, `42 rows`, `0 tables`).
pub fn format_count_with_suffix(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

/// Formats a table pagination range description (e.g. `Rows 1–50 of 200`, `Rows 1–25`, `0 rows`).
pub fn format_page_range(offset: u64, count: u64, total_rows: Option<u64>) -> String {
    if let Some(total) = total_rows {
        if total > 0 && count > 0 {
            let start = offset + 1;
            let end = (offset + count).min(total);
            format!("Rows {start}–{end} of {total}")
        } else if total > 0 {
            format!("0 of {total} rows")
        } else {
            "0 rows".to_owned()
        }
    } else if count > 0 {
        let start = offset + 1;
        let end = offset + count;
        format!("Rows {start}–{end}")
    } else {
        "0 rows".to_owned()
    }
}

/// Formats a percentage ratio with one decimal precision (e.g. `98.5%`, `100.0%`).
pub fn format_percentage(numerator: u64, denominator: u64) -> String {
    if denominator == 0 {
        return "0.0%".to_owned();
    }
    let ratio = (numerator as f64 / denominator as f64) * 100.0;
    format!("{ratio:.1}%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ellipsis_preserves_unicode_boundaries() {
        assert_eq!(truncate_ellipsis("short", 10), "short");
        assert_eq!(truncate_ellipsis("hello world", 5), "hell…");
        assert_eq!(truncate_ellipsis("tiếng việt unicode", 6), "tiếng…");
        assert_eq!(truncate_ellipsis("", 5), "");
    }

    #[test]
    fn format_bytes_uses_human_readable_units() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024 * 5), "5.0 MB");
    }

    #[test]
    fn format_duration_millis_uses_compact_units() {
        assert_eq!(format_duration_millis(45), "45ms");
        assert_eq!(format_duration_millis(1500), "1.50s");
        assert_eq!(format_duration_millis(65000), "1m 5s");
    }

    #[test]
    fn format_count_with_suffix_uses_plural_for_non_one_counts() {
        assert_eq!(format_count_with_suffix(1, "row", "rows"), "1 row");
        assert_eq!(format_count_with_suffix(0, "row", "rows"), "0 rows");
        assert_eq!(format_count_with_suffix(5, "table", "tables"), "5 tables");
    }

    #[test]
    fn format_page_range_covers_known_and_unknown_totals() {
        assert_eq!(format_page_range(0, 50, Some(200)), "Rows 1–50 of 200");
        assert_eq!(format_page_range(50, 50, Some(200)), "Rows 51–100 of 200");
        assert_eq!(format_page_range(180, 50, Some(200)), "Rows 181–200 of 200");
        assert_eq!(format_page_range(0, 25, None), "Rows 1–25");
        assert_eq!(format_page_range(0, 0, Some(0)), "0 rows");
    }

    #[test]
    fn format_percentage_handles_zero_denominator() {
        assert_eq!(format_percentage(985, 1000), "98.5%");
        assert_eq!(format_percentage(1, 1), "100.0%");
        assert_eq!(format_percentage(0, 0), "0.0%");
    }
}
