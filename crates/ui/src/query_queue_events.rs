//! Query queue reducers over shared feedback state.

use super::FeedbackState;
use crate::RequestId;

pub(super) fn on_query_queued(feedback: &mut FeedbackState, request_id: RequestId) {
    feedback.set_runtime_message(format!("Query queued · request {}", request_id.0));
}
