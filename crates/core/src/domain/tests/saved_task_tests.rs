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
