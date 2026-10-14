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
