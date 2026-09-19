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
mod tests {
    use super::*;

    #[test]
    fn ordinary_typing_never_opens_completion_or_schedules_ai_prediction() {
        let response = SqlEditorResponse {
            changed: true,
            ..Default::default()
        };

        assert_eq!(
            EditorInteractionPolicy::completion_intent(&response, false, true),
            CompletionIntent::None
        );
        assert!(!EditorInteractionPolicy::should_schedule_prediction(
            &response,
            PredictionMode::Subtle,
            true,
            false,
        ));
    }

    #[test]
    fn completion_preserves_manual_and_automatic_trigger_intent() {
        let automatic = SqlEditorResponse {
            wants_completion: true,
            ..Default::default()
        };
        assert_eq!(
            EditorInteractionPolicy::completion_intent(&automatic, false, false),
            CompletionIntent::Open(CompletionTriggerKind::Automatic)
        );

        let manual = SqlEditorResponse {
            wants_completion: true,
            wants_manual_completion: true,
            ..Default::default()
        };
        assert_eq!(
            EditorInteractionPolicy::completion_intent(&manual, false, false),
            CompletionIntent::Open(CompletionTriggerKind::Manual)
        );
    }

    #[test]
    fn open_completion_refreshes_after_edit_and_closes_after_caret_move() {
        let edit = SqlEditorResponse {
            changed: true,
            ..Default::default()
        };
        assert_eq!(
            EditorInteractionPolicy::completion_intent(&edit, true, true),
            CompletionIntent::Refresh
        );

        let caret_move = SqlEditorResponse::default();
        assert_eq!(
            EditorInteractionPolicy::completion_intent(&caret_move, true, true),
            CompletionIntent::Close
        );
    }

    #[test]
    fn ai_prediction_requires_an_explicit_request_and_opted_in_mode() {
        let response = SqlEditorResponse {
            wants_manual_prediction: true,
            ..Default::default()
        };

        assert!(!EditorInteractionPolicy::should_schedule_prediction(
            &response,
            PredictionMode::Off,
            true,
            false,
        ));
        assert!(EditorInteractionPolicy::should_schedule_prediction(
            &response,
            PredictionMode::Subtle,
            true,
            false,
        ));
        assert!(!EditorInteractionPolicy::should_schedule_prediction(
            &response,
            PredictionMode::Eager,
            false,
            false,
        ));
        assert!(!EditorInteractionPolicy::should_schedule_prediction(
            &response,
            PredictionMode::Eager,
            true,
            true,
        ));
    }
}
