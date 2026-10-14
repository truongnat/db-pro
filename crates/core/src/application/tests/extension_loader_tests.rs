//! Tests for `../extension_loader.rs` (moved out of the source file).
use super::*;
use crate::domain::extension::example_diagnostics_extension;

#[test]
fn loads_example_extension() {
    let mut reg = ExtensionRegistry::new();
    reg.load(example_diagnostics_extension()).unwrap();
    assert_eq!(reg.loaded_commands().len(), 1);
    assert_eq!(reg.loaded_sidebar().len(), 1);
    assert_eq!(reg.reports()[0].status, ExtensionLoadStatus::Loaded);
}

#[test]
fn incompatible_version_fails_cleanly_without_aborting_host() {
    let mut reg = ExtensionRegistry::new();
    let mut bad = example_diagnostics_extension();
    bad.id = "dbpro.example.future".into();
    bad.min_host_api = "99.0".into();
    let err = reg.load(bad).unwrap_err();
    assert!(matches!(err, ExtensionLoadError::IncompatibleHost { .. }));
    // Host still loads a good extension afterward.
    reg.load(example_diagnostics_extension()).unwrap();
    assert_eq!(reg.manifests().len(), 1);
    assert_eq!(reg.reports().len(), 2);
    assert_eq!(reg.reports()[0].status, ExtensionLoadStatus::Failed);
    assert_eq!(reg.reports()[1].status, ExtensionLoadStatus::Loaded);
}

#[test]
fn missing_permission_rejected() {
    let mut reg = ExtensionRegistry::new();
    let mut m = example_diagnostics_extension();
    m.permissions.clear();
    let err = reg.load(m).unwrap_err();
    assert!(matches!(
        err,
        ExtensionLoadError::MissingPermission(ExtensionPermission::RegisterCommand)
    ));
}
