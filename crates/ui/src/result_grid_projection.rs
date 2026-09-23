//! Result-grid projection lookup and selection-cache primitives.
use crate::GridProjectionKey;
use std::collections::HashMap;

/// Coordinate lookup for the current visible grid slice.
pub struct GridSelectionLookup {
    pub row_positions: HashMap<usize, usize>,
    pub column_positions: HashMap<usize, usize>,
}

impl GridSelectionLookup {
    /// Build a lookup from the filtered row projection and the visual column order.
    pub fn new(indexes: &[usize], order: &[usize]) -> Self {
        Self {
            row_positions: indexes
                .iter()
                .enumerate()
                .map(|(position, &row)| (row, position))
                .collect(),
            column_positions: order
                .iter()
                .enumerate()
                .map(|(position, &column)| (column, position))
                .collect(),
        }
    }
}

/// One-entry memo for [`GridSelectionLookup`].
#[derive(Default)]
pub struct GridSelectionCache {
    key: Option<(GridProjectionKey, Vec<usize>)>,
    lookup: Option<GridSelectionLookup>,
    rebuilds: u64,
}

impl GridSelectionCache {
    /// Take the selection lookup for `(projection_key, order)`, reusing the cached entry when the
    /// key matches. Returns `None` on a miss — the caller must rebuild and [`Self::restore`] it.
    pub fn take(&mut self, projection_key: &GridProjectionKey, order: &[usize]) -> Option<GridSelectionLookup> {
        let hit = self
            .key
            .as_ref()
            .map(|(key, cached_order)| key == projection_key && cached_order.as_slice() == order)
            .unwrap_or(false);
        if hit {
            self.key = None;
            return self.lookup.take();
        }
        self.key = None;
        self.lookup = None;
        self.rebuilds = self.rebuilds.saturating_add(1);
        None
    }

    /// Store a selection lookup the caller got from [`Self::take`] so the next frame can reuse it.
    pub fn restore(&mut self, projection_key: GridProjectionKey, order: Vec<usize>, lookup: GridSelectionLookup) {
        if self.key.is_some() {
            return;
        }
        self.key = Some((projection_key, order));
        self.lookup = Some(lookup);
    }

    /// Number of selection lookups built by this cache. Asserted by tests to pin the
    /// no-per-frame-rebuild property.
    #[allow(dead_code)]
    pub(super) fn rebuilds(&self) -> u64 {
        self.rebuilds
    }
}
