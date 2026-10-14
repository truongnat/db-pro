//! Tests for `../chart_engine.rs` (moved out of the source file).
use super::*;
use crate::{UiCell, UiColumn};

#[test]
fn is_numeric_column_recognizes_common_types() {
    assert!(ChartEngine::is_numeric_column("INTEGER"));
    assert!(ChartEngine::is_numeric_column("numeric(20,4)"));
    assert!(!ChartEngine::is_numeric_column("text"));
    assert!(!ChartEngine::is_numeric_column("jsonb"));
}

#[test]
fn project_defaults_y_to_first_numeric_column_not_index_one() {
    // Regression: y_column=None meant index 1 regardless of type — when that
    // column was text every row was skipped and the chart stayed empty while
    // the picker displayed a numeric column.
    let columns = vec![
        UiColumn {
            name: "id".into(),
            data_type: "bigint".into(),
            nullable: false,
        },
        UiColumn {
            name: "label".into(),
            data_type: "text".into(),
            nullable: true,
        },
        UiColumn {
            name: "hits".into(),
            data_type: "integer".into(),
            nullable: false,
        },
    ];
    let rows = vec![vec![
        UiCell::Number("1".into()),
        UiCell::Text("a".into()),
        UiCell::Number("5".into()),
    ]];
    let projection = ChartEngine::project(&columns, &rows, &ChartConfig::new());
    assert_eq!(projection.points.len(), 1);
    assert_eq!(projection.points[0].y, 5.0);
}

#[test]
fn project_downsamples_to_max_points() {
    let columns = vec![
        UiColumn {
            name: "x".into(),
            data_type: "integer".into(),
            nullable: false,
        },
        UiColumn {
            name: "y".into(),
            data_type: "integer".into(),
            nullable: false,
        },
    ];
    let rows: Vec<Vec<UiCell>> = (0..50)
        .map(|i| vec![UiCell::Number(i.to_string()), UiCell::Number((i * 2).to_string())])
        .collect();
    let mut config = ChartConfig::new();
    config.x_column = Some(0);
    config.y_column = Some(1);
    config.max_points = 10;
    let points = ChartEngine::project(&columns, &rows, &config);
    assert!(
        points.points.len() <= 10,
        "expected downsample, got {}",
        points.points.len()
    );
}

#[test]
fn project_aggregates_before_downsample_and_reports_null_skips() {
    let columns = vec![
        UiColumn {
            name: "category".into(),
            data_type: "text".into(),
            nullable: true,
        },
        UiColumn {
            name: "value".into(),
            data_type: "integer".into(),
            nullable: true,
        },
        UiColumn {
            name: "series".into(),
            data_type: "text".into(),
            nullable: false,
        },
    ];
    let rows = vec![
        vec![
            UiCell::Text("a".into()),
            UiCell::Number("1".into()),
            UiCell::Text("s1".into()),
        ],
        vec![
            UiCell::Text("a".into()),
            UiCell::Number("3".into()),
            UiCell::Text("s1".into()),
        ],
        vec![UiCell::Null, UiCell::Number("9".into()), UiCell::Text("s2".into())],
        vec![UiCell::Text("b".into()), UiCell::Null, UiCell::Text("s2".into())],
    ];
    let mut config = ChartConfig::new();
    config.x_column = Some(0);
    config.y_column = Some(1);
    config.series_column = Some(2);
    config.aggregation = ChartAggregation::Sum;
    config.max_points = 1;
    let projection = ChartEngine::project(&columns, &rows, &config);
    assert_eq!(projection.skipped_null_y, 1);
    assert_eq!(projection.x_fallback_to_index, 1);
    assert_eq!(projection.points.len(), 1, "downsample after aggregate");
    assert!(!projection.points[0].series.is_empty());
}

#[test]
fn project_pie_reports_null_skips() {
    let columns = vec![
        UiColumn {
            name: "label".into(),
            data_type: "text".into(),
            nullable: true,
        },
        UiColumn {
            name: "value".into(),
            data_type: "integer".into(),
            nullable: true,
        },
    ];
    let rows = vec![
        vec![UiCell::Text("a".into()), UiCell::Number("2".into())],
        vec![UiCell::Null, UiCell::Number("3".into())],
        vec![UiCell::Text("b".into()), UiCell::Null],
    ];
    let mut config = ChartConfig::new();
    config.chart_type = ChartType::Pie;
    config.x_column = Some(0);
    config.y_column = Some(1);
    let projection = ChartEngine::project_pie(&columns, &rows, &config);
    assert_eq!(projection.skipped_null_y, 1);
    assert_eq!(projection.x_fallback_to_index, 1);
    assert!(projection.points.iter().any(|p| p.label == "NULL"));
}

#[test]
fn project_handles_ten_thousand_rows_within_budget() {
    let columns = vec![
        UiColumn {
            name: "x".into(),
            data_type: "integer".into(),
            nullable: false,
        },
        UiColumn {
            name: "y".into(),
            data_type: "decimal".into(),
            nullable: false,
        },
    ];
    let rows: Vec<Vec<UiCell>> = (0..10_000)
        .map(|i| vec![UiCell::Number(i.to_string()), UiCell::Number(format!("{}.5", i % 1000))])
        .collect();
    let mut config = ChartConfig::new();
    config.x_column = Some(0);
    config.y_column = Some(1);
    config.max_points = 1_000;
    let started = std::time::Instant::now();
    let points = ChartEngine::project(&columns, &rows, &config);
    let elapsed = started.elapsed();
    assert_eq!(points.points.len(), 1_000);
    assert!(
        elapsed.as_millis() < 250,
        "10k-row project took {:?}, expected <250ms",
        elapsed
    );
}
