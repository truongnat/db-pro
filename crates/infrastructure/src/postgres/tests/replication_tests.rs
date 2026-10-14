//! Tests for `../replication.rs` (moved out of the source file).
use super::*;

#[test]
fn redact_helper_used() {
    assert!(redact_conninfo("password=x").contains("[redacted]"));
}
