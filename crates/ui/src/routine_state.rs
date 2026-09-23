//! State owned by the routine-management surface.

use super::UiFunctionSummary;

#[derive(Default)]
pub(super) struct RoutineState {
    pub(super) routine_source_draft: String,
    pub(super) routine_param_values: Vec<String>,
    pub(super) routine_param_nulls: Vec<bool>,
    pub(super) routine_ddl_preview: Option<String>,
    pub(super) routine_drop_confirm: bool,
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
}

#[cfg(test)]
mod tests {
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
}
