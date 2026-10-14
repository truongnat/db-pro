mod approval_ui;
mod config;
mod disclosure_ui;
mod handler;
mod ui;

pub use approval_ui::ExecutionApproval;
pub use disclosure_ui::AgentThinking;
use lucide_icons::Icon;
pub use ui::{ContextChip, StatusBadge, ToolCall};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextChipKind {
    Connection,
    Database,
    Schema,
    Table,
    Editor,
    File,
}

impl ContextChipKind {
    pub fn icon(&self) -> Icon {
        handler::context_chip_icon(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusBadgeVariant {
    Active,
    Running,
    Success,
    Warning,
    Destructive,
    Archived,
    Draft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionApprovalAction {
    Run,
    Preview,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct AgentTaskItem {
    pub title: String,
    pub status: AgentTaskStatus,
    pub detail: Option<String>,
}

impl AgentTaskItem {
    pub fn new(title: impl Into<String>, status: AgentTaskStatus) -> Self {
        Self {
            title: title.into(),
            status,
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

pub struct AgentPlan<'a> {
    title: &'a str,
    tasks: &'a [AgentTaskItem],
    theme: crate::DbProTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSqlActionKind {
    Explain,
    Optimize,
    FixError,
    GenerateMigration,
    DescribeSchema,
    ConvertDialect,
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
