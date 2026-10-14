//! Aggregate state for database-administration features.
//!
//! Each child keeps its own domain-specific model and command helpers. The
//! aggregate exists to keep the application composition root from exposing a
//! flat list of unrelated administration lifecycles.

use super::{
    AuditState, EventTriggerState, FdwState, MaskingState, MonitoringState, PgSettingsState, ReplicationState,
    RoutineState, SecurityState, SyntheticDataState, TransferState,
};

#[derive(Default)]
pub(crate) struct DatabaseManagementState {
    pub(super) audit: AuditState,
    pub(super) event_trigger: EventTriggerState,
    pub(super) fdw: FdwState,
    pub(super) masking: MaskingState,
    pub(super) monitoring: MonitoringState,
    pub(super) pg_settings: PgSettingsState,
    pub(super) replication: ReplicationState,
    pub(super) routine: RoutineState,
    pub(super) security: SecurityState,
    pub(super) synthetic_data: SyntheticDataState,
    pub(super) transfer: TransferState,
}
