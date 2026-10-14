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