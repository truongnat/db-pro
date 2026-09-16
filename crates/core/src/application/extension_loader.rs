//! Extension loader / registry (#256).
//!
//! Isolates load failures so the host continues when one extension is bad.
//! Does not grant filesystem/network/database access beyond declared permissions.

use crate::domain::extension::{
    host_api_satisfies, ExtensionCommand, ExtensionContributions, ExtensionLoadError, ExtensionLoadReport,
    ExtensionLoadStatus, ExtensionManifest, ExtensionPermission, ExtensionSidebarItem, HOST_API_VERSION,
};

#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    manifests: Vec<ExtensionManifest>,
    commands: Vec<(String, ExtensionCommand)>,
    sidebar: Vec<(String, ExtensionSidebarItem)>,
    reports: Vec<ExtensionLoadReport>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reports(&self) -> &[ExtensionLoadReport] {
        &self.reports
    }

    pub fn loaded_commands(&self) -> &[(String, ExtensionCommand)] {
        &self.commands
    }

    pub fn loaded_sidebar(&self) -> &[(String, ExtensionSidebarItem)] {
        &self.sidebar
    }

    pub fn manifests(&self) -> &[ExtensionManifest] {
        &self.manifests
    }

    /// Attempt to load `manifest`. On failure, record a report and keep the host usable.
    pub fn load(&mut self, manifest: ExtensionManifest) -> Result<(), ExtensionLoadError> {
        let id = manifest.id.clone();
        match self.validate_and_register(manifest) {
            Ok(()) => {
                self.reports.push(ExtensionLoadReport {
                    id,
                    status: ExtensionLoadStatus::Loaded,
                    message: "loaded".into(),
                });
                Ok(())
            }
            Err(err) => {
                self.reports.push(ExtensionLoadReport {
                    id,
                    status: ExtensionLoadStatus::Failed,
                    message: err.to_string(),
                });
                Err(err)
            }
        }
    }

    /// Load many manifests; never aborts the batch on a single failure.
    pub fn load_all(&mut self, manifests: Vec<ExtensionManifest>) -> Vec<ExtensionLoadReport> {
        for m in manifests {
            let _ = self.load(m);
        }
        self.reports.clone()
    }

    fn validate_and_register(&mut self, manifest: ExtensionManifest) -> Result<(), ExtensionLoadError> {
        if manifest.id.trim().is_empty() {
            return Err(ExtensionLoadError::InvalidManifest("id is empty".into()));
        }
        if self.manifests.iter().any(|m| m.id == manifest.id) {
            return Err(ExtensionLoadError::DuplicateId(manifest.id));
        }
        if !host_api_satisfies(HOST_API_VERSION, &manifest.min_host_api) {
            return Err(ExtensionLoadError::IncompatibleHost {
                required: manifest.min_host_api,
                host: HOST_API_VERSION.into(),
            });
        }
        self.ensure_permission(&manifest, ExtensionPermission::RegisterCommand, |c| {
            !c.commands.is_empty()
        })?;
        self.ensure_permission(&manifest, ExtensionPermission::RegisterAgentTool, |c| {
            !c.agent_tools.is_empty()
        })?;
        self.ensure_permission(&manifest, ExtensionPermission::ContributeSidebar, |c| {
            !c.sidebar_items.is_empty()
        })?;

        for cmd in &manifest.contributions.commands {
            self.commands.push((manifest.id.clone(), cmd.clone()));
        }
        for item in &manifest.contributions.sidebar_items {
            self.sidebar.push((manifest.id.clone(), item.clone()));
        }
        self.manifests.push(manifest);
        Ok(())
    }

    fn ensure_permission(
        &self,
        manifest: &ExtensionManifest,
        permission: ExtensionPermission,
        needed: impl Fn(&ExtensionContributions) -> bool,
    ) -> Result<(), ExtensionLoadError> {
        if needed(&manifest.contributions) && !manifest.permissions.contains(&permission) {
            return Err(ExtensionLoadError::MissingPermission(permission));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
}
