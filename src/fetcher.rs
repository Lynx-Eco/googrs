use reqwest::Client;
use crate::SearchMdError;
use std::time::Duration;

/// A struct responsible for fetching HTTP content.
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    /// Creates a new `Fetcher` with the given configurations.
    pub fn new(
        timeout: u64,
        ssl_verify: bool,
        proxy: Option<String>
    ) -> Result<Self, SearchMdError> {
        let client_builder = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .danger_accept_invalid_certs(!ssl_verify);

        let client_builder = if let Some(proxy_url) = proxy {
            client_builder.proxy(reqwest::Proxy::all(proxy_url)?)
        } else {
            client_builder
        };

        let client = client_builder.build()?;

        Ok(Fetcher { client })
    }

    /// Fetches the content from the given URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to fetch.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response body as a `String` or a `SearchMdError`.
    pub async fn fetch(&self, url: &str) -> Result<String, SearchMdError> {
        let response = self.client
            .get(url)
            .header("User-Agent", super::user_agents::get_useragent())
            .send().await?
            .error_for_status()?
            .text().await?;

        Ok(response)
    }
}
