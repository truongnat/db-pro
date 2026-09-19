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
mod tests {
    use super::{PaletteMode, PaletteState, SearchScope};

    #[test]
    fn default_palette_state_is_closed_and_unselected() {
        let state = PaletteState::default();

        assert!(state.mode.is_none());
        assert!(state.query.is_empty());
        assert_eq!(state.selected, 0);
        assert!(!state.focus_requested);
    }

    #[test]
    fn opening_palette_resets_query_selection_and_focuses_the_input() {
        let mut state = PaletteState {
            query: "stale".to_owned(),
            selected: 4,
            ..Default::default()
        };
        state.open_with_scope(PaletteMode::QuickOpen, SearchScope::Schema);

        assert_eq!(state.mode, Some(PaletteMode::QuickOpen));
        assert_eq!(state.scope, SearchScope::Schema);
        assert!(state.query.is_empty());
        assert_eq!(state.selected, 0);
        assert!(state.focus_requested);
    }
}
