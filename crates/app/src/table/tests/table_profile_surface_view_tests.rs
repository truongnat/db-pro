//! Tests for `../table_profile_surface_view.rs` (moved out of the source file).
    use super::*;
    use crate::UiColumn;

    #[test]
    fn long_numeric_summary_is_rendered_without_ellipsis_or_cell_clipping() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let result = UiQueryResult {
            columns: vec![UiColumn {
                name: "id".into(),
                data_type: "BIGINT".into(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1000000000000000000".into())]; 3],
            duration_ms: 0,
            row_count: 3,
        };
        let profiles = profile_result_columns(&result);
        let expected = format!("{:.2} / {:.1}", profiles[0].avg.unwrap(), profiles[0].sum.unwrap());
        let output = crate::test_frame::frame(&ctx, egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(2000.0, 500.0))),
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| draw_profile_grid(DbProTheme::light(), ui, &profiles, 3));
        });
        let summary = output
            .shapes
            .iter()
            .find_map(|clipped| {
                if let egui::Shape::Text(text) = &clipped.shape {
                    if text.galley.job.text == expected {
                        return Some((clipped.clip_rect, text));
                    }
                }
                None
            })
            .expect("full numeric summary must be rendered, without ellipsis");
        assert!(
            summary
                .0
                .contains_rect(egui::Rect::from_min_size(summary.1.pos, summary.1.galley.size())),
            "summary must fit its cell clip"
        );
    }

    #[test]
    fn test_profile_result_numeric_and_uniqueness() {
        let result = UiQueryResult {
            columns: vec![
                UiColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
                UiColumn {
                    name: "amount".to_owned(),
                    data_type: "numeric".to_owned(),
                    nullable: true,
                },
            ],
            rows: vec![
                vec![UiCell::Number("1".to_owned()), UiCell::Number("10.5".to_owned())],
                vec![UiCell::Number("2".to_owned()), UiCell::Number("20.5".to_owned())],
                vec![UiCell::Number("3".to_owned()), UiCell::Null],
            ],
            duration_ms: 5,
            row_count: 3,
        };

        let profiles = profile_result_columns(&result);
        assert_eq!(profiles.len(), 2);

        // id column
        let id_prof = &profiles[0];
        assert_eq!(id_prof.distinct_count, 3);
        assert_eq!(id_prof.null_count, 0);
        assert_eq!(id_prof.sum, Some(6.0));
        assert_eq!(id_prof.avg, Some(2.0));

        // amount column
        let amount_prof = &profiles[1];
        assert_eq!(amount_prof.distinct_count, 2);
        assert_eq!(amount_prof.null_count, 1);
        assert_eq!(amount_prof.sum, Some(31.0));
        assert_eq!(amount_prof.avg, Some(15.5));
    }

    #[test]
    fn all_null_column_has_a_distinct_pattern_label() {
        let result = UiQueryResult {
            columns: vec![UiColumn {
                name: "optional_value".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            }],
            rows: vec![vec![UiCell::Null], vec![UiCell::Null]],
            duration_ms: 0,
            row_count: 2,
        };
        let profile = &profile_result_columns(&result)[0];

        assert_eq!(profile.null_count, 2);
        assert_eq!(profile.distinct_count, 0);
        assert_eq!(profile_pattern(DbProTheme::dark(), profile, 0).0, "All null");
    }