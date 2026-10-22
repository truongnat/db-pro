//! UI ↔ runtime command/event translation.
#[path = "translate_cmd.rs"]
mod translate_cmd;
#[path = "translate_evt.rs"]
mod translate_evt;
#[path = "translate_map.rs"]
mod translate_map;

#[cfg(test)]
pub(crate) use translate_cmd::draft_to_domain;
pub(crate) use translate_cmd::translate_command;
pub(crate) use translate_evt::translate_event;
