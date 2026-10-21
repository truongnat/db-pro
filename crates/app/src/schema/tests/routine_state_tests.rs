//! Tests for `../routine_state.rs` (moved out of the source file).
    use super::*;
    use crate::UiRoutineParameter;

    #[test]
    fn sync_from_copies_source_defaults_and_resets_null_flags() {
        let mut state = RoutineState {
            routine_param_nulls: vec![true, true],
            ..RoutineState::default()
        };
        let function = UiFunctionSummary {
            schema: "public".into(),
            name: "calculate".into(),
            routine_type: "FUNCTION".into(),
            data_type: "integer".into(),
            definition: "RETURN 1".into(),
            identity_arguments: "value integer".into(),
            language: "sql".into(),
            volatility: "IMMUTABLE".into(),
            security_definer: false,
            parameters: vec![UiRoutineParameter {
                name: "value".into(),
                data_type: "integer".into(),
                mode: "IN".into(),
                has_default: true,
                default_expr: "1".into(),
            }],
        };

        state.sync_from(&function);

        assert_eq!(state.routine_source_draft, "RETURN 1");
        assert_eq!(state.routine_param_values, vec!["1"]);
        assert_eq!(state.routine_param_nulls, vec![false]);
    }
#[test]
fn debug_probe_cache_is_per_connection_and_needs_a_first_read() {
    let mut state = RoutineState::default();
    assert!(state.needs_debug_probe("conn-a"));
    state.begin_debug_probe(1, "conn-a");
    assert!(!state.needs_debug_probe("conn-a"), "an in-flight request must not re-fire");

    let probe = RoutineDebugProbe {
        state: db_pro_core::domain::routine_debug::RoutineDebugState::ExtensionMissing,
        extension_installed: false,
        extension_available: true,
        preload_configured: false,
        server_version: "18.6".into(),
        probe_error: None,
    };
    assert!(state.complete_debug_probe(1, "conn-a", probe.clone()));
    assert_eq!(state.routine_debug_probe_connection.as_deref(), Some("conn-a"));
    assert_eq!(state.routine_debug_probe, Some(probe));
    assert!(!state.needs_debug_probe("conn-a"), "a probe for this server is cached");
    assert!(state.needs_debug_probe("conn-b"), "another server needs its own read");
}

#[test]
fn stale_or_foreign_debug_probe_replies_are_rejected() {
    let mut state = RoutineState::default();
    let probe = RoutineDebugProbe::unsupported_driver();

    state.begin_debug_probe(7, "conn-a");
    assert!(!state.complete_debug_probe(8, "conn-a", probe.clone()), "wrong request id");
    // the rejection above consumed the pending request marker
    state.begin_debug_probe(7, "conn-a");
    assert!(
        !state.complete_debug_probe(7, "conn-b", probe.clone()),
        "a reply for another connection must not paint this workbench"
    );
    assert_eq!(state.routine_debug_probe, None);

    assert!(!state.complete_debug_probe(7, "conn-a", probe), "no request pending anymore");
}

#[test]
fn clear_debug_probe_forgets_everything_for_a_reconnect() {
    let mut state = RoutineState::default();
    state.begin_debug_probe(1, "conn-a");
    state.complete_debug_probe(1, "conn-a", RoutineDebugProbe::unsupported_driver());
    state.clear_debug_probe();
    assert!(state.routine_debug_probe.is_none());
    assert!(state.routine_debug_probe_connection.is_none());
    assert!(state.routine_debug_probe_request.is_none());
    assert!(state.needs_debug_probe("conn-a"), "a fresh read is required again");
}
