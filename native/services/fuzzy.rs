use std::sync::{Mutex, OnceLock};

use nucleo::{Config, Matcher, Utf32Str};

static MATCHER: OnceLock<Mutex<Matcher>> = OnceLock::new();

/// Return Nucleo's fzf-style fuzzy score for a query/candidate pair.
/// The matcher is reused so keystrokes do not repeatedly allocate its
/// internal scratch buffers.
pub fn score(query: &str, candidate: &str) -> Option<u16> {
    let query = query.trim();
    if query.is_empty() {
        return Some(0);
    }

    let matcher = MATCHER.get_or_init(|| Mutex::new(Matcher::new(Config::DEFAULT)));
    let mut matcher = matcher.lock().unwrap_or_else(|error| error.into_inner());

    let mut query_buffer = Vec::new();
    let needle = Utf32Str::new(query, &mut query_buffer);

    let mut candidate_buffer = Vec::new();
    let haystack = Utf32Str::new(candidate, &mut candidate_buffer);

    matcher.fuzzy_match(haystack, needle)
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn matches_out_of_order_terms() {
        assert!(score("firefox", "Mozilla Firefox Web Browser").is_some());
    }

    #[test]
    fn rejects_unrelated_candidates() {
        assert!(score("firefox", "System Settings").is_none());
    }
}
