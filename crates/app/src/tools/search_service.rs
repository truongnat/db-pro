//! Global search index + ranking for Quick Open / Command Palette (#201).
//!
//! UI widgets consume ranked results; they must not re-implement ad-hoc scans.

use super::{PaletteItem, SearchKind, SearchScope};

/// Inputs that identify when the search index must be rebuilt.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SearchFingerprintParts<'a> {
    pub connection_id: Option<&'a str>,
    pub schema: &'a str,
    pub tables: usize,
    pub views: usize,
    pub functions: usize,
    pub columns: usize,
    pub saved_queries: usize,
    pub history: usize,
    pub connections: usize,
    pub workspace_files: usize,
}

/// Cached searchable entries keyed by a schema/connection fingerprint.
#[derive(Debug, Default, Clone)]
pub(crate) struct SearchIndex {
    fingerprint: String,
    entries: Vec<(SearchKind, PaletteItem)>,
}

impl SearchIndex {
    pub(crate) fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Drop cached entries so the next rebuild uses live state.
    pub(crate) fn invalidate(&mut self) {
        self.fingerprint.clear();
        self.entries.clear();
    }

    /// Replace the cache unconditionally (caller already decided the fingerprint is stale).
    pub(crate) fn replace(&mut self, fingerprint: String, entries: Vec<(SearchKind, PaletteItem)>) {
        self.fingerprint = fingerprint;
        self.entries = entries;
    }

    /// Rebuild when the fingerprint changes (connection/schema/query metadata drift).
    #[cfg(test)]
    pub(crate) fn ensure(&mut self, fingerprint: String, build: impl FnOnce() -> Vec<(SearchKind, PaletteItem)>) {
        if self.fingerprint == fingerprint && !self.entries.is_empty() {
            return;
        }
        self.replace(fingerprint, build());
    }

    pub(crate) fn entries(&self) -> &[(SearchKind, PaletteItem)] {
        &self.entries
    }
}

/// Pure ranking / scope filter used by the palette and unit tests.
pub(crate) struct SearchService;

impl SearchService {
    pub(crate) fn build_fingerprint(parts: SearchFingerprintParts<'_>) -> String {
        format!(
            "c={}|s={}|t={}|v={}|f={}|col={}|sq={}|h={}|n={}|w={}",
            parts.connection_id.unwrap_or("-"),
            parts.schema,
            parts.tables,
            parts.views,
            parts.functions,
            parts.columns,
            parts.saved_queries,
            parts.history,
            parts.connections,
            parts.workspace_files,
        )
    }

    /// Score a candidate. `None` means it does not match the query.
    /// Higher scores rank first. Deterministic ties fall back to title order at the call site.
    pub(crate) fn score(query: &str, title: &str, subtitle: &str, kind: SearchKind) -> Option<u32> {
        let q = query.trim().to_ascii_lowercase();
        let base = kind.rank_boost();
        if q.is_empty() {
            return Some(1_000 + base);
        }
        let title_l = title.to_ascii_lowercase();
        let subtitle_l = subtitle.to_ascii_lowercase();
        let mut score = if title_l == q {
            50_000
        } else if title_l.starts_with(&q) {
            40_000
        } else if title_l.contains(&q) {
            30_000
        } else if subtitle_l.contains(&q) {
            20_000
        } else {
            return None;
        };
        score += base;
        // Prefer shorter titles on equal match class (stable, deterministic).
        let brevity = 1_000u32.saturating_sub(title.len() as u32);
        Some(score + brevity)
    }

    pub(crate) fn filter_rank(
        entries: &[(SearchKind, PaletteItem)],
        query: &str,
        scope: SearchScope,
        limit: usize,
    ) -> Vec<PaletteItem> {
        let mut scored: Vec<(u32, &PaletteItem)> = entries
            .iter()
            .filter(|(kind, _)| kind.matches_scope(scope))
            .filter_map(|(kind, item)| {
                Self::score(query, &item.title, &item.subtitle, *kind).map(|score| (score, item))
            })
            .collect();
        scored.sort_by(|(a_score, a_item), (b_score, b_item)| {
            b_score
                .cmp(a_score)
                .then_with(|| {
                    a_item
                        .title
                        .to_ascii_lowercase()
                        .cmp(&b_item.title.to_ascii_lowercase())
                })
                .then_with(|| {
                    a_item
                        .subtitle
                        .to_ascii_lowercase()
                        .cmp(&b_item.subtitle.to_ascii_lowercase())
                })
        });
        scored.into_iter().take(limit).map(|(_, item)| item.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::PaletteAction;
    use super::*;
    use lucide_icons::Icon;

    fn item(title: &str, subtitle: &str) -> PaletteItem {
        PaletteItem {
            icon: Icon::Table2,
            title: title.to_owned(),
            subtitle: subtitle.to_owned(),
            shortcut: None,
            action: PaletteAction::Welcome,
        }
    }

    #[test]
    fn ranking_prefers_prefix_and_exact_title() {
        let entries = vec![
            (SearchKind::Table, item("users_archive", "Open table")),
            (SearchKind::Table, item("users", "Open table")),
            (SearchKind::Column, item("user_id", "Insert column")),
            (SearchKind::Command, item("Export results", "Export current results")),
        ];
        let ranked = SearchService::filter_rank(&entries, "users", SearchScope::All, 10);
        assert_eq!(ranked[0].title, "users");
        assert!(ranked.iter().any(|i| i.title == "users_archive"));
    }

    #[test]
    fn scope_filter_excludes_other_kinds() {
        let entries = vec![
            (SearchKind::Table, item("orders", "Open table")),
            (SearchKind::Command, item("New query", "Create scratch")),
            (SearchKind::Agent, item("Ask Agent", "Agent action")),
        ];
        let schema_only = SearchService::filter_rank(&entries, "", SearchScope::Schema, 10);
        assert_eq!(schema_only.len(), 1);
        assert_eq!(schema_only[0].title, "orders");

        let agent_only = SearchService::filter_rank(&entries, "", SearchScope::Agent, 10);
        assert_eq!(agent_only.len(), 1);
        assert_eq!(agent_only[0].title, "Ask Agent");
    }

    #[test]
    fn index_invalidates_when_fingerprint_changes() {
        let mut index = SearchIndex::default();
        index.ensure("fp-a".into(), || vec![(SearchKind::Table, item("a", "t"))]);
        assert_eq!(index.entries().len(), 1);
        assert_eq!(index.fingerprint(), "fp-a");
        index.ensure("fp-a".into(), || {
            vec![(SearchKind::Table, item("should-not-rebuild", "t"))]
        });
        assert_eq!(index.entries()[0].1.title, "a");
        index.invalidate();
        assert!(index.is_empty());
        index.ensure("fp-b".into(), || vec![(SearchKind::Table, item("b", "t"))]);
        assert_eq!(index.entries()[0].1.title, "b");
    }

    #[test]
    fn large_schema_search_is_bounded() {
        let entries: Vec<_> = (0..5_000)
            .map(|i| (SearchKind::Table, item(&format!("table_{i:04}"), "Open table")))
            .collect();
        let ranked = SearchService::filter_rank(&entries, "table_42", SearchScope::Schema, 80);
        assert!(ranked.len() <= 80);
        assert!(ranked
            .iter()
            .any(|i| i.title.contains("0042") || i.title.contains("42")));
    }
}
