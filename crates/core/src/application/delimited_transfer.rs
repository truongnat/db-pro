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
#[path = "tests/delimited_transfer_tests.rs"]
mod tests;
