use reqwest::Client;
use crate::user_agents::get_useragent;
use crate::SearchMdError;
use std::time::Duration;
use reqwest::cookie::Jar;
use std::sync::Arc;

use rand::seq::SliceRandom;

/// A struct responsible for fetching HTTP content with advanced scraping techniques.
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    /// Creates a new `Fetcher` with the given configurations, including cookie handling.
    pub fn new(
        timeout: u64,
        ssl_verify: bool,
        proxy: Option<String>
    ) -> Result<Self, SearchMdError> {
        // Initialize a cookie jar to handle cookies
        let cookie_store = Arc::new(Jar::default());

        let client_builder = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .danger_accept_invalid_certs(!ssl_verify)
            .cookie_provider(cookie_store.clone());

        let client_builder = if let Some(proxy_url) = proxy {
            client_builder.proxy(reqwest::Proxy::all(proxy_url)?)
        } else {
            client_builder
        };

        let client = client_builder.build()?;

        Ok(Fetcher { client })
    }

    /// Returns a random Referer from a predefined list.
    fn get_random_referer(&self) -> String {
        let referers = vec![
            "https://www.google.com/",
            "https://www.bing.com/",
            "https://www.yahoo.com/",
            "https://duckduckgo.com/"
            // Add more referers as needed
        ];
        let mut rng = rand::thread_rng();
        referers.choose(&mut rng).unwrap().to_string()
    }

    /// Sets additional headers to emulate a real browser and enhance fingerprinting.
    fn set_headers(&self, url: &str) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .header("User-Agent", get_useragent())
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("DNT", "1") // Do Not Track
            .header("Referer", self.get_random_referer())
            // .header("Accept-Encoding", "gzip, deflate, br")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-User", "?1")
            .header("Sec-Fetch-Dest", "document")
    }

    /// Fetches the content from the given URL with emulated headers, fingerprinting, and cookie handling.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response body as a `String` or a `SearchMdError`.
    pub async fn fetch(&self, url: &str) -> Result<String, SearchMdError> {
        let response = self.set_headers(url).send().await?.error_for_status()?.text().await?;

        Ok(response)
    }

    /// Fetches markdown content from the given URL, prepending "https://r.jina.ai" to the URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch, without the "https://r.jina.ai" prefix.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response body as a `String` or a `SearchMdError`.
    pub async fn fetch_md(&self, url: &str) -> Result<String, SearchMdError> {
        let full_url = format!("https://r.jina.ai/{}", url);
        self.fetch(&full_url).await
    }
}
