//! Tests for `../saved_task.rs` (moved out of the source file).
use super::*;

#[test]
fn rejects_embedded_password_in_sql_payload() {
    let payload = SavedTaskPayload::Sql {
        sql: "SELECT 1 -- password=secret".into(),
    };
    assert!(payload.validate_no_embedded_secrets().is_err());
}

#[test]
fn destructive_sql_detected() {
    let payload = SavedTaskPayload::Sql {
        sql: "DELETE FROM users WHERE id = 1".into(),
    };
    assert!(payload.is_destructive());
    let select = SavedTaskPayload::Sql {
        sql: "SELECT * FROM users".into(),
    };
    assert!(!select.is_destructive());
}

#[test]
fn call_do_execute_are_destructive_via_shared_classifier() {
    // The old token blacklist missed CALL/DO/EXECUTE; the shared classifier is
    // the authority and rates them Destructive (#375).
    for sql in [
        "CALL dangerous_proc()",
        "DO $$ BEGIN PERFORM 1; END $$",
        "EXECUTE prepared_statement",
    ] {
        let payload = SavedTaskPayload::Sql { sql: sql.to_owned() };
        assert!(payload.is_destructive(), "{sql}");
    }
}

#[test]
fn sql_credential_literals_are_rejected() {
    for sql in [
        "CREATE ROLE app WITH LOGIN PASSWORD 'hunter2'",
        "CREATE USER u IDENTIFIED BY 's3cret'",
        "ALTER USER bob USING 'p@ss'",
    ] {
        let payload = SavedTaskPayload::Sql { sql: sql.to_owned() };
        assert!(payload.validate_no_embedded_secrets().is_err(), "{sql}");
    }
}

#[test]
fn sql_words_looking_like_credential_keywords_are_allowed() {
    let payload = SavedTaskPayload::Sql {
        sql: "SELECT password_changed_at, identified_by FROM users".into(),
    };
    assert!(payload.validate_no_embedded_secrets().is_ok());
}

#[test]
fn store_upsert_and_run_history() {
    let mut store = SavedTaskStore::new();
    let id = Uuid::new_v4();
    let task = SavedTask {
        id,
        name: "Nightly export".into(),
        description: "CSV".into(),
        connection_id: "conn-1".into(),
        payload: SavedTaskPayload::Export {
            table: Some("public.orders".into()),
            format: "csv".into(),
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_run: None,
        schedule: None,
    };
    store.upsert(task).unwrap();
    let run = SavedTaskRun {
        id: Uuid::new_v4(),
        task_id: id,
        status: SavedTaskRunStatus::Success,
        started_at: chrono::Utc::now(),
        finished_at: chrono::Utc::now(),
        duration_ms: 12,
        message: "ok".into(),
        trigger: SavedTaskRunTrigger::Manual,
    };
    store.record_run(run);
    assert_eq!(
        store.tasks[0].last_run.as_ref().unwrap().status,
        SavedTaskRunStatus::Success
    );
    assert_eq!(store.runs_for(&id).len(), 1);
}

#[test]
fn destructive_schedule_requires_explicit_allow() {
    let mut store = SavedTaskStore::new();
    let id = Uuid::new_v4();
    let err = store
        .upsert(SavedTask {
            id,
            name: "Wipe".into(),
            description: String::new(),
            connection_id: "conn-1".into(),
            payload: SavedTaskPayload::Sql {
                sql: "DELETE FROM t".into(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_run: None,
            schedule: Some(TaskSchedule::every_secs(60)),
        })
        .unwrap_err();
    assert!(err.contains("allow_destructive"));
}

#[test]
fn due_schedule_fires_once_and_advances() {
    let mut store = SavedTaskStore::new();
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let mut schedule = TaskSchedule::every_secs(30);
    schedule.next_run_at = Some(now - chrono::Duration::seconds(1));
    store
        .upsert(SavedTask {
            id,
            name: "Ping".into(),
            description: String::new(),
            connection_id: "conn-1".into(),
            payload: SavedTaskPayload::Sql { sql: "SELECT 1".into() },
            created_at: now,
            updated_at: now,
            last_run: None,
            schedule: Some(schedule),
        })
        .unwrap();
    let due = store.due_scheduled_task_ids(now);
    assert_eq!(due, vec![id]);
    let due_again = store.due_scheduled_task_ids(now);
    assert!(due_again.is_empty());
    let next = store.tasks[0].schedule.as_ref().unwrap().next_run_at.unwrap();
    assert!(next > now);
}

fn scheduled_task_fixture() -> SavedTask {
    let now = "2026-10-16T12:00:00Z".parse::<chrono::DateTime<chrono::Utc>>().unwrap();
    SavedTask {
        id: Uuid::from_u128(1),
        name: "Scheduled ping".into(),
        description: String::new(),
        connection_id: "conn-1".into(),
        payload: SavedTaskPayload::Sql { sql: "SELECT 1".into() },
        created_at: now,
        updated_at: now,
        last_run: None,
        schedule: Some(TaskSchedule {
            enabled: true,
            interval_secs: 30,
            expression: "every 30s".into(),
            next_run_at: Some(now),
            last_scheduled_at: None,
            missed_run_policy: MissedRunPolicy::SkipMissed,
            max_retries: 1,
            retry_count: 0,
            allow_destructive: false,
        }),
    }
}

#[test]
fn enabling_zero_interval_is_rejected_without_mutating_store() {
    for has_next_run in [false, true] {
        let mut task = scheduled_task_fixture();
        let id = task.id;
        let schedule = task.schedule.as_mut().unwrap();
        schedule.enabled = false;
        schedule.interval_secs = 0;
        schedule.retry_count = 1;
        schedule.last_scheduled_at = schedule.next_run_at;
        if !has_next_run {
            schedule.next_run_at = None;
        }
        let mut store = SavedTaskStore::new();
        store.upsert(task).expect("disabled zero interval can be stored");
        let before = store.clone();

        assert!(store.set_schedule_enabled(&id, true).is_err());
        assert_eq!(store, before, "failed enable must preserve all timing and retry state");
        assert!(store.due_scheduled_task_ids(before.tasks[0].created_at).is_empty());
        assert_eq!(store, before);
    }
}

#[test]
fn enabling_valid_schedule_preserves_existing_timing_and_retry_state() {
    let mut task = scheduled_task_fixture();
    let id = task.id;
    let schedule = task.schedule.as_mut().unwrap();
    schedule.enabled = false;
    schedule.retry_count = 1;
    let before = schedule.clone();
    let mut store = SavedTaskStore::new();
    store.upsert(task).unwrap();

    store.set_schedule_enabled(&id, true).unwrap();

    let mut expected = before;
    expected.enabled = true;
    assert_eq!(store.tasks[0].schedule.as_ref().unwrap(), &expected);
}

#[test]
fn enabling_valid_schedule_initializes_missing_next_run() {
    let mut task = scheduled_task_fixture();
    let id = task.id;
    let schedule = task.schedule.as_mut().unwrap();
    schedule.enabled = false;
    schedule.next_run_at = None;
    let mut store = SavedTaskStore::new();
    store.upsert(task).unwrap();
    let before = chrono::Utc::now();

    store.set_schedule_enabled(&id, true).unwrap();

    let after = chrono::Utc::now();
    let schedule = store.tasks[0].schedule.as_ref().unwrap();
    assert!(schedule.enabled);
    let next = schedule.next_run_at.unwrap();
    assert!(next >= before + chrono::Duration::seconds(30));
    assert!(next <= after + chrono::Duration::seconds(30));
}

#[test]
fn enabling_destructive_schedule_without_allow_is_atomic() {
    let mut task = scheduled_task_fixture();
    let id = task.id;
    task.payload = SavedTaskPayload::Sql {
        sql: "DELETE FROM t".into(),
    };
    task.schedule.as_mut().unwrap().enabled = false;
    let mut store = SavedTaskStore::new();
    store.upsert(task).unwrap();
    let before = store.clone();

    assert!(store.set_schedule_enabled(&id, true).is_err());
    assert_eq!(store, before);
}

#[test]
fn skip_missed_runs_current_interval_with_polling_jitter_once() {
    for lateness_ms in [0, 50, 1_000, 29_999] {
        let task = scheduled_task_fixture();
        let id = task.id;
        let slot = task.schedule.as_ref().unwrap().next_run_at.unwrap();
        let now = slot + chrono::Duration::milliseconds(lateness_ms);
        let mut store = SavedTaskStore::new();
        store.upsert(task).unwrap();
        let before = store.clone();

        assert!(store
            .due_scheduled_task_ids(slot - chrono::Duration::nanoseconds(1))
            .is_empty());
        assert_eq!(store, before);
        assert_eq!(store.due_scheduled_task_ids(now), vec![id]);
        assert!(store.due_scheduled_task_ids(now).is_empty());
        assert!(store
            .due_scheduled_task_ids(now + chrono::Duration::milliseconds(50))
            .is_empty());
        let schedule = store.tasks[0].schedule.as_ref().unwrap();
        assert_eq!(schedule.last_scheduled_at, Some(slot));
        assert_eq!(schedule.next_run_at, Some(now + chrono::Duration::seconds(30)));
    }
}

#[test]
fn skip_missed_discards_full_interval_backlog_and_closed_app_gap() {
    for lateness_secs in [30, 90, 7_200] {
        let task = scheduled_task_fixture();
        let id = task.id;
        let slot = task.schedule.as_ref().unwrap().next_run_at.unwrap();
        let now = slot + chrono::Duration::seconds(lateness_secs);
        let mut store = SavedTaskStore::new();
        store.upsert(task).unwrap();

        assert!(store.due_scheduled_task_ids(now).is_empty());
        let schedule = store.tasks[0].schedule.as_ref().unwrap();
        assert_eq!(schedule.next_run_at, Some(now + chrono::Duration::seconds(30)));
        assert_eq!(schedule.last_scheduled_at, None, "skipped backlog was not dispatched");
        assert!(store.due_scheduled_task_ids(now).is_empty());
        assert!(store
            .due_scheduled_task_ids(now + chrono::Duration::seconds(29))
            .is_empty());
        assert_eq!(
            store.due_scheduled_task_ids(now + chrono::Duration::seconds(30)),
            vec![id]
        );
        assert!(store
            .due_scheduled_task_ids(now + chrono::Duration::seconds(30))
            .is_empty());
    }
}

#[test]
fn run_once_dispatches_backlog_once_and_advances_from_now() {
    let mut task = scheduled_task_fixture();
    let id = task.id;
    let slot = task.schedule.as_ref().unwrap().next_run_at.unwrap();
    task.schedule.as_mut().unwrap().missed_run_policy = MissedRunPolicy::RunOnce;
    let now = slot + chrono::Duration::hours(2);
    let mut store = SavedTaskStore::new();
    store.upsert(task).unwrap();

    assert_eq!(store.due_scheduled_task_ids(now), vec![id]);
    let schedule = store.tasks[0].schedule.as_ref().unwrap();
    assert_eq!(schedule.next_run_at, Some(now + chrono::Duration::seconds(30)));
    assert_eq!(schedule.last_scheduled_at, Some(slot));
    assert!(store.due_scheduled_task_ids(now).is_empty());
    assert!(store
        .due_scheduled_task_ids(now + chrono::Duration::seconds(29))
        .is_empty());
}

#[test]
fn skip_missed_preserves_retry_after_failed_scheduled_run() {
    let task = scheduled_task_fixture();
    let id = task.id;
    let slot = task.schedule.as_ref().unwrap().next_run_at.unwrap();
    let mut store = SavedTaskStore::new();
    store.upsert(task).unwrap();
    assert_eq!(store.due_scheduled_task_ids(slot), vec![id]);
    store.record_run(SavedTaskRun {
        id: Uuid::from_u128(2),
        task_id: id,
        status: SavedTaskRunStatus::Failed,
        started_at: slot,
        finished_at: slot,
        duration_ms: 0,
        message: "failure".into(),
        trigger: SavedTaskRunTrigger::Scheduled,
    });
    let retry_at = slot + chrono::Duration::seconds(5);
    assert_eq!(store.tasks[0].schedule.as_ref().unwrap().next_run_at, Some(retry_at));
    assert_eq!(store.tasks[0].schedule.as_ref().unwrap().retry_count, 1);

    assert_eq!(
        store.due_scheduled_task_ids(retry_at + chrono::Duration::milliseconds(50)),
        vec![id]
    );
    assert!(store
        .due_scheduled_task_ids(retry_at + chrono::Duration::milliseconds(50))
        .is_empty());
}

#[test]
fn scheduler_still_blocks_destructive_payload_without_schedule_opt_in() {
    let mut task = scheduled_task_fixture();
    let now = task.created_at;
    task.payload = SavedTaskPayload::Sql {
        sql: "DELETE FROM t".into(),
    };
    let mut store = SavedTaskStore::new();
    // Persisted tasks can be loaded without upsert validation.
    store.tasks.push(task);

    assert!(store.due_scheduled_task_ids(now).is_empty());
    assert!(!store.tasks[0].schedule.as_ref().unwrap().enabled);
    assert!(store.tasks[0].schedule.as_ref().unwrap().last_scheduled_at.is_none());
}
