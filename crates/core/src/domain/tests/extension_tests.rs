//! Tests for `../extension.rs` (moved out of the source file).
use super::*;

#[test]
fn host_version_compare() {
    assert!(host_api_satisfies("1.0", "1.0"));
    assert!(host_api_satisfies("1.2", "1.0"));
    assert!(!host_api_satisfies("1.0", "1.1"));
    assert!(!host_api_satisfies("0.9", "1.0"));
}

#[test]
fn example_manifest_serializes() {
    let json = serde_json::to_string(&example_diagnostics_extension()).unwrap();
    assert!(json.contains("dbpro.example.diagnostics"));
}
