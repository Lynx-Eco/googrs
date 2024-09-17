mod user_agents;

use reqwest::Client;
use scraper::{ Html, Selector };
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
}

pub async fn search(
    term: &str,
    num_results: usize,
    lang: &str,
    proxy: Option<&str>,
    sleep_interval: u64,
    timeout: u64,
    safe: &str,
    ssl_verify: bool,
    region: Option<&str>,
    start_num: usize
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .danger_accept_invalid_certs(!ssl_verify);

    if let Some(proxy_url) = proxy {
        client_builder = client_builder.proxy(reqwest::Proxy::all(proxy_url)?);
    }

    let client = client_builder.build()?;

    let mut results = Vec::new();
    let mut start = start_num;
    let mut fetched_results = 0;

    while fetched_results < num_results {
        let remaining = num_results.saturating_sub(fetched_results);
        let num = std::cmp::min(remaining, 10); // Google typically returns max 10 results per page

        let mut query_params = vec![
            ("q", term.to_string()),
            ("num", (num + 2).to_string()), // Prevents multiple requests
            ("hl", lang.to_string()),
            ("start", start.to_string()),
            ("safe", safe.to_string())
        ];

        if let Some(region_code) = region {
            query_params.push(("gl", region_code.to_string()));
        }

        let resp = client
            .get("https://www.google.com/search")
            .header("User-Agent", user_agents::get_useragent())
            .query(&query_params)
            .send().await?;

        resp.error_for_status_ref()?;

        let html = resp.text().await?;
        let document = Html::parse_document(&html);
        let selector = Selector::parse("div.g").unwrap();

        let mut new_results = 0;

        for element in document.select(&selector) {
            if
                let (Some(link), Some(title), Some(description)) = (
                    element.select(&Selector::parse("a").unwrap()).next(),
                    element.select(&Selector::parse("h3").unwrap()).next(),
                    element
                        .select(&Selector::parse("div[style='-webkit-line-clamp:2']").unwrap())
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

                if fetched_results >= num_results {
                    break;
                }
            }
        }

        if new_results == 0 {
            // Uncomment the line below if you want to print a message when the desired amount of queries cannot be fulfilled
            // println!("Only {} results found for query requiring {} results. Moving on to the next query.", fetched_results, num_results);
            break; // No more results found
        }

        start += new_results;
        tokio::time::sleep(Duration::from_secs(sleep_interval)).await;
    }

    Ok(results)
}

pub async fn search_url(
    term: &str,
    num_results: usize,
    lang: &str,
    proxy: Option<&str>,
    sleep_interval: u64,
    timeout: u64,
    safe: &str,
    ssl_verify: bool,
    region: Option<&str>,
    start_num: usize
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let results = search(
        term,
        num_results,
        lang,
        proxy,
        sleep_interval,
        timeout,
        safe,
        ssl_verify,
        region,
        start_num
    ).await?;

    Ok(
        results
            .into_iter()
            .map(|r| r.url)
            .collect()
    )
}
