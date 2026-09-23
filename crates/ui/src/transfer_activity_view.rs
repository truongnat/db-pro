use super::*;

impl DbProApp {
    pub(super) fn draw_transfers_activity(&mut self, ui: &mut egui::Ui) {
        let is_production = self
            .active_connection()
            .map(|connection| connection.environment.eq_ignore_ascii_case("Production"))
            .unwrap_or(false);
        let action = transfer_activity_surface_view::TransferActivityContext {
            theme: self.theme,
            management: &mut self.management,
            table_details: &self.schema.explorer.schema.table_details,
            is_production,
        }
        .draw(ui);
        if let Some(action) = action {
            self.apply_transfer_activity_action(action);
        }
    }

    fn apply_transfer_activity_action(&mut self, action: transfer_activity_surface_view::TransferActivityAction) {
        use transfer_activity_surface_view::TransferActivityAction;

        match action {
            TransferActivityAction::OpenBackupSettings => {
                self.workspace.activity = Activity::Settings;
                self.preferences.section = SettingsSection::Backup;
                self.workspace.sidebar_open = true;
            }
            TransferActivityAction::PreviewSyntheticSeed => self.preview_synthetic_seed(),
            TransferActivityAction::ExportSyntheticSeedSql => self.export_synthetic_seed_sql(),
            TransferActivityAction::ApplySyntheticSeed => self.apply_synthetic_seed(),
            TransferActivityAction::PreviewMasking => self.transfer_harness_context().preview_masking_sample(),
            TransferActivityAction::RunMaskedCsvExport => {
                self.transfer_harness_context().run_masked_csv_export_harness();
            }
            TransferActivityAction::RunSyntheticTransfer { cancel_midway } => {
                self.transfer_harness_context()
                    .run_synthetic_transfer_harness(cancel_midway);
            }
            TransferActivityAction::RunCsvExport => self.transfer_harness_context().run_csv_export_harness(),
            TransferActivityAction::RunCsvImportPreview => {
                self.transfer_harness_context().run_csv_import_preview_harness();
            }
            TransferActivityAction::RunJsonlExport => self.transfer_harness_context().run_jsonl_export_harness(),
            TransferActivityAction::RunExcelExport => self.transfer_harness_context().run_excel_export_harness(),
            TransferActivityAction::RunDbToDbTransfer => {
                self.transfer_harness_context().run_db_to_db_transfer_harness();
            }
            TransferActivityAction::ClearJobs => self.management.transfer.transfer_jobs.clear(),
        }
    }

    pub(crate) fn preview_synthetic_seed(&mut self) {
        self.transfer_harness_context().preview_synthetic_seed();
    }

    pub(crate) fn export_synthetic_seed_sql(&mut self) {
        match self.transfer_harness_context().build_synthetic_seed_sql() {
            Ok((sql, count)) => {
                self.set_active_query_text(sql);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.management.synthetic_data.synthetic_error = None;
                self.feedback.runtime_message = format!("Synthetic INSERT SQL ({count} rows) exported to Query editor");
            }
            Err(error) => {
                self.management.synthetic_data.synthetic_error = Some(error);
            }
        }
    }

    pub(crate) fn apply_synthetic_seed(&mut self) {
        let is_production = self
            .active_connection()
            .map(|connection| connection.environment.eq_ignore_ascii_case("Production"))
            .unwrap_or(false);
        if is_production && !self.management.synthetic_data.synthetic_production_confirm {
            self.management.synthetic_data.synthetic_error =
                Some("Production confirmation required before applying seed INSERT".into());
            return;
        }
        self.export_synthetic_seed_sql();
        if self.management.synthetic_data.synthetic_error.is_some() {
            return;
        }
        if self.connection.lifecycle.active_connection_id().is_none() || !self.connection.lifecycle.is_connected() {
            self.management.synthetic_data.synthetic_error = Some("Connect to a database before applying seed".into());
            return;
        }
        if self.dispatch_query() {
            self.feedback.runtime_message = "Synthetic seed INSERT dispatched via query runtime".into();
        }
    }

    fn transfer_harness_context(&mut self) -> transfer_harness_view::TransferHarnessContext<'_> {
        transfer_harness_view::TransferHarnessContext {
            management: &mut self.management,
            feedback: &mut self.feedback,
            table_details: &self.schema.explorer.schema.table_details,
        }
    }
}
