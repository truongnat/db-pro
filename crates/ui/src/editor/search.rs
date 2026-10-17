/// Case-insensitive substring search over `text`, yielding `(start, end)`
/// byte ranges in the ORIGINAL text — never offsets into a lowercased copy.
///
/// Comparing `text.to_lowercase()` and `query.to_lowercase()` shifts byte
/// offsets whenever case folding changes char width (`İ` U+0130 → "i̇" is
/// 2→4 bytes, Kelvin `K` U+212A → "k" is 3→1 byte), so matched ranges would
/// highlight the wrong characters or slice mid-char.
/// Matching is per-char lowercase equality: one text char against one query
/// char. 'İ' (U+0130) lowercases to the two chars "i̇", which equals only
/// 'İ' itself — it cannot equal a single query 'i' or a two-char query
/// "i̇" (chars pair 1:1). Full case-folded substring matching is
/// deliberately out of scope for a find UI; what matters is that ranges
/// always lie on char boundaries of the original text.
pub fn case_insensitive_matches(text: &str, query: &str) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    if query.is_empty() {
        return matches;
    }
    let query_chars: Vec<char> = query.chars().collect();
    let text_chars: Vec<(usize, char)> = text.char_indices().collect();
    if query_chars.len() > text_chars.len() {
        return matches;
    }
    // Non-overlapping, left-to-right like `str::match_indices`.
    let mut i = 0;
    while i + query_chars.len() <= text_chars.len() {
        let hit = text_chars[i..i + query_chars.len()]
            .iter()
            .zip(&query_chars)
            .all(|((_, tc), qc)| tc.to_lowercase().eq(qc.to_lowercase()));
        if hit {
            let start = text_chars[i].0;
            let end = text_chars[i + query_chars.len() - 1].0 + text_chars[i + query_chars.len() - 1].1.len_utf8();
            matches.push((start, end));
            i += query_chars.len();
        } else {
            i += 1;
        }
    }
    matches
}

#[cfg(test)]
#[path = "tests/search_tests.rs"]
mod tests;
