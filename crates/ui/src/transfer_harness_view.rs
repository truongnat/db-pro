use super::*;

impl DbProApp {
    fn build_synthetic_plan(&self) -> Result<db_pro_core::domain::synthetic_data::SyntheticPlan, String> {
        synthetic_data::build_plan(
            &self.synthetic_data.synthetic_table,
            &self.synthetic_data.synthetic_row_count,
            &self.synthetic_data.synthetic_seed,
            &self.synthetic_data.synthetic_null_pct,
            &self.schema_explorer.schema.table_details,
        )
    }

    pub(crate) fn preview_synthetic_seed(&mut self) {
        match self.build_synthetic_plan() {
            Ok(plan) => match db_pro_core::domain::synthetic_data::generate_preview(&plan, 20) {
                Ok(preview) => {
                    self.synthetic_data.synthetic_preview = Some(preview);
                    self.synthetic_data.synthetic_error = None;
                }
                Err(err) => {
                    self.synthetic_data.synthetic_error = Some(err);
                    self.synthetic_data.synthetic_preview = None;
                }
            },
            Err(err) => {
                self.synthetic_data.synthetic_error = Some(err);
                self.synthetic_data.synthetic_preview = None;
            }
        }
    }

    pub(crate) fn export_synthetic_seed_sql(&mut self) {
        let plan = match self.build_synthetic_plan() {
            Ok(p) => p,
            Err(err) => {
                self.synthetic_data.synthetic_error = Some(err);
                return;
            }
        };
        let count = match usize::try_from(plan.row_count.min(500)) {
            Ok(count) => count,
            Err(_) => {
                self.synthetic_data.synthetic_error = Some("invalid row count".to_owned());
                return;
            }
        };
        let rows = match db_pro_core::domain::synthetic_data::generate_rows(&plan, count) {
            Ok(r) => r,
            Err(err) => {
                self.synthetic_data.synthetic_error = Some(err);
                return;
            }
        };
        match db_pro_core::domain::synthetic_data::render_insert_sql(&plan, &rows) {
            Ok(sql) => {
                self.set_active_query_text(sql);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.synthetic_data.synthetic_error = None;
                self.feedback.runtime_message = format!("Synthetic INSERT SQL ({count} rows) exported to Query editor");
            }
            Err(err) => self.synthetic_data.synthetic_error = Some(err),
        }
    }

    pub(crate) fn apply_synthetic_seed(&mut self) {
        let is_production = self
            .active_connection()
            .map(|c| c.environment.eq_ignore_ascii_case("Production"))
            .unwrap_or(false);
        if is_production && !self.synthetic_data.synthetic_production_confirm {
            self.synthetic_data.synthetic_error =
                Some("Production confirmation required before applying seed INSERT".into());
            return;
        }
        self.export_synthetic_seed_sql();
        if self.synthetic_data.synthetic_error.is_some() {
            return;
        }
        if self.connection.lifecycle.active_connection_id().is_none() || !self.connection.lifecycle.is_connected() {
            self.synthetic_data.synthetic_error = Some("Connect to a database before applying seed".into());
            return;
        }
        self.dispatch_query();
        self.feedback.runtime_message = "Synthetic seed INSERT dispatched via query runtime".into();
    }

    pub(crate) fn preview_masking_sample(&mut self) {
        self.masking.masking_preview = Some(masking::build_preview(
            &self.masking.masking_columns_csv,
            self.masking.masking_rule,
            self.masking.masking_keyed,
        ));
        self.masking.masking_error = None;
    }

    pub(crate) fn run_masked_csv_export_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileTarget, DelimitedFormat, TransferService};
        use db_pro_core::domain::masking::{mask_transfer_batch, ColumnMask, MaskingProfile};
        use db_pro_core::domain::transfer::{
            TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let path = std::env::temp_dir().join(format!("db-pro-masked-{}.csv", self.transfer.transfer_jobs.len() + 1));
        let headers = vec!["id".into(), "email".into(), "phone".into()];
        let profile = MaskingProfile {
            name: "export".into(),
            schema: String::new(),
            table: String::new(),
            columns: self
                .masking
                .masking_columns_csv
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|column| ColumnMask {
                    column: column.to_owned(),
                    rule: self.masking.masking_rule,
                    replacement: "[masked]".into(),
                    keep_prefix: 2,
                    keep_suffix: 2,
                })
                .collect(),
            keyed: self.masking.masking_keyed,
            key_id: "local-dev".into(),
        };
        let raw = vec![
            db_pro_core::domain::transfer::TransferRow {
                cells: vec!["1".into(), "ada@example.com".into(), "1234567890".into()],
            },
            db_pro_core::domain::transfer::TransferRow {
                cells: vec!["2".into(), "grace@example.com".into(), "0987654321".into()],
            },
        ];
        let masked = mask_transfer_batch(&headers, &raw, &profile, "db-pro-local-masking-key");
        let mut job = TransferJob {
            id: format!("masked-{}", self.transfer.transfer_jobs.len() + 1),
            label: format!("Masked CSV → {}", path.display()),
            source: TransferSourceKind::Synthetic {
                rows: masked.len() as u64,
            },
            target: TransferTargetKind::File {
                path: path.display().to_string(),
                format: "csv".into(),
            },
            mapping: Default::default(),
            batch_size: 50,
            options: Default::default(),
            status: TransferStatus::Pending,
            progress: Default::default(),
            error: None,
        };
        match DelimitedFileTarget::create(&path, DelimitedFormat::Csv, headers.clone()) {
            Ok(mut target) => {
                let cancel = TransferCancellation::new();
                struct MaskedSource {
                    rows: Vec<db_pro_core::domain::transfer::TransferRow>,
                    idx: usize,
                }
                impl db_pro_core::application::TransferSource for MaskedSource {
                    fn next_batch(
                        &mut self,
                        max_rows: usize,
                    ) -> Result<
                        Option<Vec<db_pro_core::domain::transfer::TransferRow>>,
                        db_pro_core::domain::transfer::TransferError,
                    > {
                        if self.idx >= self.rows.len() {
                            return Ok(None);
                        }
                        let end = (self.idx + max_rows).min(self.rows.len());
                        let batch = self.rows[self.idx..end].to_vec();
                        self.idx = end;
                        Ok(Some(batch))
                    }
                }
                let mut source = MaskedSource { rows: masked, idx: 0 };
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
                self.feedback.runtime_message = format!("Masked CSV written to {}", path.display());
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
                self.masking.masking_error = Some(err.to_string());
            }
        }
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 40 {
            self.transfer.transfer_jobs.truncate(40);
        }
    }

    pub(crate) fn run_synthetic_transfer_harness(&mut self, cancel_midway: bool) {
        use db_pro_core::application::TransferService;
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus};

        let id = format!("xfer-{}", self.transfer.transfer_jobs.len() + 1);
        let mut job = TransferJob::new_synthetic(id, if cancel_midway { 20_000 } else { 5_000 }, 128);
        let cancel = TransferCancellation::new();
        if cancel_midway {
            use db_pro_core::application::{CountingTarget, SyntheticSource, TransferSource, TransferTarget};
            let mut source = SyntheticSource::new(20_000);
            let mut target = CountingTarget::default();
            job.status = TransferStatus::Running;
            if let Ok(Some(batch)) = source.next_batch(job.batch_size) {
                job.progress.rows_read += batch.len() as u64;
                if let Ok(written) = target.write_batch(&batch) {
                    job.progress.rows_written += written;
                    job.progress.bytes_written = target.bytes_written;
                }
            }
            cancel.cancel();
            let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            job.progress.bytes_written = target.bytes_written;
        } else {
            let _ = TransferService::run_synthetic(&mut job, &cancel);
        }
        self.feedback.runtime_message = format!(
            "Transfer {} · {:?} · wrote {}",
            job.id, job.status, job.progress.rows_written
        );
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_csv_export_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileTarget, DelimitedFormat, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{
            TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.csv", self.transfer.transfer_jobs.len() + 1));
        let id = format!("csv-{}", self.transfer.transfer_jobs.len() + 1);
        let mut job = TransferJob {
            id,
            label: format!("CSV export → {}", path.display()),
            source: TransferSourceKind::Synthetic { rows: 1_000 },
            target: TransferTargetKind::File {
                path: path.to_string_lossy().into_owned(),
                format: "csv".into(),
            },
            mapping: Default::default(),
            batch_size: 100,
            options: Default::default(),
            status: TransferStatus::Pending,
            progress: Default::default(),
            error: None,
        };
        let mut source = SyntheticSource::new(1_000);
        let target = DelimitedFileTarget::create(&path, DelimitedFormat::Csv, vec!["id".into(), "name".into()]);
        match target {
            Ok(mut target) => {
                let cancel = TransferCancellation::new();
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.feedback.runtime_message = format!("CSV transfer {} · {:?} · {}", job.id, job.status, path.display());
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_csv_import_preview_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileSource, DelimitedFileTarget, DelimitedFormat};
        use db_pro_core::domain::transfer::{TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push("dbpro-import-sample.csv");
        let _ = DelimitedFileTarget::create(&path, DelimitedFormat::Csv, vec!["id".into(), "name".into()]).and_then(
            |mut target| {
                use db_pro_core::application::TransferTarget;
                use db_pro_core::domain::transfer::TransferRow;
                target.write_batch(&[
                    TransferRow {
                        cells: vec!["1".into(), "alpha".into()],
                    },
                    TransferRow {
                        cells: vec!["2".into(), "beta".into()],
                    },
                ])?;
                target.finish()
            },
        );

        let mut job =
            TransferJob::new_synthetic(format!("csv-import-{}", self.transfer.transfer_jobs.len() + 1), 0, 50);
        job.label = format!("CSV import preview ← {}", path.display());
        job.target = TransferTargetKind::File {
            path: path.to_string_lossy().into_owned(),
            format: "csv".into(),
        };
        match DelimitedFileSource::open(&path, DelimitedFormat::Csv, true) {
            Ok(mut source) => match source.preview_rows(20) {
                Ok(rows) => {
                    job.status = TransferStatus::Succeeded;
                    job.progress.rows_read = rows.len() as u64;
                    job.progress.message = format!("headers={:?}; preview {} row(s)", source.headers(), rows.len());
                }
                Err(err) => {
                    job.status = TransferStatus::Failed;
                    job.error = Some(err.to_string());
                }
            },
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.feedback.runtime_message = job.progress.message.clone();
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_jsonl_export_harness(&mut self) {
        use db_pro_core::application::{JsonlFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.jsonl", self.transfer.transfer_jobs.len() + 1));
        let mut job = TransferJob::new_synthetic(format!("jsonl-{}", self.transfer.transfer_jobs.len() + 1), 400, 50);
        job.label = format!("JSONL export → {}", path.display());
        job.target = TransferTargetKind::File {
            path: path.to_string_lossy().into_owned(),
            format: "jsonl".into(),
        };
        match JsonlFileTarget::create(&path, vec!["id".into(), "name".into()]) {
            Ok(mut target) => {
                let mut source = SyntheticSource::new(400);
                let cancel = TransferCancellation::new();
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.feedback.runtime_message = format!("JSONL {} · {:?}", job.id, job.status);
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_excel_export_harness(&mut self) {
        use db_pro_core::application::{ExcelFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.xlsx", self.transfer.transfer_jobs.len() + 1));
        let mut job = TransferJob::new_synthetic(format!("xlsx-{}", self.transfer.transfer_jobs.len() + 1), 80, 20);
        job.label = format!("Excel export → {}", path.display());
        job.target = TransferTargetKind::File {
            path: path.to_string_lossy().into_owned(),
            format: "xlsx".into(),
        };
        match ExcelFileTarget::create(&path, "Sheet1", vec!["id".into(), "name".into()]) {
            Ok(mut target) => {
                let mut source = SyntheticSource::new(80);
                let cancel = TransferCancellation::new();
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.feedback.runtime_message = format!("Excel {} · {:?}", job.id, job.status);
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_db_to_db_transfer_harness(&mut self) {
        use db_pro_core::application::{
            assert_endpoint_capabilities, build_conversion_plan, DbColumnSpec, GeneratorTableSource, MemoryTableTarget,
            TransferService,
        };
        use db_pro_core::domain::connection::DriverType;
        use db_pro_core::domain::transfer::{
            DbTableEndpoint, DbTransferOptions, TransferCancellation, TransferConflictPolicy, TransferJob,
            TransferMapping, TransferStatus, TransferTransactionPolicy,
        };

        let source_cols = vec![
            DbColumnSpec {
                name: "id".into(),
                type_name: "uuid".into(),
            },
            DbColumnSpec {
                name: "payload".into(),
                type_name: "jsonb".into(),
            },
        ];
        let target_cols = vec![
            DbColumnSpec {
                name: "id".into(),
                type_name: "text".into(),
            },
            DbColumnSpec {
                name: "payload".into(),
                type_name: "text".into(),
            },
        ];
        let plan = match build_conversion_plan(
            DriverType::Postgres,
            DriverType::SQLite,
            &source_cols,
            &target_cols,
            &TransferMapping::default(),
        ) {
            Ok(plan) => plan,
            Err(err) => {
                self.feedback.runtime_message = format!("DB→DB plan failed: {err}");
                return;
            }
        };
        if let Err(err) = plan.ensure_runnable() {
            self.feedback.runtime_message = format!("DB→DB blocked: {err}");
            return;
        }
        if let Err(err) = assert_endpoint_capabilities(true, true, false, false) {
            self.feedback.runtime_message = format!("DB→DB capability gate: {err}");
            return;
        }

        let mut job = TransferJob::new_db_table_copy(
            format!("dbdb-{}", self.transfer.transfer_jobs.len() + 1),
            DbTableEndpoint {
                connection_id: "src-conn".into(),
                schema: "public".into(),
                table: "events".into(),
            },
            DbTableEndpoint {
                connection_id: "dst-conn".into(),
                schema: "main".into(),
                table: "events".into(),
            },
            100,
            DbTransferOptions {
                conflict: TransferConflictPolicy::Skip,
                transaction: TransferTransactionPolicy::PerBatch,
                create_target_if_missing: false,
            },
        );
        job.label = format!(
            "DB→DB PG→SQLite preview ({} cols, {} warnings)",
            plan.columns.len(),
            plan.warnings.len()
        );
        let mut source = GeneratorTableSource::new(1_500);
        let mut target = MemoryTableTarget::new(TransferConflictPolicy::Skip, TransferTransactionPolicy::PerBatch);
        let cancel = TransferCancellation::new();
        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        job.progress.bytes_written = target.bytes_written;
        if result.status == TransferStatus::Succeeded || result.status == TransferStatus::Partial {
            self.feedback.runtime_message = format!(
                "DB→DB {} · {:?} · committed_batches={} · warnings={}",
                job.id,
                result.status,
                result.progress.committed_batches,
                plan.warnings.len()
            );
        } else {
            self.feedback.runtime_message = format!("DB→DB {} · {:?} · {:?}", job.id, result.status, job.error);
        }
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }
}
