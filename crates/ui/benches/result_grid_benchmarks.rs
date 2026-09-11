use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use db_pro_ui::{filtered_sorted_indexes, UiCell, UiColumn, UiQueryResult};

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

criterion_group!(benches, bench_million_row_metadata, bench_visible_scroll_window);
criterion_main!(benches);
