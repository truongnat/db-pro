//! Tests for `../delimited_transfer.rs` (moved out of the source file).
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
        options: Default::default(),
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

// #429: a delimited writer must defuse spreadsheet formulas — database text starting with
// `=`, `+`, `-`, `@`, tab, CR, newline or space gets the `'` literal prefix.
#[test]
fn guard_csv_field_prefixes_formula_leading_values() {
    use std::borrow::Cow;

    for (input, expected) in [
        (
            "=HYPERLINK(\"http://evil\",\"x\")",
            "'=HYPERLINK(\"http://evil\",\"x\")",
        ),
        ("+1+1", "'+1+1"),
        ("-1+1", "'-1+1"),
        ("@SUM(A1)", "'@SUM(A1)"),
        ("\tcmd", "'\tcmd"),
        ("\rcmd", "'\rcmd"),
        (" lead", "' lead"),
    ] {
        assert_eq!(
            guard_csv_field(input),
            Cow::<str>::Owned(expected.to_owned()),
            "{input:?}"
        );
    }
    // Real numeric literals stay numeric; plain text is untouched.
    for input in ["-42", "+1.5", "-3.14e2", "plain", ""] {
        assert_eq!(guard_csv_field(input), Cow::<str>::Borrowed(input), "{input:?}");
    }
}

#[test]
fn delimited_target_sanitizes_formula_cells() {
    let path = temp_path("formula.csv");
    let _ = fs::remove_file(&path);

    let mut target =
        DelimitedFileTarget::create(&path, DelimitedFormat::Csv, vec!["id".into(), "payload".into()]).unwrap();
    target
        .write_batch(&[TransferRow {
            cells: vec!["1".into(), "=cmd|'/c calc'!A1".into()],
        }])
        .unwrap();
    target.finish().unwrap();

    let body = fs::read_to_string(&path).unwrap();
    assert!(
        body.contains("'=cmd|'/c calc'!A1"),
        "formula-leading cell must be prefixed: {body}"
    );
    let _ = fs::remove_file(&path);
}
