use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use db_pro_ui::{filtered_sorted_indexes, UiCell, UiColumn, UiQueryResult};
use std::collections::HashMap;

fn million_row_result() -> UiQueryResult {
    UiQueryResult {
        columns: vec![
            UiColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
            },
            UiColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
            },
            UiColumn {
                name: "status".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
            },
        ],
        rows: (0..1_000_000)
            .map(|index| {
                vec![
                    UiCell::Number(index.to_string()),
                    UiCell::Text(format!("customer-{index}")),
                    UiCell::Text(if index % 2 == 0 { "active" } else { "idle" }.to_owned()),
                ]
            })
            .collect(),
        row_count: 1_000_000,
        duration_ms: 0,
    }
}

fn bench_million_row_metadata(c: &mut Criterion) {
    let result = million_row_result();
    let mut group = c.benchmark_group("result_grid_million_rows");
    group.throughput(Throughput::Elements(1_000_000));
    group.sample_size(10);
    group.bench_function("project_without_filter_or_sort", |b| {
        b.iter(|| {
            black_box(filtered_sorted_indexes(black_box(&result), "", None, false));
        });
    });
    group.finish();
}

/// Sorted projection on a 200k-row x 4-column result: the fixture the sort issue measured
/// (`docs/release/evidence/v01-runtime/providers/53-result-grid-projection-cache.md`).
///
/// The temporal case is the one that reaches seconds: its values parse, so the comparator runs the
/// date/time parsers on both operands for every comparison. That cost is why the draw path must not
/// rebuild this projection per frame (`GridProjectionCache`).
fn sortable_result(row_count: usize) -> UiQueryResult {
    UiQueryResult {
        columns: (0..4)
            .map(|index| UiColumn {
                name: format!("column_{index}"),
                data_type: "text".to_owned(),
                nullable: true,
            })
            .collect(),
        rows: (0..row_count)
            .map(|index| {
                vec![
                    UiCell::Number(index.to_string()),
                    UiCell::Text(format!("customer-{index:06}")),
                    UiCell::Text(format!(
                        "2026-01-{:02} 12:{:02}:{:02}",
                        index % 28 + 1,
                        index % 60,
                        index % 60
                    )),
                    UiCell::Text(if index % 2 == 0 { "active" } else { "idle" }.to_owned()),
                ]
            })
            .collect(),
        row_count: row_count as u64,
        duration_ms: 0,
    }
}

fn bench_sorted_projection(c: &mut Criterion) {
    let result = sortable_result(200_000);
    let mut group = c.benchmark_group("result_grid_projection_sorted");
    group.throughput(Throughput::Elements(200_000));
    group.sample_size(10);
    group.bench_function("sort_plain_text_column", |b| {
        b.iter(|| {
            black_box(filtered_sorted_indexes(black_box(&result), "", Some(1), false));
        });
    });
    group.bench_function("sort_temporal_text_column", |b| {
        b.iter(|| {
            black_box(filtered_sorted_indexes(black_box(&result), "", Some(2), false));
        });
    });
    group.bench_function("filter_and_sort", |b| {
        b.iter(|| {
            black_box(filtered_sorted_indexes(
                black_box(&result),
                "customer-0001",
                Some(1),
                false,
            ));
        });
    });
    group.finish();
}

fn bench_visible_scroll_window(c: &mut Criterion) {
    let result = million_row_result();
    let indexes = filtered_sorted_indexes(&result, "", None, false);
    let mut group = c.benchmark_group("result_grid_scroll_window");
    group.throughput(Throughput::Elements(100));
    group.bench_function("materialize_100_visible_rows", |b| {
        b.iter(|| {
            let start = black_box(500_000usize);
            let end = (start + 100).min(indexes.len());
            let visible_cells = indexes[start..end]
                .iter()
                .map(|row_index| result.rows[*row_index].len())
                .sum::<usize>();
            black_box(visible_cells);
        });
    });
    group.finish();
}

fn grid_result(row_count: usize, column_count: usize) -> UiQueryResult {
    UiQueryResult {
        columns: (0..column_count)
            .map(|index| UiColumn {
                name: format!("column_{index}"),
                data_type: "text".to_owned(),
                nullable: true,
            })
            .collect(),
        rows: (0..row_count)
            .map(|row_index| {
                (0..column_count)
                    .map(|column_index| UiCell::Text(format!("r{row_index}c{column_index}")))
                    .collect()
            })
            .collect(),
        row_count: row_count as u64,
        duration_ms: 0,
    }
}

fn bench_requested_grid_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("result_grid_requested_sizes");
    for row_count in [1_000, 10_000] {
        let result = grid_result(row_count, 50);
        let indexes = (0..row_count).collect::<Vec<_>>();
        group.throughput(Throughput::Elements((row_count * 50) as u64));
        group.bench_function(format!("build_visual_maps_{row_count}_rows_50_columns"), |b| {
            b.iter(|| {
                let row_positions: HashMap<usize, usize> = indexes
                    .iter()
                    .enumerate()
                    .map(|(visual_row, &row_index)| (row_index, visual_row))
                    .collect();
                let column_positions: HashMap<usize, usize> = result
                    .columns
                    .iter()
                    .enumerate()
                    .map(|(visual_column, _)| (visual_column, visual_column))
                    .collect();
                black_box((row_positions.len(), column_positions.len()));
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_million_row_metadata,
    bench_sorted_projection,
    bench_visible_scroll_window,
    bench_requested_grid_sizes
);
criterion_main!(benches);
