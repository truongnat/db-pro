# Checklist — #501

- [x] Confirm actions carry `backend_id` + `backend_start` (Option<String>)
- [x] `expected_backend_start` field added: UiCommand → RuntimeCommand → api → service → port
- [x] PG SQL guards `a.backend_start = $2::timestamptz` inside the signal statement
- [x] Missing/empty backend_start refused (app feedback + service Validation)
- [x] Tests: cancel/terminate carry backend_start; None → no dispatch; service rejects empty
- [x] `cargo check` on all touched crates
- [x] `cargo test` on touched modules
