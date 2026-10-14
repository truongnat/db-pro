//! Native UI/runtime protocol facade.

#[path = "runtime_protocol.rs"]
mod protocol;
#[path = "runtime_connection_types.rs"]
mod runtime_connection_types;
#[path = "runtime_query_types.rs"]
mod runtime_query_types;
#[path = "runtime_schema_types.rs"]
mod runtime_schema_types;
#[path = "task_bridge.rs"]
mod task_bridge;

pub use protocol::{UiCommand, UiEvent};
pub use runtime_connection_types::*;
pub use runtime_query_types::*;
pub use runtime_schema_types::*;
pub use task_bridge::TaskBridge;
pub use task_bridge::MAX_RUNTIME_EVENTS_PER_FRAME;

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
