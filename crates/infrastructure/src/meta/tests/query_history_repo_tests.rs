//! Tests for `../query_history_repo.rs` (moved out of the source file).
use super::parse_metric;

#[test]
fn query_history_metrics_reject_invalid_values() {
    assert_eq!(parse_metric("42", "row_count").unwrap(), 42);
    for value in ["", "-1", "1.5"] {
        assert!(parse_metric(value, "row_count").is_err(), "value {value:?} should fail");
    }
}
