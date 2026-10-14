use super::*;

/// State owned by the command palette and global quick-open search.
#[derive(Debug)]
pub(crate) struct PaletteState {
    pub(super) mode: Option<PaletteMode>,
    pub(super) query: String,
    pub(super) scope: SearchScope,
    pub(super) selected: usize,
    pub(super) focus_requested: bool,
    pub(super) search_index: SearchIndex,
}

impl Default for PaletteState {
    fn default() -> Self {
        Self {
            mode: None,
            query: String::new(),
            scope: SearchScope::All,
            selected: 0,
            focus_requested: false,
            search_index: SearchIndex::default(),
        }
    }
}

impl PaletteState {
    pub(super) fn open(&mut self, mode: PaletteMode) {
        self.open_with_scope(mode, SearchScope::All);
    }

    pub(super) fn open_with_scope(&mut self, mode: PaletteMode, scope: SearchScope) {
        self.mode = Some(mode);
        self.query.clear();
        self.scope = scope;
        self.selected = 0;
        self.focus_requested = true;
        self.search_index.invalidate();
    }
}

#[cfg(test)]
#[path = "tests/palette_state_tests.rs"]
mod tests;

