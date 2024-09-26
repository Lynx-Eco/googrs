//! # SearchMD Library
//!
//! `search_md` is a library for performing Google searches and parsing the results into Markdown format.
//! It provides functionalities to search, fetch, parse, and clean Markdown content from search results.

pub mod md_parse;
pub mod reassembler;
pub mod scorer;
pub mod user_agents;
pub mod fetcher;

use std::time::Duration;
use serde_urlencoded; // Added to fix the undeclared crate/module error
use htmd::{ options::Options, HtmlToMarkdown };
use md_parse::MarkdownBlock;
use reassembler::MarkdownReassembler;
use scorer::MarkdownScorer;
use fetcher::Fetcher;
use serde::Serialize;
use thiserror::Error;

/// Represents a basic search result from Google.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
}

/// Represents a search result with Markdown content.
#[derive(Debug, Serialize)]
pub struct MarkdownResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub content: String,
}

/// List of URLs known to block scraping.
pub static SCRAPING_BLOCKLIST: &[&str] = &[
    "reddit.com",
    "facebook.com",
    "twitter.com",
    "linkedin.com",
    "instagram.com",
    "tiktok.com",
    "pinterest.com",
    "quora.com",
    "glassdoor.com",
    "yelp.com",
];

/// Custom error type for the SearchMD library.
#[derive(Error, Debug)]
pub enum SearchMdError {
    /// Represents errors from the Reqwest library.
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Represents errors from the Scraper crate's Selector parsing.
    #[error("Selector parse error: {0}")]
    SelectorParseError(#[from] scraper::error::SelectorErrorKind<'static>),

    /// Represents errors from the HTMD library.
    #[error("HTMD conversion error: {0}")]
    HtmdError(String),

    /// Represents other generic errors.
    #[error("Other error: {0}")]
    Other(String),
}

/// Type alias for `Result` with `SearchMdError`.
pub type SearchMdResult<T> = Result<T, SearchMdError>;

/// Configuration parameters for performing a search.
#[derive(Clone)]
pub struct SearchConfig {
    pub term: String,
    pub num_results: usize,
    pub lang: String,
    pub proxy: Option<String>,
    pub sleep_interval: u64,
    pub timeout: u64,
    pub safe: String,
    pub ssl_verify: bool,
    pub region: Option<String>,
    pub start_num: usize,
    pub date_range: Option<(String, String)>,
}

impl SearchConfig {
    /// Creates a new `SearchConfig` with default values.
    pub fn new() -> Self {
        Self {
            term: "".to_string(),
            num_results: 10,
            lang: "en".to_string(),
            proxy: None,
            sleep_interval: 1,
            timeout: 10,
            safe: "off".to_string(),
            ssl_verify: true,
            region: None,
            start_num: 0,
            date_range: None,
        }
    }

    // Builder methods for setting configuration parameters
    pub fn set_term(mut self, term: String) -> Self {
        self.term = term;
        self
    }

    pub fn num_results(mut self, num: usize) -> Self {
        self.num_results = num;
        self
    }

    pub fn lang(mut self, lang: String) -> Self {
        self.lang = lang;
        self
    }

    pub fn proxy(mut self, proxy: String) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn sleep_interval(mut self, interval: u64) -> Self {
        self.sleep_interval = interval;
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn safe(mut self, safe: String) -> Self {
        self.safe = safe;
        self
    }

    pub fn ssl_verify(mut self, verify: bool) -> Self {
        self.ssl_verify = verify;
        self
    }

    pub fn region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn start_num(mut self, start: usize) -> Self {
        self.start_num = start;
        self
    }

    pub fn date_range(mut self, start_date: String, end_date: String) -> Self {
        self.date_range = Some((start_date, end_date));
        self
    }
}

/// Builder for the SearchMD library.
pub struct SearchMdBuilder {
    config: SearchConfig,
    scorer: Option<MarkdownScorer>,
    reassembler: Option<MarkdownReassembler>,
}

impl SearchMdBuilder {
    /// Creates a new `SearchMdBuilder` with default configurations.
    pub fn new() -> Self {
        Self {
            config: SearchConfig::new(),
            scorer: None,
            reassembler: None,
        }
    }

    /// Sets the search term.
    pub fn term(mut self, term: String) -> Self {
        self.config = self.config.set_term(term);
        self
    }

    /// Sets the number of results to fetch.
    pub fn num_results(mut self, num: usize) -> Self {
        self.config = self.config.num_results(num);
        self
    }

    /// Sets the language for the search.
    pub fn lang(mut self, lang: String) -> Self {
        self.config = self.config.lang(lang);
        self
    }

    /// Sets a proxy for the search.
    pub fn proxy(mut self, proxy: String) -> Self {
        self.config = self.config.proxy(proxy);
        self
    }

    /// Sets the sleep interval between requests.
    pub fn sleep_interval(mut self, interval: u64) -> Self {
        self.config = self.config.sleep_interval(interval);
        self
    }

    /// Sets the timeout for HTTP requests.
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.config = self.config.timeout(timeout);
        self
    }

    /// Sets the safe search parameter.
    pub fn safe(mut self, safe: String) -> Self {
        self.config = self.config.safe(safe);
        self
    }

    /// Enables or disables SSL verification.
    pub fn ssl_verify(mut self, verify: bool) -> Self {
        self.config = self.config.ssl_verify(verify);
        self
    }

    /// Sets the region for the search.
    pub fn region(mut self, region: String) -> Self {
        self.config = self.config.region(region);
        self
    }

    /// Sets the starting index for the search results.
    pub fn start_num(mut self, start: usize) -> Self {
        self.config = self.config.start_num(start);
        self
    }

    /// Sets the date range for the search.
    pub fn date_range(mut self, start_date: String, end_date: String) -> Self {
        self.config = self.config.date_range(start_date, end_date);
        self
    }

    /// Optionally sets a custom `MarkdownScorer`.
    pub fn scorer(mut self, scorer: MarkdownScorer) -> Self {
        self.scorer = Some(scorer);
        self
    }

    /// Optionally sets a custom `MarkdownReassembler`.
    pub fn reassembler(mut self, reassembler: MarkdownReassembler) -> Self {
        self.reassembler = Some(reassembler);
        self
    }

    /// Initializes the builder and returns a `SearchMd` instance.
    pub fn build(self) -> SearchMd {
        let fetcher = Fetcher::new(
            self.config.timeout,
            self.config.ssl_verify,
            self.config.proxy.clone()
        ).expect("Failed to initialize Fetcher");
        let converter = HtmlToMarkdown::builder()
            .options(Options {
                link_style: htmd::options::LinkStyle::Referenced,
                link_reference_style: htmd::options::LinkReferenceStyle::Collapsed,
                ..Default::default()
            })
            .skip_tags(vec!["script", "style", "iframe", "img", "svg"])
            .build();

        SearchMd {
            config: self.config,
            fetcher,
            scorer: self.scorer.unwrap_or_else(MarkdownScorer::new),
            reassembler: self.reassembler.unwrap_or_else(MarkdownReassembler::new),
            converter,
        }
    }
}

/// The main struct for performing searches and processing results.
pub struct SearchMd {
    config: SearchConfig,
    fetcher: Fetcher,
    scorer: MarkdownScorer,
    reassembler: MarkdownReassembler,
    converter: HtmlToMarkdown,
}

impl SearchMd {
    /// Performs a Google search and returns a list of search results.
    pub async fn search(&self, term: &str) -> SearchMdResult<Vec<SearchResult>> {
        self.fetch_search_results(term).await
    }

    /// Performs a Google search and returns a list of search results with Markdown content.
    pub async fn search_md(&self, term: &str) -> SearchMdResult<Vec<MarkdownResult>> {
        let search_results = self.search(term).await?;

        self.fetch_and_process_results(search_results).await
    }

    /// Fetches search results from Google based on the given configuration.
    async fn fetch_search_results(&self, term: &str) -> SearchMdResult<Vec<SearchResult>> {
        let mut results = Vec::new();
        let mut start = self.config.start_num;
        let mut fetched_results = 0;

        while fetched_results < self.config.num_results {
            let remaining = self.config.num_results.saturating_sub(fetched_results);
            let num = std::cmp::min(remaining, 10); // Google typically returns max 10 results per page

            let mut query_params = vec![
                ("q", term.to_string()),
                ("num", (num + 2).to_string()), // Prevents multiple requests
                ("hl", self.config.lang.clone()),
                ("start", start.to_string()),
                ("safe", self.config.safe.clone())
            ];

            // Add date range if specified
            if let Some((start_date, end_date)) = &self.config.date_range {
                query_params.push((
                    "tbs",
                    format!("cdr:1,cd_min:{},cd_max:{}", start_date, end_date),
                ));
            }

            if let Some(region_code) = &self.config.region {
                query_params.push(("gl", region_code.clone()));
            }

            let url = format!(
                "https://www.google.com/search?{}",
                serde_urlencoded
                    ::to_string(&query_params)
                    .map_err(|e| SearchMdError::Other(e.to_string()))?
            );
            let html = self.fetcher.fetch(&url).await?;

            let document = scraper::Html::parse_document(&html);
            let selector = scraper::Selector
                ::parse("div.g")
                .map_err(SearchMdError::SelectorParseError)?;

            let mut new_results = 0;

            for element in document.select(&selector) {
                if
                    let (Some(link), Some(title), Some(description)) = (
                        element.select(&scraper::Selector::parse("a").unwrap()).next(),
                        element.select(&scraper::Selector::parse("h3").unwrap()).next(),
                        element
                            .select(
                                &scraper::Selector
                                    ::parse("div[style='-webkit-line-clamp:2']")
                                    .unwrap()
                            )
                            .next(),
                    )
                {
                    let url = link.value().attr("href").unwrap_or("").to_string();
                    let title = title.text().collect::<String>();
                    let description = description.text().collect::<String>();

                    results.push(SearchResult {
                        url,
                        title,
                        description,
                    });

                    new_results += 1;
                    fetched_results += 1;

                    if fetched_results >= self.config.num_results {
                        break;
                    }
                }
            }

            if new_results == 0 {
                break; // No more results found
            }

            start += new_results;
            tokio::time::sleep(Duration::from_secs(self.config.sleep_interval)).await;
        }

        Ok(results)
    }

    /// Fetches and processes each search result into Markdown.
    async fn fetch_and_process_results(
        &self,
        search_results: Vec<SearchResult>
    ) -> SearchMdResult<Vec<MarkdownResult>> {
        let mut markdown_results = Vec::new();

        for search_result in search_results {
            if is_blocked(&search_result.url) {
                continue;
            }
            let markdown = match self.fetcher.fetch(&search_result.url).await {
                Ok(html) => {
                    let markdown = self.converter.convert(&html).unwrap();
                    clean_markdown(&markdown, &self.scorer, &self.reassembler)
                }
                Err(e) => {
                    eprintln!("Failed to fetch or process {}: {}", search_result.url, e);
                    continue;
                }
            };

            markdown_results.push(MarkdownResult {
                url: search_result.url,
                title: search_result.title,
                description: search_result.description,
                content: markdown,
            });
            tokio::time::sleep(Duration::from_secs(self.config.sleep_interval)).await;
        }

        Ok(markdown_results)
    }
}

/// Cleans and processes Markdown content.
fn clean_markdown(
    input_markdown: &str,
    scorer: &MarkdownScorer,
    reassembler: &MarkdownReassembler
) -> String {
    // Parse and preprocess markdown
    let preprocessed_markdown = preprocess_markdown(input_markdown);

    // Parse markdown into blocks
    let parser = md_parse::MarkdownParser::new(&preprocessed_markdown);
    let blocks = parser.parse();

    // Score blocks
    let scored_blocks = scorer.score_blocks(&blocks);

    // Determine threshold
    let threshold = scorer.calculate_threshold(&scored_blocks);

    // Filter blocks based on threshold
    let filtered_blocks: Vec<MarkdownBlock> = scored_blocks
        .into_iter()
        .filter(|&(_, score)| score >= threshold)
        .map(|(block, _)| block.clone())
        .collect();

    // Reassemble markdown
    reassembler.reassemble(&filtered_blocks)
}

/// Removes all content after the "✕" character, if present.
fn preprocess_markdown(input: &str) -> String {
    input.split('✕').next().unwrap_or(input).to_string()
}

/// Checks if a given URL is in the scraping blocklist.
pub fn is_blocked(url: &str) -> bool {
    SCRAPING_BLOCKLIST.iter().any(|&blocked| url.contains(blocked))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_markdown() {
        let input = "This is a test ✕ Remove this part.";
        let output = preprocess_markdown(input);
        assert_eq!(output, "This is a test ");
    }

    #[test]
    fn test_is_blocked() {
        assert!(is_blocked("https://www.reddit.com"));
        assert!(!is_blocked("https://www.example.com"));
    }

    // Add more tests as needed
}
