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
#[path = "tests/mod_tests.rs"]
mod tests;
