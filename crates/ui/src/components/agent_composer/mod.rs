mod config;
mod handler;
mod ui;

pub use ui::AgentComposer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMode {
    Chat,
    Plan,
    Code,
}

impl AgentMode {
    pub fn label(&self) -> &'static str {
        match self {
            AgentMode::Chat => "Chat",
            AgentMode::Plan => "Plan",
            AgentMode::Code => "SQL Agent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentComposerAction {
    Submit,
    Stop,
    Clear,
}

#[cfg(test)]
mod tests {
    use super::{AgentComposerAction, AgentMode};

    #[test]
    fn mode_labels_remain_stable() {
        assert_eq!(AgentMode::Chat.label(), "Chat");
        assert_eq!(AgentMode::Plan.label(), "Plan");
        assert_eq!(AgentMode::Code.label(), "SQL Agent");
    }

    #[test]
    fn action_enum_keeps_the_public_variants() {
        assert_eq!(AgentComposerAction::Submit, AgentComposerAction::Submit);
        assert_ne!(AgentComposerAction::Submit, AgentComposerAction::Stop);
        assert_ne!(AgentComposerAction::Stop, AgentComposerAction::Clear);
    }
}
