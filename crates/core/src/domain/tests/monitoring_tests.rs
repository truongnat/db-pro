//! Tests for `../monitoring.rs` (moved out of the source file).
use super::*;

#[test]
fn format_bytes_keeps_exact_byte_count() {
    assert_eq!(format_bytes_exact(1536), "1.50 KiB (1536 B)");
    assert!(format_bytes_exact(u64::MAX).contains(&u64::MAX.to_string()));
}

#[test]
fn sort_labels_are_stable() {
    assert_eq!(StatStatementSort::TotalTime.as_label(), "total time");
    assert_eq!(StatStatementSort::Calls.as_label(), "calls");
}
