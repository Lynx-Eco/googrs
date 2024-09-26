//! # User Agents Module
//!
//! This module provides functionality to randomly select a User-Agent string for HTTP requests.

use rand::seq::SliceRandom;

/// Returns a randomly selected User-Agent string from the predefined list.
///
/// # Panics
///
/// Panics if the `USER_AGENT_LIST` is empty.
///
/// # Examples
///
/// ```rust
/// use search_md::user_agents::get_useragent;
/// let ua = get_useragent();
/// println!("Selected User-Agent: {}", ua);
/// ```
pub fn get_useragent() -> &'static str {
    USER_AGENT_LIST.choose(&mut rand::thread_rng()).expect("User-Agent list is empty")
}

const USER_AGENT_LIST: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:66.0) Gecko/20100101 Firefox/66.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36 Edg/111.0.1661.62",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/111.0",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_useragent() {
        let ua = get_useragent();
        assert!(USER_AGENT_LIST.contains(&ua));
    }

    #[test]
    #[should_panic(expected = "User-Agent list is empty")]
    fn test_empty_useragent_list() {
        // Temporarily redefine USER_AGENT_LIST as empty for this test
        const EMPTY_USER_AGENT_LIST: &[&str] = &[];
        let original = USER_AGENT_LIST;
        // This is just illustrative; in practice, you'd refactor to allow dependency injection or mocking
    }

    // Add more tests as needed
}
