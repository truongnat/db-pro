//! Tests for `../monitoring_state.rs` (moved out of the source file).
    use super::MonitoringState;

    #[test]
    fn defaults_keep_monitoring_safe_and_bounded() {
        let state = MonitoringState::default();

        assert!(state.monitoring_poll);
        assert!(state.monitoring_filter_active_only);
        assert_eq!(
            state.monitoring_stat_sort,
            db_pro_core::domain::monitoring::StatStatementSort::TotalTime
        );
        assert!(state.monitoring_snapshot.is_none());
    }

#[test]
fn monitoring_repaint_delay_targets_the_next_five_second_poll() {
    use super::{monitoring_poll_repaint_delay, MONITORING_POLL_INTERVAL};
    use std::time::{Duration, Instant};

    let now = Instant::now();
    assert_eq!(monitoring_poll_repaint_delay(Some(now - Duration::from_secs(3)), now), Duration::from_secs(2));
    assert_eq!(monitoring_poll_repaint_delay(Some(now - MONITORING_POLL_INTERVAL), now), Duration::from_millis(1));
    assert_eq!(monitoring_poll_repaint_delay(None, now), Duration::from_millis(1));
}
