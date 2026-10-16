//! Unicode-insensitive search that keeps match spans in the original UTF-8 text.

pub(crate) fn case_insensitive_matches(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }

    let folded_needle = needle.chars().flat_map(char::to_lowercase).collect::<String>();
    let mut folded_haystack = String::with_capacity(haystack.len());
    let mut starts = Vec::new();
    let mut ends = Vec::new();

    for (original_start, character) in haystack.char_indices() {
        let original_end = original_start + character.len_utf8();
        for folded_character in character.to_lowercase() {
            let folded_start = folded_haystack.len();
            folded_haystack.push(folded_character);
            let folded_end = folded_haystack.len();
            starts.push((folded_start, original_start));
            ends.push((folded_end, original_end));
        }
    }

    folded_haystack
        .match_indices(&folded_needle)
        .filter_map(|(folded_start, matched)| {
            let folded_end = folded_start + matched.len();
            let original_start = starts
                .binary_search_by_key(&folded_start, |(offset, _)| *offset)
                .ok()
                .map(|index| starts[index].1)?;
            let original_end = ends
                .binary_search_by_key(&folded_end, |(offset, _)| *offset)
                .ok()
                .map(|index| ends[index].1)?;
            Some((original_start, original_end))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::case_insensitive_matches;

    #[test]
    fn maps_expanding_lowercase_matches_back_to_original_byte_ranges() {
        assert_eq!(case_insensitive_matches("İx", "x"), vec![(2, 3)]);
        assert_eq!(case_insensitive_matches("İx", "i"), vec![(0, 2)]);
        assert_eq!(case_insensitive_matches("aİbİ", "i\u{307}"), vec![(1, 3), (4, 6)]);
    }

    #[test]
    fn keeps_ascii_case_insensitive_and_empty_query_behavior() {
        assert_eq!(
            case_insensitive_matches("Select SELECT", "select"),
            vec![(0, 6), (7, 13)]
        );
        assert!(case_insensitive_matches("anything", "").is_empty());
    }
}
