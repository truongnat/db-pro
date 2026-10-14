use super::{CompletionTriggerKind, PredictionMode, SqlEditorResponse};

/// Centralizes editor interaction decisions so rendering remains a pure source of
/// user intents and application orchestration cannot accidentally reintroduce noisy
/// completion or prediction behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionIntent {
    None,
    Open(CompletionTriggerKind),
    Refresh,
    Close,
}

pub struct EditorInteractionPolicy;

impl EditorInteractionPolicy {
    pub fn completion_intent(
        response: &SqlEditorResponse,
        completion_is_open: bool,
        cursor_context_changed: bool,
    ) -> CompletionIntent {
        if response.wants_completion {
            let trigger = if response.wants_manual_completion {
                CompletionTriggerKind::Manual
            } else {
                CompletionTriggerKind::Automatic
            };
            return CompletionIntent::Open(trigger);
        }
        if completion_is_open && response.changed {
            return CompletionIntent::Refresh;
        }
        if completion_is_open && cursor_context_changed {
            return CompletionIntent::Close;
        }

        CompletionIntent::None
    }

    pub fn should_schedule_prediction(
        response: &SqlEditorResponse,
        mode: PredictionMode,
        selection_is_empty: bool,
        cursor_in_string_or_comment: bool,
    ) -> bool {
        response.wants_manual_prediction
            && mode != PredictionMode::Off
            && !response.wants_dismiss_prediction
            && selection_is_empty
            && !cursor_in_string_or_comment
    }
}

#[cfg(test)]
#[path = "tests/interaction_tests.rs"]
mod tests;
