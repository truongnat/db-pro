//! JSONL + Excel transfer adapters (#195).

use crate::domain::transfer::{TransferError, TransferRow};
use rust_xlsxwriter::{Format, Workbook};
use serde_json::{Map, Value};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use super::transfer_service::{TransferSource, TransferTarget};

/// Streaming JSON Lines writer (one JSON object per row).
pub struct JsonlFileTarget {
    writer: Option<BufWriter<File>>,
    final_path: PathBuf,
    temp_path: PathBuf,
    headers: Vec<String>,
}

impl JsonlFileTarget {
    pub fn create(path: impl AsRef<Path>, headers: Vec<String>) -> Result<Self, TransferError> {
        let final_path = path.as_ref().to_path_buf();
        let temp_path = final_path.with_extension("jsonl.partial");
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent).map_err(|e| TransferError::Target(e.to_string()))?;
        }
        let file = File::create(&temp_path).map_err(|e| TransferError::Target(e.to_string()))?;
        Ok(Self {
            writer: Some(BufWriter::new(file)),
            final_path,
            temp_path,
            headers,
        })
    }
}

impl TransferTarget for JsonlFileTarget {
    fn write_batch(&mut self, rows: &[TransferRow]) -> Result<u64, TransferError> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| TransferError::Target("writer finished".into()))?;
        for row in rows {
            let mut map = Map::new();
            for (idx, cell) in row.cells.iter().enumerate() {
                let key = self.headers.get(idx).cloned().unwrap_or_else(|| format!("col_{idx}"));
                map.insert(key, Value::String(cell.clone()));
            }
            let line = serde_json::to_string(&Value::Object(map)).map_err(|e| TransferError::Target(e.to_string()))?;
            writeln!(writer, "{line}").map_err(|e| TransferError::Target(e.to_string()))?;
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

/// Streaming JSONL reader.
pub struct JsonlFileSource {
    lines: std::io::Lines<BufReader<File>>,
    headers: Vec<String>,
}

impl JsonlFileSource {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, TransferError> {
        let file = File::open(path.as_ref()).map_err(|e| TransferError::Source(e.to_string()))?;
        let mut lines = BufReader::new(file).lines();
        let headers = match lines.next() {
            Some(Ok(first)) => {
                let value: Value = serde_json::from_str(&first).map_err(|e| TransferError::Source(e.to_string()))?;
                let headers = match value.as_object() {
                    Some(map) => map.keys().cloned().collect(),
                    None => return Err(TransferError::Source("JSONL first line must be an object".into())),
                };
                // Re-open to include first line in iteration.
                let file = File::open(path.as_ref()).map_err(|e| TransferError::Source(e.to_string()))?;
                return Ok(Self {
                    lines: BufReader::new(file).lines(),
                    headers,
                });
            }
            Some(Err(err)) => return Err(TransferError::Source(err.to_string())),
            None => Vec::new(),
        };
        Ok(Self { lines, headers })
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}

impl TransferSource for JsonlFileSource {
    fn next_batch(&mut self, max_rows: usize) -> Result<Option<Vec<TransferRow>>, TransferError> {
        let mut batch = Vec::with_capacity(max_rows);
        for _ in 0..max_rows {
            match self.lines.next() {
                Some(Ok(line)) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let value: Value = serde_json::from_str(line).map_err(|e| TransferError::Source(e.to_string()))?;
                    let obj = value
                        .as_object()
                        .ok_or_else(|| TransferError::Source("JSONL row must be an object".into()))?;
                    let cells = if self.headers.is_empty() {
                        obj.values().map(|v| json_value_to_cell(v)).collect()
                    } else {
                        self.headers
                            .iter()
                            .map(|h| obj.get(h).map(json_value_to_cell).unwrap_or_default())
                            .collect()
                    };
                    batch.push(TransferRow { cells });
                }
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

fn json_value_to_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Excel target: buffers worksheet rows then writes workbook on finish.
/// Source still streams in batches; workbook materialization is XLSX-format limited.
pub struct ExcelFileTarget {
    rows: Vec<TransferRow>,
    headers: Vec<String>,
    sheet_name: String,
    final_path: PathBuf,
    temp_path: PathBuf,
}

impl ExcelFileTarget {
    pub fn create(
        path: impl AsRef<Path>,
        sheet_name: impl Into<String>,
        headers: Vec<String>,
    ) -> Result<Self, TransferError> {
        let final_path = path.as_ref().to_path_buf();
        let temp_path = final_path.with_extension("xlsx.partial");
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent).map_err(|e| TransferError::Target(e.to_string()))?;
        }
        Ok(Self {
            rows: Vec::new(),
            headers,
            sheet_name: sheet_name.into(),
            final_path,
            temp_path,
        })
    }
}

impl TransferTarget for ExcelFileTarget {
    fn write_batch(&mut self, rows: &[TransferRow]) -> Result<u64, TransferError> {
        self.rows.extend(rows.iter().cloned());
        Ok(rows.len() as u64)
    }

    fn finish(&mut self) -> Result<(), TransferError> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&self.sheet_name)
            .map_err(|e| TransferError::Target(e.to_string()))?;
        let header_format = Format::new().set_bold();
        for (col, header) in self.headers.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, header, &header_format)
                .map_err(|e| TransferError::Target(e.to_string()))?;
        }
        for (row_idx, row) in self.rows.iter().enumerate() {
            for (col, cell) in row.cells.iter().enumerate() {
                worksheet
                    .write_string((row_idx + 1) as u32, col as u16, cell)
                    .map_err(|e| TransferError::Target(e.to_string()))?;
            }
        }
        workbook
            .save(&self.temp_path)
            .map_err(|e| TransferError::Target(e.to_string()))?;
        fs::rename(&self.temp_path, &self.final_path).map_err(|e| TransferError::Target(e.to_string()))?;
        Ok(())
    }

    fn cleanup_partial(&mut self) -> Result<(), TransferError> {
        self.rows.clear();
        let _ = fs::remove_file(&self.temp_path);
        let _ = fs::remove_file(&self.final_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{SyntheticSource, TransferService};
    use crate::domain::transfer::{TransferCancellation, TransferJob, TransferStatus};

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-json-excel-{}-{}", std::process::id(), name));
        path
    }

    #[test]
    fn jsonl_round_trip_streams() {
        let path = temp_path("data.jsonl");
        let _ = fs::remove_file(&path);
        let mut job = TransferJob::new_synthetic("jsonl", 500, 50);
        let mut source = SyntheticSource::new(500);
        let mut target = JsonlFileTarget::create(&path, vec!["id".into(), "name".into()]).unwrap();
        let cancel = TransferCancellation::new();
        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        assert_eq!(result.status, TransferStatus::Succeeded);
        let mut reader = JsonlFileSource::open(&path).unwrap();
        let mut n = 0u64;
        while let Some(batch) = reader.next_batch(80).unwrap() {
            n += batch.len() as u64;
        }
        assert_eq!(n, 500);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn excel_export_writes_workbook() {
        let path = temp_path("data.xlsx");
        let _ = fs::remove_file(&path);
        let mut job = TransferJob::new_synthetic("xlsx", 120, 40);
        let mut source = SyntheticSource::new(120);
        let mut target = ExcelFileTarget::create(&path, "Export", vec!["id".into(), "name".into()]).unwrap();
        let cancel = TransferCancellation::new();
        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        assert_eq!(result.status, TransferStatus::Succeeded);
        assert!(path.exists());
        assert!(fs::metadata(&path).unwrap().len() > 0);
        let _ = fs::remove_file(&path);
    }
}
