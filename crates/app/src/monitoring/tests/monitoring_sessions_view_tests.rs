use super::*;

#[test]
fn session_actions_stay_inside_the_sidebar() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let session = db_pro_core::domain::monitoring::MonitorSession {
        backend_id: 4201,
        database: Some("sample_store".into()),
        username: Some("app_reader".into()),
        application_name: Some("reporting-worker".into()),
        client_addr: None,
        state: Some("active".into()),
        wait_event_type: None,
        wait_event: None,
        query_text: Some("SELECT id, email FROM public.customers WHERE active = true;".into()),
        query_duration_ms: Some(125),
        backend_start: None,
        xact_start: None,
        query_start: None,
        xact_age_ms: Some(125),
        backend_age_ms: None,
        idle_in_transaction: false,
        is_current: false,
    };
    let snapshot = db_pro_core::domain::monitoring::MonitoringSnapshot {
        connection_id: "capture".into(),
        driver: "postgresql".into(),
        sessions: vec![session],
        locks: Vec::new(),
        lock_error: None,
        relation_sizes: Vec::new(),
        server: None,
        local: None,
        workload: None,
        fetched_at_ms: 0,
        message: String::new(),
    };
    for width in [220.0, 260.0] {
        let mut filter = false;
        let mut view = MonitoringSessionsContext {
            theme: DbProTheme::dark(),
            snapshot: &snapshot,
            filter_active_only: &mut filter,
        };
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 500.0))),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let right = ui.max_rect().right();
                    assert!(view.draw(ui).is_empty());
                    assert!(
                        ui.min_rect().right() <= right + 1.0,
                        "session actions escape {width}-point sidebar: {:?}",
                        ui.min_rect()
                    );
                });
            },
        );
    }
}
