//! Tests for `../json_excel_transfer.rs` (moved out of the source file).
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

#[test]
fn cleanup_preserves_existing_jsonl_and_excel_destinations() {
    let jsonl_path = temp_path("preserve.jsonl");
    let excel_path = temp_path("preserve.xlsx");
    let jsonl_original = b"previous jsonl export";
    let excel_original = b"previous excel export";
    fs::write(&jsonl_path, jsonl_original).unwrap();
    fs::write(&excel_path, excel_original).unwrap();

    let mut jsonl = JsonlFileTarget::create(&jsonl_path, vec!["id".into()]).unwrap();
    let mut excel = ExcelFileTarget::create(&excel_path, "Export", vec!["id".into()]).unwrap();
    jsonl.cleanup_partial().unwrap();
    excel.cleanup_partial().unwrap();

    assert_eq!(fs::read(&jsonl_path).unwrap(), jsonl_original);
    assert_eq!(fs::read(&excel_path).unwrap(), excel_original);
    let _ = fs::remove_file(jsonl_path);
    let _ = fs::remove_file(excel_path);
}
