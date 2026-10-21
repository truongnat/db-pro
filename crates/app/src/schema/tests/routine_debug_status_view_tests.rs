//! Tests for `../routine_debug_status_view.rs`.

use super::*;

fn probe_for(state: RoutineDebugState) -> RoutineDebugProbe {
    RoutineDebugProbe {
        state,
        extension_installed: matches!(
            state,
            RoutineDebugState::ExtensionNotPreloaded | RoutineDebugState::ServerReady
        ),
        extension_available: state != RoutineDebugState::UnsupportedDriver,
        preload_configured: state == RoutineDebugState::ServerReady,
        server_version: "18.6".to_owned(),
        probe_error: match state {
            RoutineDebugState::ProbeFailed => Some("permission denied for catalog".to_owned()),
            _ => None,
        },
    }
}

#[test]
fn every_probe_state_renders_a_reason_and_the_standing_stepping_note() {
    for state in [
        RoutineDebugState::UnsupportedDriver,
        RoutineDebugState::ProbeFailed,
        RoutineDebugState::ExtensionMissing,
        RoutineDebugState::ExtensionNotPreloaded,
        RoutineDebugState::ServerReady,
    ] {
        let guidance = build_debug_guidance(Some(&probe_for(state)), false, None);
        assert!(!guidance.status.is_empty(), "{state:?} must show a status badge");
        assert!(!guidance.reason.is_empty(), "{state:?} must explain the measured state");
        // A string still equal to its key means the catalog lookup failed in this locale.
        assert_ne!(guidance.stepping_note, "routine.debug_stepping_note");
        assert!(
            guidance.install_guide.is_some()
                == matches!(state, RoutineDebugState::ExtensionMissing | RoutineDebugState::ExtensionNotPreloaded),
            "{state:?}: manual guidance only appears where the server actually needs a change"
        );
    }
}

#[test]
fn missing_extension_reason_names_the_measured_server_version() {
    let guidance = build_debug_guidance(Some(&probe_for(RoutineDebugState::ExtensionMissing)), false, None);
    assert!(
        guidance.reason.contains("18.6"),
        "the version placeholder must interpolate in every locale; got: {}",
        guidance.reason
    );
}

#[test]
fn probe_failure_reason_carries_the_server_error_text() {
    let guidance = build_debug_guidance(Some(&probe_for(RoutineDebugState::ProbeFailed)), false, None);
    assert!(
        guidance.reason.contains("permission denied for catalog"),
        "the error placeholder must interpolate in every locale; got: {}",
        guidance.reason
    );
}

#[test]
fn production_warning_appears_only_for_a_production_environment() {
    let missing = probe_for(RoutineDebugState::ExtensionMissing);
    let with_label = build_debug_guidance(Some(&missing), false, Some("Production"));
    let warning = with_label.production_warning.expect("production connections must be warned");
    assert!(
        warning.contains("Production"),
        "the environment placeholder must interpolate in every locale; got: {warning}"
    );
    let without_label = build_debug_guidance(Some(&missing), false, None);
    assert_eq!(without_label.production_warning, None);
}

#[test]
fn unprobed_and_checking_variants_stay_distinct_and_honest() {
    let unprobed = build_debug_guidance(None, false, None);
    assert_ne!(unprobed.status, "routine.debug_state_unprobed");
    assert_ne!(unprobed.reason, "routine.debug_reason_unprobed");
    assert!(unprobed.install_guide.is_none(), "nothing is known yet, so nothing is prescribed");

    let checking = build_debug_guidance(None, true, None);
    assert_eq!(checking.status, checking.reason);
    assert_ne!(checking.status, "routine.debug_checking");
}

#[test]
fn every_debug_locale_key_resolves_to_catalog_text() {
    // Locale-agnostic coverage gate: in any active locale a missing catalog entry makes
    // `t!` echo the key back, so key != text proves all 5 catalogs carry the entry.
    let pairs = [
        ("routine.debug_title", t!("routine.debug_title")),
        ("routine.debug_recheck", t!("routine.debug_recheck")),
        ("routine.debug_checking", t!("routine.debug_checking")),
        ("routine.debug_state_unprobed", t!("routine.debug_state_unprobed")),
        ("routine.debug_reason_unprobed", t!("routine.debug_reason_unprobed")),
        ("routine.debug_state_unsupported", t!("routine.debug_state_unsupported")),
        ("routine.debug_reason_unsupported", t!("routine.debug_reason_unsupported")),
        ("routine.debug_state_probe_failed", t!("routine.debug_state_probe_failed")),
        (
            "routine.debug_state_missing",
            t!("routine.debug_state_missing"),
        ),
        (
            "routine.debug_state_not_preloaded",
            t!("routine.debug_state_not_preloaded"),
        ),
        (
            "routine.debug_state_server_ready",
            t!("routine.debug_state_server_ready"),
        ),
        (
            "routine.debug_reason_server_ready",
            t!("routine.debug_reason_server_ready"),
        ),
        ("routine.debug_install_guide", t!("routine.debug_install_guide")),
        ("routine.debug_stepping_note", t!("routine.debug_stepping_note")),
    ];
    for (key, text) in pairs {
        assert!(!text.is_empty(), "{key} resolved to empty text");
        assert_ne!(text.as_ref(), key, "{key} is missing from the active locale catalog");
    }
    let interpolated = [
        (
            "routine.debug_reason_missing",
            t!("routine.debug_reason_missing", version = "18.6"),
        ),
        (
            "routine.debug_reason_not_preloaded",
            t!("routine.debug_reason_not_preloaded"),
        ),
        (
            "routine.debug_reason_probe_failed",
            t!("routine.debug_reason_probe_failed", error = "catalog read failed"),
        ),
        (
            "routine.debug_production_warning",
            t!("routine.debug_production_warning", environment = "Production"),
        ),
    ];
    for (key, text) in interpolated {
        assert!(!text.is_empty(), "{key} resolved to empty text");
        assert_ne!(text.as_ref(), key, "{key} is missing from the active locale catalog");
    }
}
