// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! State owned by the routine-management surface.

use super::UiFunctionSummary;
use db_pro_core::domain::routine_debug::RoutineDebugProbe;

#[derive(Default)]
pub(super) struct RoutineState {
    pub(super) routine_source_draft: String,
    pub(super) routine_param_values: Vec<String>,
    pub(super) routine_param_nulls: Vec<bool>,
    pub(super) routine_ddl_preview: Option<String>,
    pub(super) routine_drop_confirm: bool,
    /// Last PL/pgSQL debug-capability probe read from the server (#252).
    pub(super) routine_debug_probe: Option<RoutineDebugProbe>,
    /// Connection the stored probe belongs to, so a switched connection cannot reuse it.
    pub(super) routine_debug_probe_connection: Option<String>,
    /// `(request id, connection id)` of the in-flight probe request.
    pub(super) routine_debug_probe_request: Option<(u64, String)>,
}

impl RoutineState {
    pub(super) fn sync_from(&mut self, function: &UiFunctionSummary) {
        self.routine_source_draft = function.definition.clone();
        let inputs = function.parameters.iter().filter(|parameter| {
            let mode = parameter.mode.to_ascii_uppercase();
            mode == "IN" || mode == "INOUT" || mode == "VARIADIC" || mode.is_empty()
        });
        self.routine_param_values = inputs
            .map(|parameter| {
                if parameter.has_default {
                    parameter.default_expr.clone()
                } else {
                    String::new()
                }
            })
            .collect();
        self.routine_param_nulls = vec![false; self.routine_param_values.len()];
    }

    /// Whether a fresh probe is needed for `connection_id` before the workbench renders.
    ///
    /// `false` while a request is already in flight (no duplicate round trips) and when the
    /// stored probe already describes this connection.
    pub(super) fn needs_debug_probe(&self, connection_id: &str) -> bool {
        if self.routine_debug_probe_request.is_some() {
            return false;
        }
        self.routine_debug_probe_connection.as_deref() != Some(connection_id) || self.routine_debug_probe.is_none()
    }

    /// Records that a probe request is pending, so the reply can be authenticated.
    pub(super) fn begin_debug_probe(&mut self, request_id: u64, connection_id: &str) {
        self.routine_debug_probe_request = Some((request_id, connection_id.to_owned()));
    }

    /// Stores a probe reply only if it answers the pending request for the same connection.
    ///
    /// Returns `false` for a stale or foreign reply, which must not paint another server's
    /// debug state onto the current workbench.
    pub(super) fn complete_debug_probe(
        &mut self,
        request_id: u64,
        connection_id: &str,
        probe: RoutineDebugProbe,
    ) -> bool {
        let Some((pending_id, pending_connection)) = self.routine_debug_probe_request.take() else {
            return false;
        };
        if pending_id != request_id || pending_connection != connection_id {
            return false;
        }
        self.routine_debug_probe_connection = Some(connection_id.to_owned());
        self.routine_debug_probe = Some(probe);
        true
    }

    /// Drops the cached probe, e.g. when the connection changes, so the next draw re-reads.
    pub(super) fn clear_debug_probe(&mut self) {
        self.routine_debug_probe = None;
        self.routine_debug_probe_connection = None;
        self.routine_debug_probe_request = None;
    }
}

#[cfg(test)]
#[path = "tests/routine_state_tests.rs"]
mod tests;
