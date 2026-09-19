/// Draft state for the welcome surface.
#[derive(Debug, Default)]
pub(crate) struct WelcomeState {
    pub(super) prompt: String,
}

#[cfg(test)]
mod tests {
    use super::WelcomeState;

    #[test]
    fn default_welcome_state_has_no_draft_prompt() {
        assert!(WelcomeState::default().prompt.is_empty());
    }
}
