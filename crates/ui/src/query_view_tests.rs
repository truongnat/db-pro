mod egress_tests {
    use super::super::*;

    /// Every text run the frame actually painted, with the editor actions menu open.
    fn rendered_editor_actions_texts(app: &mut DbProApp) -> Vec<String> {
        fn collect(shape: &egui::Shape, texts: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(text) => texts.push(text.galley.text().to_owned()),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        collect(shape, texts);
                    }
                }
                _ => {}
            }
        }

        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let output = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = app.draw_query_editor_actions(ui);
            });
        });

        let mut texts = Vec::new();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut texts);
        }
        texts
    }

    #[test]
    fn ai_prediction_control_discloses_its_egress() {
        let mut app = DbProApp::default();

        let texts = rendered_editor_actions_texts(&mut app);

        assert!(
            texts.iter().any(|text| text == AI_PREDICTION_EGRESS_NOTE),
            "the AI prediction control must state that it sends SQL and schema context (#242); painted texts: {texts:?}"
        );
        assert!(
            texts.iter().any(|text| text == "AI prediction"),
            "the prediction mode control itself is still painted; painted texts: {texts:?}"
        );
    }

    #[test]
    fn prediction_default_is_subtle_with_the_note_visible() {
        // Idle Query must stay snappy: Subtle keeps ghost text opt-in until reveal,
        // while the egress note still discloses the AI path when prediction runs.
        let app = DbProApp::default();

        assert_eq!(app.prediction_mode, PredictionMode::Subtle);
        assert!(AI_PREDICTION_EGRESS_NOTE.contains("configured AI provider"));
    }
}

mod export_tests {
    use super::super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("db-pro-export-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn dir_entries(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("read temp dir")
            .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn atomic_write_publishes_the_whole_file_and_leaves_no_scratch_file() {
        let dir = temp_dir("publish");
        let path = dir.join("out.csv");

        write_file_atomically(&path, b"id,name\n1,alpha\n").expect("write");

        assert_eq!(std::fs::read_to_string(&path).expect("read"), "id,name\n1,alpha\n");
        assert_eq!(
            dir_entries(&dir),
            vec!["out.csv".to_owned()],
            "the temp file must not survive a successful write"
        );
    }

    /// The regression #244 (E-1) describes: a write that cannot complete must leave **no** truncated
    /// file at the destination. The directory is made read-only so the temp file cannot be created —
    /// the destination is then untouched, where `std::fs::write` would have failed mid-stream.
    #[cfg(unix)]
    #[test]
    fn failed_atomic_write_leaves_the_destination_untouched() {
        let dir = temp_dir("failure");
        let path = dir.join("out.csv");
        std::fs::write(&path, "previous export\n").expect("seed existing file");

        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).expect("chmod read-only");
        let error = write_file_atomically(&path, b"id,name\n1,alpha\n")
            .expect_err("a write into a read-only directory must fail");
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).expect("chmod back");

        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            "previous export\n",
            "the destination must keep its previous content, never a truncated prefix"
        );
        assert_eq!(
            dir_entries(&dir),
            vec!["out.csv".to_owned()],
            "no scratch file may survive"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_atomic_write_into_a_missing_directory_creates_nothing() {
        let dir = temp_dir("missing");
        let path = dir.join("no-such-dir").join("out.csv");

        write_file_atomically(&path, b"id,name\n").expect_err("the parent does not exist");

        assert!(!path.exists());
        assert_eq!(dir_entries(&dir), Vec::<String>::new(), "nothing may be created");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Pins that the **export path** publishes rather than rewrites in place — the property the
    /// acceptance cares about ("no partial file survives"), which a test of the helper alone does not
    /// establish: reverting `export_result_to_disk` to `std::fs::write` leaves a helper-only suite
    /// green. An in-place rewrite truncates the destination before writing into it; a publish
    /// replaces it, so the destination is a different file afterwards even though the path is the
    /// same.
    #[cfg(unix)]
    #[test]
    fn export_replaces_the_destination_instead_of_truncating_it() {
        use std::os::unix::fs::MetadataExt as _;

        let dir = temp_dir("publish-semantics");
        let path = dir.join("out.csv");
        std::fs::write(&path, "previous export\n").expect("seed existing file");
        let inode_before = std::fs::metadata(&path).expect("metadata").ino();

        let mut app = DbProApp {
            export_open: true,
            export_path: path.to_string_lossy().into_owned(),
            export_format: "CSV".to_owned(),
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1".to_owned())]],
            row_count: 1,
            duration_ms: 0,
        };

        app.export_result(&result);
        app.export_result_confirming_overwrite(&result);

        assert_eq!(std::fs::read_to_string(&path).expect("read"), "id\n1\n");
        assert_ne!(
            std::fs::metadata(&path).expect("metadata").ino(),
            inode_before,
            "the destination must be replaced by a rename, not rewritten in place: an in-place write \
             is what leaves a truncated file when it fails part-way"
        );
        assert_eq!(dir_entries(&dir), vec!["out.csv".to_owned()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_refuses_to_replace_an_existing_file_without_confirmation() {
        let dir = temp_dir("confirm");
        let path = dir.join("out.csv");
        std::fs::write(&path, "previous export\n").expect("seed existing file");
        let mut app = DbProApp {
            export_open: true,
            export_path: path.to_string_lossy().into_owned(),
            export_format: "CSV".to_owned(),
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1".to_owned())]],
            row_count: 1,
            duration_ms: 0,
        };

        app.export_result(&result);

        assert!(app.export_overwrite_pending, "the first click must ask, not overwrite");
        assert!(app.export_open, "the dialog stays open for the confirmation");
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            "previous export\n",
            "nothing may be written before the user confirms"
        );

        app.export_result_confirming_overwrite(&result);

        assert!(!app.export_overwrite_pending);
        assert!(!app.export_open);
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "id\n1\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_message_names_a_capped_result_as_capped() {
        let dir = temp_dir("capped");
        let path = dir.join("out.csv");
        let mut app = DbProApp {
            export_open: true,
            export_path: path.to_string_lossy().into_owned(),
            export_format: "CSV".to_owned(),
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1".to_owned())]],
            row_count: 500,
            duration_ms: 0,
        };

        app.export_result(&result);

        assert!(
            app.runtime_message.contains("Exported 1 rows") && app.runtime_message.contains("500 rows matched"),
            "a capped export must not read as a complete one: {}",
            app.runtime_message
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
