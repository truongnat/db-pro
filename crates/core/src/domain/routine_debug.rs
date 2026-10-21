//! PL/pgSQL routine-debug capability probe (#252).
//!
//! Server-side debugging of routines (breakpoints, stepping, variable inspection) is only
//! possible when the PostgreSQL server carries the `plugin_debugger` extension — the
//! extension that installs the `pldbgapi` functions a debugger talks to — *and* that module
//! appears in `shared_preload_libraries`, which needs a server-side config edit plus a
//! restart. Neither fact is driver state, so no capability preset may claim it: every preset
//! keeps [`crate::domain::capabilities::SchemaCapabilities::routine_debug`] at `false` and the
//! honest answer comes from [`crate::application::SchemaService::routine_debug_probe`].
//!
//! DB Pro never installs server extensions and never edits server configuration; this probe
//! only reads `pg_catalog`, so it is safe against read-only and production connections.

use serde::{Deserialize, Serialize};

use super::query::{CellValue, QueryResult};

/// Extensions that can provide the server-side PL/pgSQL debug API.
pub const DEBUG_EXTENSION_NAMES: [&str; 2] = ["plugin_debugger", "pldbgapi"];

/// Exact SQL sent to the server by the debug-capability probe.
///
/// Kept as one constant so the unit test and the running probe cannot drift apart.
pub const DEBUG_CAPABILITY_PROBE_SQL: &str = "SELECT \
    EXISTS (SELECT 1 FROM pg_catalog.pg_extension WHERE extname IN ('plugin_debugger', 'pldbgapi')) AS debug_extension_installed, \
    EXISTS (SELECT 1 FROM pg_catalog.pg_available_extensions WHERE name IN ('plugin_debugger', 'pldbgapi')) AS debug_extension_available, \
    pg_catalog.current_setting('shared_preload_libraries', true) LIKE '%plugin_debugger%' AS preload_configured, \
    pg_catalog.current_setting('server_version') AS server_version";

/// What the probe means for the routine workbench.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutineDebugState {
    /// Not a PostgreSQL connection: no PL/pgSQL debug provider exists for this driver.
    UnsupportedDriver,
    /// The catalog probe could not run or its row could not be read.
    ProbeFailed,
    /// PostgreSQL, but neither debug extension is installed on the server.
    ExtensionMissing,
    /// A debug extension is installed but not preloaded, so `pldbgapi` calls cannot work yet.
    ExtensionNotPreloaded,
    /// The server side is ready; DB Pro still ships no `pldbgapi` protocol client, so
    /// interactive stepping stays unavailable and only safe typed execution works.
    ServerReady,
}

impl RoutineDebugState {
    /// Whether interactive stepping (breakpoints, single-step, variable watch) can be
    /// offered in this build for this probe result.
    ///
    /// Always `false`: even a fully prepared server needs the `pldbgapi` protocol client
    /// that #252 blocks on, and AGENTS.md forbids faking the unsupported path.
    pub fn stepping_supported(self) -> bool {
        false
    }
}

/// Result of one debug-capability probe against one live connection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutineDebugProbe {
    pub state: RoutineDebugState,
    pub extension_installed: bool,
    pub extension_available: bool,
    pub preload_configured: bool,
    pub server_version: String,
    pub probe_error: Option<String>,
}

impl RoutineDebugProbe {
    /// Probe result for a driver that has no PL/pgSQL debug provider to look for.
    pub fn unsupported_driver() -> Self {
        Self {
            state: RoutineDebugState::UnsupportedDriver,
            extension_installed: false,
            extension_available: false,
            preload_configured: false,
            server_version: String::new(),
            probe_error: None,
        }
    }

    /// Probe result when the catalog read itself failed.
    pub fn probe_failed(error: String) -> Self {
        Self {
            state: RoutineDebugState::ProbeFailed,
            extension_installed: false,
            extension_available: false,
            preload_configured: false,
            server_version: String::new(),
            probe_error: Some(error),
        }
    }

    /// Interprets the probe row, falling back to [`RoutineDebugState::ProbeFailed`] when the
    /// server did not answer with the expected single row.
    pub fn from_probe_result(result: &QueryResult) -> Self {
        let Some(row) = result.rows.first() else {
            return Self::probe_failed("debug capability probe returned no rows".to_owned());
        };
        let cells = &row.0;
        let Some(installed) = cell_bool(cells.first()) else {
            return Self::probe_failed("debug capability probe returned an unreadable first column".to_owned());
        };
        let available = cell_bool(cells.get(1)).unwrap_or(false);
        let preloaded = cell_bool(cells.get(2)).unwrap_or(false);
        let server_version = cell_text(cells.get(3)).unwrap_or_default();
        let state = if !installed {
            RoutineDebugState::ExtensionMissing
        } else if !preloaded {
            RoutineDebugState::ExtensionNotPreloaded
        } else {
            RoutineDebugState::ServerReady
        };
        Self {
            state,
            extension_installed: installed,
            extension_available: available,
            preload_configured: preloaded,
            server_version,
            probe_error: None,
        }
    }
}

fn cell_bool(cell: Option<&CellValue>) -> Option<bool> {
    match cell? {
        CellValue::Bool(value) => Some(*value),
        CellValue::Int64(value) => Some(*value != 0),
        CellValue::Text(value) => value.parse::<bool>().ok(),
        _ => None,
    }
}

fn cell_text(cell: Option<&CellValue>) -> Option<String> {
    match cell? {
        CellValue::Text(value) => Some(value.clone()),
        CellValue::Null => None,
        other => Some(format!("{other:?}")),
    }
}

#[cfg(test)]
#[path = "tests/routine_debug_tests.rs"]
mod tests;
