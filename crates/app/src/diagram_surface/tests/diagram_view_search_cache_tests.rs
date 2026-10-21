//! #544 — tests for the per-key ER search caches in `../diagram_view.rs` and
//! `../diagram_state.rs`, plus the old-vs-new matcher equality and the release
//! frame budget on the issue's 1 300-table × 30-column fixture.

use super::*;
use crate::app::diagram_state::ErSubsetCache;

/// The pre-#544 matcher, kept verbatim as the behavioral reference: allocate a
/// fresh lowercased `String` for the table name, the schema, and every column.
fn matches_diagram_search_reference(table: &UiTableSummary, query: &str) -> bool {
    table.name.to_ascii_lowercase().contains(query)
        || table.schema.to_ascii_lowercase().contains(query)
        || table
            .columns
            .iter()
            .any(|column| column.name.to_ascii_lowercase().contains(query))
}

fn summary(schema: &str, name: &str, columns: &[&str]) -> UiTableSummary {
    UiTableSummary {
        schema: schema.to_owned(),
        name: name.to_owned(),
        row_count: None,
        columns: columns
            .iter()
            .map(|column| crate::runtime::UiSchemaColumn {
                name: (*column).to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
                is_primary_key: false,
            })
            .collect(),
        foreign_keys: Vec::new(),
    }
}

/// Queries exercised against both implementations, including the exact
/// normalized form the draw path produces.
fn corpus() -> Vec<UiTableSummary> {
    vec![
        summary("public", "Customers", &["Id", "customer_NAME", "email"]),
        summary("Public", "Ünïcødé_Ţable", &["Ölde_Id", "straße_col"]),
        summary("dbo", "ORDER_ITEMS", &["customerId", "日本_language"]),
        summary("main", "türk_tablo", &["İstanbul", "ıstanbul", "ẞ_COLUMN"]),
        summary("app", "audit_log", &["payload"]),
        // Empty name column set: schema/name-only tables still match.
        summary("pg", "empty_cols", &[]),
    ]
}

#[test]
fn allocation_free_matcher_agrees_with_legacy_implementation_on_unicode_corpus() {
    let queries = [
        "",
        "customer",
        "CUSTOMER",
        "id",
        "ünïcødé",
        "ÜNL", // case-folded ASCII head + raw non-ASCII byte
        "Ţabl",
        "ölde",
        "straße",
        "日本",
        "İstanbul",
        "ıstanbul",
        "ẞ",
        "ß",
        "column",
        "public",
        "pg",
        "zzz_nomatch",
        "empty_cols",
        "üüü",
    ];
    for table in corpus() {
        for query in queries {
            let query = query.to_ascii_lowercase();
            assert_eq!(
                matches_diagram_search(&table, &query),
                matches_diagram_search_reference(&table, &query),
                "legacy vs allocation-free matcher diverged for {:?} / query {query:?}",
                table.name
            );
        }
    }
}

#[test]
fn listed_count_cache_returns_legacy_counts_and_only_rescans_on_key_change() {
    let tables = corpus();
    let mut cache = ErMatchCountCache::default();

    let legacy = |query: &str| tables.iter().filter(|t| matches_diagram_search_reference(t, query)).count();

    // First frame computes; repeats on the same (generation, count, query) hit.
    let want = legacy("customer").min(6);
    assert_eq!(diagram_listed_count(&mut cache, 1, &tables, Some("customer"), 5), want);
    assert_eq!(cache.recomputes(), 1);
    for _ in 0..8 {
        assert_eq!(
            diagram_listed_count(&mut cache, 1, &tables, Some("customer"), 5),
            legacy("customer").min(6)
        );
    }
    assert_eq!(cache.recomputes(), 1, "identical key must never rescan");

    // Every key dimension invalidates independently.
    diagram_listed_count(&mut cache, 2, &tables, Some("customer"), 5);
    assert_eq!(cache.recomputes(), 2, "design generation bump invalidates");
    diagram_listed_count(&mut cache, 2, &tables, Some("column"), 5);
    assert_eq!(cache.recomputes(), 3, "query change invalidates");
    let mut extended = tables.clone();
    extended.push(summary("app", "extra_table", &["customer_id"]));
    diagram_listed_count(&mut cache, 2, &extended, Some("column"), 5);
    assert_eq!(cache.recomputes(), 4, "catalog length change invalidates");

    // Counts equal the legacy scan for a set of queries, and the render-limit cap
    // matches the old `take(render_limit + 1)` behavior.
    for query in ["customer", "id", "ünïcødé", "zzz_nomatch"] {
        let mut fresh = ErMatchCountCache::default();
        let want = legacy(query).min(6);
        assert_eq!(diagram_listed_count(&mut fresh, 7, &tables, Some(query), 5), want);
    }

    // Empty query in search mode lists nothing and touches no scan at all.
    let mut untouched = ErMatchCountCache::default();
    assert_eq!(diagram_listed_count(&mut untouched, 7, &tables, Some(""), 5), 0);
    assert_eq!(untouched.recomputes(), 0);
    // No search mode: length cap, like the legacy `None` branch.
    assert_eq!(diagram_listed_count(&mut untouched, 7, &tables, None, 2), 3);
}

fn fixture_graph(tables: &[UiTableSummary], schema_version: u64) -> ErGraph {
    ErGraph::build(tables, schema_version, (4usize, 200.0f32))
}

fn medium_catalog() -> Vec<UiTableSummary> {
    (0..40)
        .map(|index| {
            summary(
                if index % 3 == 0 { "app" } else { "public" },
                &format!("orders_{index:03}"),
                &[&format!("customer_ref_{index}"), "created_at", "payload"],
            )
        })
        .collect()
}

#[test]
fn subset_cache_equals_uncached_scan_and_bfs_and_revalidates_per_key() {
    let tables = medium_catalog();
    let graph = fixture_graph(&tables, 3);

    // Reference: exactly what the uncached per-frame pass did.
    let query = "customer_ref";
    let legacy_seeds: Vec<usize> = graph
        .nodes
        .iter()
        .filter(|node| matches_diagram_search_reference(&node.table, query))
        .map(|node| node.id)
        .collect();
    let legacy_subset = graph.bfs_neighborhood(&legacy_seeds, (1, 100));

    let mut cache = ErSubsetCache::default();
    let matched = cache.resolve(&graph, query, 1);
    assert_eq!(matched.seeds, legacy_seeds.as_slice());
    assert_eq!(matched.subset, legacy_subset.as_slice());
    assert_eq!(cache.recomputes(), 1);

    // Repeat frames (pan/zoom repaints): same answer, zero rescans.
    for _ in 0..8 {
        let again = cache.resolve(&graph, query, 1);
        assert_eq!(again.seeds, legacy_seeds.as_slice());
        assert_eq!(again.subset, legacy_subset.as_slice());
    }
    assert_eq!(cache.recomputes(), 1);

    // Depth change invalidates (BFS was run at the old depth).
    let deeper_subset = cache.resolve(&graph, query, 2).subset.to_vec();
    assert_eq!(cache.recomputes(), 2);
    let legacy_deep = graph.bfs_neighborhood(&legacy_seeds, (2, 100));
    assert_eq!(deeper_subset, legacy_deep);

    // Query change invalidates.
    cache.resolve(&graph, "orders_00", 2);
    assert_eq!(cache.recomputes(), 3);

    // New graph (schema_version or node count) invalidates.
    let rebuilt = fixture_graph(&tables, 4);
    cache.resolve(&rebuilt, "orders_00", 2);
    assert_eq!(cache.recomputes(), 4);
    let mut grown = tables.clone();
    grown.push(summary("app", "orders_040", &["customer_ref_40"]));
    let grown_graph = fixture_graph(&grown, 4);
    cache.resolve(&grown_graph, "orders_00", 2);
    assert_eq!(cache.recomputes(), 5, "extra node forces a rebuild");

    // Empty-query neighborhood branch: first three nodes, cap 40 — same as legacy.
    let mut empty_cache = ErSubsetCache::default();
    let empty = empty_cache.resolve(&graph, "", 1);
    let legacy_empty_seeds: Vec<usize> = graph.nodes.iter().take(3).map(|node| node.id).collect();
    let legacy_empty = graph.bfs_neighborhood(&legacy_empty_seeds, (1, 40));
    assert_eq!(empty.seeds, legacy_empty_seeds.as_slice());
    assert_eq!(empty.subset, legacy_empty.as_slice());

    // Empty graph: seeds empty in both branches → the caller shows the
    // building/no-match empty state, like before.
    let empty_graph = ErGraph::default();
    let mut none_cache = ErSubsetCache::default();
    assert!(none_cache.resolve(&empty_graph, "customer", 1).seeds.is_empty());
    assert!(none_cache.resolve(&empty_graph, "", 1).seeds.is_empty());
}

/// #544 measurement: the issue fixture (1 300 tables × 30 columns ≈ 39k names)
/// with search active. `legacy` re-runs the pre-fix per-frame work (allocating
/// lowercase scan of the catalog + graph node scan + BFS) on every repaint;
/// `cached` runs the production passes with the caches. Debug builds only print
/// (instruments add constant overhead); release builds enforce the acceptance
/// budget: < 0.1 ms per cached frame and a ≥ 10× margin vs legacy.
/// One production search-active frame against the caches (listed count pass +
/// seed/BFS pass), exactly what `draw_diagram` runs per repaint.
fn cached_search_frame(
    cache: &mut ErMatchCountCache,
    subset_cache: &mut ErSubsetCache,
    tables: &[UiTableSummary],
    graph: &ErGraph,
    query: &str,
    generation: u64,
) -> (usize, usize) {
    let listed = diagram_listed_count(cache, generation, tables, Some(query), 5);
    let matched = subset_cache.resolve(graph, query, 1);
    (listed, matched.subset.len())
}

#[test]
fn er_search_cached_frames_fit_budget_on_issue_fixture() {
    let tables: Vec<UiTableSummary> = (0..1_300)
        .map(|table| UiTableSummary {
            schema: if table % 2 == 0 { "app" } else { "public" }.to_owned(),
            name: format!("table_{table:05}"),
            row_count: None,
            columns: (0..30)
                .map(|column| crate::runtime::UiSchemaColumn {
                    name: format!("column_{column:05}"),
                    data_type: "text".to_owned(),
                    nullable: true,
                    is_primary_key: false,
                })
                .collect(),
            foreign_keys: Vec::new(),
        })
        .collect();
    let graph = fixture_graph(&tables, 1);
    let query = "zzz_nomatch"; // worst case: every name compared, nothing short-circuits

    let legacy_frame = || {
        let listed = tables
            .iter()
            .filter(|table| matches_diagram_search_reference(table, query))
            .count()
            .min(6);
        let seeds: Vec<usize> = graph
            .nodes
            .iter()
            .filter(|node| matches_diagram_search_reference(&node.table, query))
            .map(|node| node.id)
            .collect();
        let subset = graph.bfs_neighborhood(&seeds, (1, 100));
        (listed, subset.len())
    };

    let mut cache = ErMatchCountCache::default();
    let mut subset_cache = ErSubsetCache::default();

    // The cached pass must produce the same result as the legacy pass. The very
    // first frame is cold: it pays the (now allocation-free) scan once.
    let cold_start = std::time::Instant::now();
    let first = cached_search_frame(&mut cache, &mut subset_cache, &tables, &graph, query, 1);
    let cold = cold_start.elapsed();
    assert_eq!(legacy_frame(), first);
    assert_eq!(cache.recomputes(), 1);
    assert_eq!(subset_cache.recomputes(), 1);

    let frames = 10;
    let mut legacy_best = std::time::Duration::MAX;
    let _ = legacy_frame();
    for _ in 0..frames {
        let t = std::time::Instant::now();
        let _ = legacy_frame();
        legacy_best = legacy_best.min(t.elapsed());
    }
    // First cached frame pays the (now allocation-free) scan; the rest are hits.
    let mut cached_hit_best = std::time::Duration::MAX;
    for _ in 0..frames {
        let t = std::time::Instant::now();
        let _ = cached_search_frame(&mut cache, &mut subset_cache, &tables, &graph, query, 1);
        cached_hit_best = cached_hit_best.min(t.elapsed());
    }
    println!(
        "er_search fixture=1300x30 frames={frames} legacy_per_frame_best={legacy_best:?} cached_cold_first={cold:?} cached_hit_best={cached_hit_best:?}"
    );
    assert_eq!(cache.recomputes(), 1, "steady-state frames must not rescan the catalog");
    assert_eq!(subset_cache.recomputes(), 1, "steady-state frames must not rescan the graph");

    if !cfg!(debug_assertions) {
        assert!(
            cached_hit_best < std::time::Duration::from_micros(100),
            "cached search frame must stay under 0.1 ms, measured {cached_hit_best:?}"
        );
        assert!(
            legacy_best > std::time::Duration::from_micros(10),
            "legacy pass should still be measurable on this fixture (guard against a dead benchmark)"
        );
    }
}
