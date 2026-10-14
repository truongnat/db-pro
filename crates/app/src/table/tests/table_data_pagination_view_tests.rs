//! Tests for `../table_data_pagination_view.rs` (moved out of the source file).
    use super::*;

    fn paging(has_next: bool, has_previous: bool) -> TableDataPaging {
        TableDataPaging {
            page_range: String::new(),
            total_rows: 3,
            total_known: true,
            has_next,
            has_previous,
            inline_query_result: false,
        }
    }

    #[test]
    fn hides_navigation_when_rows_fit_on_one_page() {
        assert!(!should_show_navigation(&paging(false, false)));
    }

    #[test]
    fn keeps_navigation_when_an_adjacent_page_exists() {
        assert!(should_show_navigation(&paging(true, false)));
        assert!(should_show_navigation(&paging(false, true)));
    }