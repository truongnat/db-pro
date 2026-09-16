//! CSV/TSV streaming adapters for TransferService (#194).

use crate::domain::transfer::{TransferError, TransferRow};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use super::transfer_service::{TransferSource, TransferTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimitedFormat {
    Csv,
    Tsv,
}

impl DelimitedFormat {
    pub fn delimiter(self) -> u8 {
        match self {
            Self::Csv => b',',
            Self::Tsv => b'\t',
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Tsv => "tsv",
        }
    }
}

/// Streaming delimited file source (does not load the whole file).
pub struct DelimitedFileSource {
    reader: csv::Reader<BufReader<File>>,
    headers: Vec<String>,
}

impl DelimitedFileSource {
    pub fn open(path: impl AsRef<Path>, format: DelimitedFormat, has_headers: bool) -> Result<Self, TransferError> {
        let file = File::open(path.as_ref()).map_err(|e| TransferError::Source(e.to_string()))?;
        let mut builder = csv::ReaderBuilder::new();
        builder.delimiter(format.delimiter()).has_headers(has_headers);
        let mut reader = builder.from_reader(BufReader::new(file));
        let headers = if has_headers {
            reader
                .headers()
                .map_err(|e| TransferError::Source(e.to_string()))?
                .iter()
                .map(str::to_owned)
                .collect()
        } else {
            Vec::new()
        };
        Ok(Self { reader, headers })
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Peek first `limit` rows for import preview (rewinds by re-open is caller's job;
    /// this consumes from the current position).
    pub fn preview_rows(&mut self, limit: usize) -> Result<Vec<TransferRow>, TransferError> {
        let mut rows = Vec::new();
        for result in self.reader.records().take(limit) {
            let record = result.map_err(|e| TransferError::Source(e.to_string()))?;
            rows.push(TransferRow {
                cells: record.iter().map(str::to_owned).collect(),
            });
        }
        Ok(rows)
    }
}

impl TransferSource for DelimitedFileSource {
    fn next_batch(&mut self, max_rows: usize) -> Result<Option<Vec<TransferRow>>, TransferError> {
        let mut batch = Vec::with_capacity(max_rows);
        for _ in 0..max_rows {
            match self.reader.records().next() {
                Some(Ok(record)) => batch.push(TransferRow {
                    cells: record.iter().map(str::to_owned).collect(),
                }),
                Some(Err(err)) => return Err(TransferError::Source(err.to_string())),
                None => break,
            }
        }
        if batch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(batch))
        }
    }
}

/// Streaming delimited writer that publishes atomically (temp → rename).
pub struct DelimitedFileTarget {
    writer: Option<csv::Writer<BufWriter<File>>>,
    final_path: PathBuf,
    temp_path: PathBuf,
    wrote_header: bool,
    headers: Vec<String>,
}

impl DelimitedFileTarget {
    pub fn create(
        path: impl AsRef<Path>,
        format: DelimitedFormat,
        headers: Vec<String>,
    ) -> Result<Self, TransferError> {
        let final_path = path.as_ref().to_path_buf();
        let temp_path = final_path.with_extension(format!("{}.partial", format.extension()));
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent).map_err(|e| TransferError::Target(e.to_string()))?;
        }
        let file = File::create(&temp_path).map_err(|e| TransferError::Target(e.to_string()))?;
        let mut builder = csv::WriterBuilder::new();
        builder.delimiter(format.delimiter());
        let writer = builder.from_writer(BufWriter::new(file));
        Ok(Self {
            writer: Some(writer),
            final_path,
            temp_path,
            wrote_header: false,
            headers,
        })
    }
}

impl TransferTarget for DelimitedFileTarget {
    fn write_batch(&mut self, rows: &[TransferRow]) -> Result<u64, TransferError> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| TransferError::Target("writer already finished".into()))?;
        if !self.wrote_header && !self.headers.is_empty() {
            writer
                .write_record(&self.headers)
                .map_err(|e| TransferError::Target(e.to_string()))?;
            self.wrote_header = true;
        }
        for row in rows {
            writer
                .write_record(&row.cells)
                .map_err(|e| TransferError::Target(e.to_string()))?;
        }
        writer.flush().map_err(|e| TransferError::Target(e.to_string()))?;
        Ok(rows.len() as u64)
    }

    fn finish(&mut self) -> Result<(), TransferError> {
        if let Some(mut writer) = self.writer.take() {
            writer.flush().map_err(|e| TransferError::Target(e.to_string()))?;
            drop(writer);
        }
        fs::rename(&self.temp_path, &self.final_path).map_err(|e| TransferError::Target(e.to_string()))?;
        Ok(())
    }

    fn cleanup_partial(&mut self) -> Result<(), TransferError> {
        self.writer.take();
        let _ = fs::remove_file(&self.temp_path);
        let _ = fs::remove_file(&self.final_path);
        Ok(())
    }
}

/// Serialize provider cell text for CSV (quotes handled by csv crate).
pub fn csv_cell_from_display(value: &str, treat_empty_as_null_token: bool) -> String {
    if treat_empty_as_null_token && value.is_empty() {
        String::new()
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::transfer_service::{TransferService, TransferSource};
    use crate::domain::transfer::{
        TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
    };
    use std::io::Write;

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-transfer-{}-{}", std::process::id(), name));
        path
    }

    #[test]
    fn csv_round_trip_streams_without_materializing_all_at_once() {
        let path = temp_path("out.csv");
        let _ = fs::remove_file(&path);
        let rows = 2_500u64;
        let mut job = TransferJob {
            id: "csv-1".into(),
            label: "csv export".into(),
            source: TransferSourceKind::Synthetic { rows },
            target: TransferTargetKind::File {
                path: path.to_string_lossy().into_owned(),
                format: "csv".into(),
            },
            mapping: Default::default(),
            batch_size: 100,
            status: TransferStatus::Pending,
            progress: Default::default(),
            error: None,
        };

        let mut source = crate::application::SyntheticSource::new(rows);
        let mut target =
            DelimitedFileTarget::create(&path, DelimitedFormat::Csv, vec!["id".into(), "name".into()]).unwrap();
        let cancel = TransferCancellation::new();
        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        assert_eq!(result.status, TransferStatus::Succeeded);
        assert!(path.exists());
        let partial = path.with_extension("csv.partial");
        assert!(!partial.exists());

        let mut reader = DelimitedFileSource::open(&path, DelimitedFormat::Csv, true).unwrap();
        assert_eq!(reader.headers(), &["id".to_owned(), "name".to_owned()]);
        let mut read = 0u64;
        while let Some(batch) = reader.next_batch(200).unwrap() {
            read += batch.len() as u64;
            assert!(batch.len() <= 200);
        }
        assert_eq!(read, rows);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn cancel_removes_partial_csv_output() {
        let path = temp_path("partial.csv");
        let _ = fs::remove_file(&path);
        let mut job = TransferJob::new_synthetic("csv-cancel", 10_000, 50);
        job.target = TransferTargetKind::File {
            path: path.to_string_lossy().into_owned(),
            format: "csv".into(),
        };
        let mut source = crate::application::SyntheticSource::new(10_000);
        let mut target =
            DelimitedFileTarget::create(&path, DelimitedFormat::Csv, vec!["id".into(), "name".into()]).unwrap();
        let cancel = TransferCancellation::new();
        let first = source.next_batch(50).unwrap().unwrap();
        job.progress.rows_read += first.len() as u64;
        let _ = target.write_batch(&first).unwrap();
        cancel.cancel();
        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        assert_eq!(result.status, TransferStatus::Cancelled);
        assert!(!path.exists());
        assert!(!path.with_extension("csv.partial").exists());
    }

    #[test]
    fn malformed_row_surfaces_actionable_error() {
        let path = temp_path("bad.csv");
        {
            let mut f = File::create(&path).unwrap();
            writeln!(f, "a,b").unwrap();
            writeln!(f, "\"broken,1").unwrap();
        }
        let mut source = DelimitedFileSource::open(&path, DelimitedFormat::Csv, true).unwrap();
        let err = source.next_batch(10).unwrap_err();
        assert!(matches!(err, TransferError::Source(_)));
        let msg = err.to_string().to_ascii_lowercase();
        assert!(
            msg.contains("csv") || msg.contains("quote") || msg.contains("utf") || msg.contains("end of file"),
            "unexpected error: {msg}"
        );
        let _ = fs::remove_file(&path);
    }
}
