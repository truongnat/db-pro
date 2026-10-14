//! SQL prediction reducers over the query-session state.

use super::*;
use crate::RequestId;

pub(super) fn on_sql_prediction_ready(
    query_session: &mut QuerySessionState,
    request_id: RequestId,
    document_id: String,
    document_version: u64,
    anchor: usize,
    replacement_range: (usize, usize),
    prediction_text: String,
) {
    let Some(doc) = query_session.documents.iter_mut().find(|doc| doc.id == document_id) else {
        return;
    };
    if doc.pending_prediction_request != Some(request_id) {
        return;
    }
    doc.pending_prediction_request = None;
    if doc.buffer.version() != document_version || doc.cursor.offset != anchor {
        doc.prediction_stale_responses_dropped = doc.prediction_stale_responses_dropped.saturating_add(1);
        return;
    }
    let (before, after) = doc.buffer.split_at(anchor);
    let prediction_text = if replacement_range.0 == replacement_range.1 {
        crate::editor::prediction::normalize_prediction_overlap(before, after, &prediction_text)
    } else {
        prediction_text
    };
    if crate::editor::prediction::is_prediction_acceptable(&prediction_text) {
        doc.prediction = Some(crate::editor::prediction::EditPrediction::with_range_and_version(
            anchor,
            replacement_range,
            prediction_text,
            Some(request_id),
            document_version,
        ));
        if let Some(prediction) = doc.prediction.clone() {
            if let Some(fingerprint) = doc.prediction_context_fingerprint {
                doc.cache_prediction(fingerprint, prediction, std::time::Instant::now());
            }
        }
    } else {
        doc.prediction_rejected = doc.prediction_rejected.saturating_add(1);
    }
    if let Some(started_at) = doc.prediction_request_started_at.take() {
        doc.prediction_last_latency_ms = Some(started_at.elapsed().as_millis() as u64);
    }
}

pub(super) fn on_sql_prediction_failed(
    query_session: &mut QuerySessionState,
    request_id: RequestId,
    document_id: String,
) {
    if let Some(doc) = query_session
        .documents
        .iter_mut()
        .find(|doc| doc.id == document_id && doc.pending_prediction_request == Some(request_id))
    {
        doc.pending_prediction_request = None;
        doc.prediction_request_started_at = None;
    }
}
