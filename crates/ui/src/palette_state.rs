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

#[cfg(test)]
mod tests {
    use super::PaletteState;

    #[test]
    fn default_palette_state_is_closed_and_unselected() {
        let state = PaletteState::default();

        assert!(state.mode.is_none());
        assert!(state.query.is_empty());
        assert_eq!(state.selected, 0);
        assert!(!state.focus_requested);
    }
}
