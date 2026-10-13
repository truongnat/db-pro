// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;

pub(super) struct TransferHarnessContext<'a> {
    pub(super) management: &'a mut DatabaseManagementState,
    pub(super) feedback: &'a mut FeedbackState,
    pub(super) table_details: &'a [UiTableSummary],
}

impl TransferHarnessContext<'_> {
    fn build_synthetic_plan(&self) -> Result<db_pro_core::domain::synthetic_data::SyntheticPlan, String> {
        synthetic_data::build_plan(
            &self.management.synthetic_data.synthetic_table,
            &self.management.synthetic_data.synthetic_row_count,
            &self.management.synthetic_data.synthetic_seed,
            &self.management.synthetic_data.synthetic_null_pct,
            self.table_details,
        )
    }

    pub(super) fn preview_synthetic_seed(&mut self) {
        match self.build_synthetic_plan() {
            Ok(plan) => match db_pro_core::domain::synthetic_data::generate_preview(&plan, 20) {
                Ok(preview) => {
                    self.management.synthetic_data.synthetic_preview = Some(preview);
                    self.management.synthetic_data.synthetic_error = None;
                }
                Err(err) => {
                    self.management.synthetic_data.synthetic_error = Some(err);
                    self.management.synthetic_data.synthetic_preview = None;
                }
            },
            Err(err) => {
                self.management.synthetic_data.synthetic_error = Some(err);
                self.management.synthetic_data.synthetic_preview = None;
            }
        }
    }

    pub(super) fn build_synthetic_seed_sql(&self) -> Result<(String, usize), String> {
        let plan = self.build_synthetic_plan()?;
        let count = usize::try_from(plan.row_count.min(500)).map_err(|_| "invalid row count".to_owned())?;
        let rows = db_pro_core::domain::synthetic_data::generate_rows(&plan, count)?;
        let sql = db_pro_core::domain::synthetic_data::render_insert_sql(&plan, &rows)?;
        Ok((sql, count))
    }

    pub(super) fn preview_masking_sample(&mut self) {
        self.management.masking.masking_preview = Some(masking::build_preview(
            &self.management.masking.masking_columns_csv,
            self.management.masking.masking_rule,
            self.management.masking.masking_keyed,
        ));
        self.management.masking.masking_error = None;
    }

    pub(crate) fn run_masked_csv_export_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileTarget, DelimitedFormat, TransferService};
        use db_pro_core::domain::masking::{mask_transfer_batch, ColumnMask, MaskingProfile};
        use db_pro_core::domain::transfer::{
            TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let path = std::env::temp_dir().join(format!(
            "db-pro-masked-{}.csv",
            self.management.transfer.transfer_jobs.len() + 1
        ));
        let headers = vec!["id".into(), "email".into(), "phone".into()];
        let profile = MaskingProfile {
            name: "export".into(),
            schema: String::new(),
            table: String::new(),
            columns: self
                .management
                .masking
                .masking_columns_csv
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|column| ColumnMask {
                    column: column.to_owned(),
                    rule: self.management.masking.masking_rule,
                    replacement: "[masked]".into(),
                    keep_prefix: 2,
                    keep_suffix: 2,
                })
                .collect(),
            keyed: self.management.masking.masking_keyed,
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
            id: format!("masked-{}", self.management.transfer.transfer_jobs.len() + 1),
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
                self.management.masking.masking_error = Some(err.to_string());
            }
        }
        self.management.transfer.transfer_jobs.insert(0, job);
        if self.management.transfer.transfer_jobs.len() > 40 {
            self.management.transfer.transfer_jobs.truncate(40);
        }
    }

    pub(crate) fn run_synthetic_transfer_harness(&mut self, cancel_midway: bool) {
        use db_pro_core::application::TransferService;
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus};

        let id = format!("xfer-{}", self.management.transfer.transfer_jobs.len() + 1);
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
        self.management.transfer.transfer_jobs.insert(0, job);
        if self.management.transfer.transfer_jobs.len() > 20 {
            self.management.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_csv_export_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileTarget, DelimitedFormat, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{
            TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let mut path = std::env::temp_dir();
        path.push(format!(
            "dbpro-export-{}.csv",
            self.management.transfer.transfer_jobs.len() + 1
        ));
        let id = format!("csv-{}", self.management.transfer.transfer_jobs.len() + 1);
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
        self.management.transfer.transfer_jobs.insert(0, job);
        if self.management.transfer.transfer_jobs.len() > 20 {
            self.management.transfer.transfer_jobs.truncate(20);
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

        let mut job = TransferJob::new_synthetic(
            format!("csv-import-{}", self.management.transfer.transfer_jobs.len() + 1),
            0,
            50,
        );
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
        self.management.transfer.transfer_jobs.insert(0, job);
        if self.management.transfer.transfer_jobs.len() > 20 {
            self.management.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_jsonl_export_harness(&mut self) {
        use db_pro_core::application::{JsonlFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!(
            "dbpro-export-{}.jsonl",
            self.management.transfer.transfer_jobs.len() + 1
        ));
        let mut job = TransferJob::new_synthetic(
            format!("jsonl-{}", self.management.transfer.transfer_jobs.len() + 1),
            400,
            50,
        );
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
        self.management.transfer.transfer_jobs.insert(0, job);
        if self.management.transfer.transfer_jobs.len() > 20 {
            self.management.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_excel_export_harness(&mut self) {
        use db_pro_core::application::{ExcelFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!(
            "dbpro-export-{}.xlsx",
            self.management.transfer.transfer_jobs.len() + 1
        ));
        let mut job = TransferJob::new_synthetic(
            format!("xlsx-{}", self.management.transfer.transfer_jobs.len() + 1),
            80,
            20,
        );
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
        self.management.transfer.transfer_jobs.insert(0, job);
        if self.management.transfer.transfer_jobs.len() > 20 {
            self.management.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_chosen_copy(&mut self, connector: &dyn db_pro_core::application::TableCopyConnector) {
        use db_pro_core::application::{copy_tables, CopyFormat, TableCopyRequest};
        use db_pro_core::domain::transfer::{
            DbTableEndpoint, DbTransferOptions, TransferJob, TransferStatus, TransferTransactionPolicy,
        };

        let transfer = &self.management.transfer;
        let format = match CopyFormat::parse(&transfer.copy_format) {
            Ok(format) => format,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        let conflict = match parse_copy_conflict(&transfer.copy_conflict) {
            Ok(conflict) => conflict,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        let request = TableCopyRequest {
            source_connection_id: transfer.source_connection_id.clone(),
            target_connection_id: transfer.target_connection_id.clone(),
            tables: split_list(&transfer.copy_tables),
            format,
            conflict,
            columns: split_list(&transfer.copy_columns),
        };
        let output = match copy_tables(connector, &request) {
            Ok(output) => output,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        self.management.transfer.last_copy_body = output.body.clone();
        let mut next_id = self.management.transfer.transfer_jobs.len();
        for qualified in output.tables.iter().rev() {
            next_id += 1;
            let (schema, table) = split_schema_table(qualified);
            let mut job = TransferJob::new_db_table_copy(
                format!("copy-{next_id}"),
                DbTableEndpoint {
                    connection_id: output.source_connection_id.clone(),
                    schema: schema.clone(),
                    table: table.clone(),
                },
                DbTableEndpoint {
                    connection_id: output.target_connection_id.clone(),
                    schema,
                    table,
                },
                100,
                DbTransferOptions {
                    conflict: output.conflict,
                    transaction: TransferTransactionPolicy::PerBatch,
                    create_target_if_missing: false,
                },
            );
            job.status = TransferStatus::Succeeded;
            job.progress.rows_read = output.rows_written + output.rows_skipped;
            job.progress.rows_written = output.rows_written;
            job.progress.error_rows = output.rows_skipped;
            self.management.transfer.transfer_jobs.insert(0, job);
        }
        if self.management.transfer.transfer_jobs.len() > 20 {
            self.management.transfer.transfer_jobs.truncate(20);
        }
        let format_name = match output.format {
            CopyFormat::Csv => "csv",
            CopyFormat::Json => "json",
        };
        self.feedback.runtime_message = format!(
            "Copied {} → {} · {format_name} · wrote {} skipped {}",
            output.source_connection_id, output.target_connection_id, output.rows_written, output.rows_skipped
        );
    }
}

fn parse_copy_conflict(value: &str) -> Result<db_pro_core::domain::transfer::TransferConflictPolicy, String> {
    use db_pro_core::domain::transfer::TransferConflictPolicy;
    match value.trim().to_ascii_lowercase().as_str() {
        "abort" => Ok(TransferConflictPolicy::Abort),
        "skip" => Ok(TransferConflictPolicy::Skip),
        "replace" => Ok(TransferConflictPolicy::Replace),
        _ => Err("choose abort, skip, or replace".into()),
    }
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split([',', '\n'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

fn split_schema_table(qualified: &str) -> (String, String) {
    match qualified.rsplit_once('.') {
        Some((schema, table)) if !schema.is_empty() && !table.is_empty() => (schema.to_owned(), table.to_owned()),
        _ => (String::new(), qualified.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiSchemaColumn;

    #[test]
    fn synthetic_seed_context_builds_sql_from_explicit_management_state() {
        let mut management = DatabaseManagementState::default();
        management.synthetic_data.synthetic_table = "public.users".to_owned();
        management.synthetic_data.synthetic_row_count = "2".to_owned();
        management.synthetic_data.synthetic_seed = "42".to_owned();
        management.synthetic_data.synthetic_null_pct = "0".to_owned();
        let table_details = vec![UiTableSummary {
            schema: "public".to_owned(),
            name: "users".to_owned(),
            row_count: None,
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: Vec::new(),
        }];
        let mut feedback = FeedbackState::default();
        let context = TransferHarnessContext {
            management: &mut management,
            feedback: &mut feedback,
            table_details: &table_details,
        };

        let (sql, count) = context.build_synthetic_seed_sql().expect("synthetic SQL");

        assert_eq!(count, 2);
        assert!(sql.contains("INSERT INTO \"public\".\"users\""));
    }

    struct OrdersCopy {
        queries: std::sync::Mutex<Vec<(String, String)>>,
        executes: std::sync::Mutex<Vec<(String, Vec<String>)>>,
    }

    impl OrdersCopy {
        fn new() -> Self {
            Self {
                queries: std::sync::Mutex::new(Vec::new()),
                executes: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl db_pro_core::application::TableCopyConnector for OrdersCopy {
        fn quote(&self, _connection_id: &str, identifier: &str) -> Result<String, String> {
            Ok(format!("\"{}\"", identifier.replace('"', "\"\"")))
        }

        fn placeholder(&self, _connection_id: &str, index: usize) -> Result<String, String> {
            Ok(format!("${index}"))
        }

        fn query(&self, connection_id: &str, sql: &str) -> Result<db_pro_core::domain::query::QueryResult, String> {
            use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};
            self.queries
                .lock()
                .expect("query log")
                .push((connection_id.to_owned(), sql.to_owned()));
            let (columns, rows) = if connection_id == "prod" {
                (
                    vec![("id", false), ("name", true)],
                    vec![
                        vec![CellValue::Text("1".into()), CellValue::Text("Alice".into())],
                        vec![CellValue::Text("2".into()), CellValue::Text("Bob".into())],
                    ],
                )
            } else if connection_id == "stage" {
                (vec![("id", false)], vec![vec![CellValue::Text("1".into())]])
            } else {
                return Err(format!("connection {connection_id} is not open"));
            };
            let row_count = rows.len() as u64;
            Ok(QueryResult {
                columns: columns
                    .into_iter()
                    .map(|(name, nullable)| ColumnMeta {
                        name: name.into(),
                        data_type: "text".into(),
                        nullable,
                    })
                    .collect(),
                rows: rows.into_iter().map(Row).collect(),
                row_count,
                duration_ms: 0,
            })
        }

        fn execute(
            &self,
            connection_id: &str,
            sql: &str,
            params: &[db_pro_core::domain::query::QueryParam],
        ) -> Result<u64, String> {
            let values = params
                .iter()
                .filter_map(|param| match param {
                    db_pro_core::domain::query::QueryParam::Text(value) => Some(value.clone()),
                    _ => None,
                })
                .collect();
            self.executes
                .lock()
                .expect("execute log")
                .push((format!("{connection_id} {sql}"), values));
            Ok(1)
        }
    }

    fn choose_orders(management: &mut DatabaseManagementState, format: &str, conflict: &str) {
        management.transfer.source_connection_id = "prod".into();
        management.transfer.target_connection_id = "stage".into();
        management.transfer.copy_tables = "public.orders".into();
        management.transfer.copy_format = format.into();
        management.transfer.copy_conflict = conflict.into();
    }

    fn run_copy(management: &mut DatabaseManagementState, connector: &OrdersCopy) -> String {
        let mut feedback = FeedbackState::default();
        let table_details = Vec::new();
        TransferHarnessContext {
            management,
            feedback: &mut feedback,
            table_details: &table_details,
        }
        .run_chosen_copy(connector);
        feedback.runtime_message
    }

    #[test]
    fn chosen_copy_reads_tables_from_empty_form() {
        use db_pro_core::domain::transfer::{
            TransferConflictPolicy, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let mut management = DatabaseManagementState::default();
        assert!(management.transfer.copy_columns.is_empty());
        choose_orders(&mut management, "csv", "skip");
        let connector = OrdersCopy::new();
        let message = run_copy(&mut management, &connector);
        assert!(message.contains("prod"), "{message}");
        assert!(message.contains("stage"), "{message}");
        assert!(!message.contains("src-conn"), "{message}");

        let job = &management.transfer.transfer_jobs[0];
        match &job.source {
            TransferSourceKind::Table {
                connection_id,
                schema,
                table,
            } => {
                assert_eq!(connection_id, "prod");
                assert_eq!(schema, "public");
                assert_eq!(table, "orders");
            }
            other => panic!("source {other:?}"),
        }
        match &job.target {
            TransferTargetKind::Table {
                connection_id,
                schema,
                table,
            } => {
                assert_eq!(connection_id, "stage");
                assert_eq!(schema, "public");
                assert_eq!(table, "orders");
            }
            other => panic!("target {other:?}"),
        }
        assert_eq!(job.options.conflict, TransferConflictPolicy::Skip);
        assert_eq!(job.status, TransferStatus::Succeeded);
        let body = management.transfer.last_copy_body.clone();
        assert!(body.starts_with("id,name\n"), "{body}");
        assert!(body.contains("2,Bob"), "{body}");
        assert!(!body.contains("1,Alice"), "{body}");
        assert!(!body.contains("src-conn"), "{body}");
        assert!(!body.contains("row-"), "{body}");
        let queries = connector.queries.lock().expect("query log");
        assert!(
            queries
                .iter()
                .any(|(id, sql)| id == "prod" && sql.contains("SELECT * FROM \"public\".\"orders\"")),
            "{queries:?}"
        );
        assert!(
            queries
                .iter()
                .any(|(id, sql)| id == "stage" && sql.contains("SELECT \"id\" FROM \"public\".\"orders\"")),
            "{queries:?}"
        );
        drop(queries);
        let executes = connector.executes.lock().expect("execute log");
        assert_eq!(executes.len(), 1, "{executes:?}");
        assert_eq!(executes[0].1, vec!["2".to_owned(), "Bob".to_owned()]);
        drop(executes);

        choose_orders(&mut management, "csv", "abort");
        let before = management.transfer.transfer_jobs.len();
        let abort = OrdersCopy::new();
        let message = run_copy(&mut management, &abort);
        assert!(message.contains("conflict on key 1"), "{message}");
        assert_eq!(management.transfer.transfer_jobs.len(), before);
        assert_eq!(management.transfer.last_copy_body, body);
        assert!(abort.executes.lock().expect("execute log").is_empty());

        choose_orders(&mut management, "json", "replace");
        let replace = OrdersCopy::new();
        let message = run_copy(&mut management, &replace);
        assert!(message.contains("json"), "{message}");
        let json = &management.transfer.last_copy_body;
        assert!(json.starts_with("[{"), "{json}");
        assert!(json.contains("\"name\":\"Alice\""), "{json}");
        assert!(json.contains("\"name\":\"Bob\""), "{json}");
        assert_eq!(
            management.transfer.transfer_jobs[0].options.conflict,
            TransferConflictPolicy::Replace
        );
        assert_eq!(management.transfer.transfer_jobs[0].status, TransferStatus::Succeeded);
        let writes = replace.executes.lock().expect("execute log");
        assert!(writes.iter().any(|(sql, _)| sql.contains("DELETE FROM")), "{writes:?}");
        assert!(writes.iter().any(|(_, params)| params == &["1".to_owned(), "Alice".to_owned()]));
        assert!(writes.iter().any(|(_, params)| params == &["2".to_owned(), "Bob".to_owned()]));
        drop(writes);

        let hub = include_str!("tools_hub_surface_view.rs");
        assert!(!hub.contains("Cross-connection streaming"));
        assert!(hub.contains("Copy chosen tables between connections as CSV or JSON"));
    }
}
