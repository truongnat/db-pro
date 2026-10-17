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
    /// `len:last_entry_id` — id changes on every push, len catches clears (#500).
    pub history: &'a str,
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
#[path = "tests/search_service_tests.rs"]
mod tests;

