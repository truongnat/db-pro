use super::rules::ValidationMode;
use super::state::FormState;

/// Applies validation timing to existing form state without mutating stored errors.
pub(super) fn compose_access_label(
    label: &str,
    required: bool,
    helper_text: Option<&str>,
    error_text: Option<&str>,
) -> String {
    let mut access_label = label.to_owned();
    if required {
        access_label.push_str(", required");
    }
    if let Some(error) = error_text.filter(|text| !text.trim().is_empty()) {
        access_label.push_str(". Error: ");
        access_label.push_str(error);
    } else if let Some(helper) = helper_text.filter(|text| !text.trim().is_empty()) {
        access_label.push_str(". ");
        access_label.push_str(helper);
    }
    access_label
}

pub(super) fn should_show_error(state: &FormState, field: &str) -> bool {
    if !state.has_error(field) {
        return false;
    }
    // Callers mark blur as touched; OnChange uses dirty state, while every mode reveals errors after submit.
    match state.mode {
        ValidationMode::OnSubmit => state.submitted,
        ValidationMode::OnBlur | ValidationMode::OnTouched => state.is_touched(field) || state.submitted,
        ValidationMode::OnChange => state.is_dirty(field) || state.submitted,
    }
}
