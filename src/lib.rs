mod user_agents;
use md_parse::MarkdownBlock;
use readablers::ReadabilityOptions;
use reqwest::Url;
use reqwest::Client;
use scraper::{ Html, Selector };
use serde::Serialize;
use tokio::task;
use std::time::Duration;
use htmd::{ options::Options, Element, HtmlToMarkdown };
use markup5ever::{ local_name, LocalName, Namespace };
use std::sync::{ Arc, Mutex };
use task::spawn_blocking;
use readablers::Readability;
use html2text::{
    from_read_rich,
    render::text_renderer::RichAnnotation,
    render::text_renderer::TaggedLine,
};
use std::fs::File;
use std::io::Write;
mod md_parse;
mod scorer;
mod reassembler;

use md_parse::MarkdownParser;
use scorer::MarkdownScorer;
use reassembler::MarkdownReassembler;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
}
#[derive(Debug, Serialize)]
pub struct MarkdownResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub content: String,
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

pub async fn search_md(
    term: &str,
    num_results: usize,
    lang: &str,
    proxy: Option<&str>,
    sleep_interval: u64,
    timeout: u64,
    safe: &str,
    ssl_verify: bool,
    region: Option<&str>,
    start_num: usize,
    remove_links: bool
) -> Result<Vec<MarkdownResult>, Box<dyn std::error::Error>> {
    let search_result = search(
        term,
        num_results,
        lang,
        proxy,
        sleep_interval,
        timeout,
        safe,
        ssl_verify,
        region,
        0
    ).await?;

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .danger_accept_invalid_certs(!ssl_verify)
        .build()?;

    let mut markdown_results = Vec::new();

    for search_result in search_result {
        if is_blocked(&search_result.url) {
            continue;
        }
        let resp = client.get(&search_result.url).send().await?;
        let html = resp.text().await?;

        let converter = HtmlToMarkdown::builder()
            .options(Options {
                link_style: htmd::options::LinkStyle::Referenced,
                link_reference_style: htmd::options::LinkReferenceStyle::Collapsed,
                ..Default::default()
            })
            .skip_tags(vec!["script", "style", "iframe", "img", "svg"])
            .build();

        let mut markdown = converter.convert(&html)?;
        markdown = clean_markdown(&markdown);
        markdown_results.push(MarkdownResult {
            url: search_result.url,
            title: search_result.title,
            description: search_result.description,
            content: markdown,
        });
        tokio::time::sleep(Duration::from_secs(sleep_interval)).await;
    }

    Ok(markdown_results)
}

pub async fn search_reader(
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
) -> Result<Vec<ReaderResult>, Box<dyn std::error::Error>> {
    let urls = search_url(
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

    let mut reader_results = Vec::new();

    for url in urls {
        let url_clone = url.clone();
        // Use spawn_blocking to run the synchronous scrape function
        let client = Client::new();
        let response = client.get(&url).send().await?;
        let html = response.text().await?;

        let options = ReadabilityOptions {
            debug: true,
            ..ReadabilityOptions::default()
        };
        let mut readability = Readability::new(&html, options);

        let result = readability.parse();
        let result = result.unwrap();
        let reader_result = ReaderResult {
            url: url_clone,
            title: result.title,
            description: result.excerpt.unwrap_or_default(),
            content: result.content,
        };

        reader_results.push(reader_result);

        tokio::time::sleep(Duration::from_secs(sleep_interval)).await;
    }

    Ok(reader_results)
}

#[derive(Debug, Serialize)]
pub struct ReaderResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub content: String,
}

pub async fn search_to_text(
    term: &str,
    num_results: usize,
    lang: &str,
    proxy: Option<&str>,
    sleep_interval: u64,
    timeout: u64,
    safe: &str,
    ssl_verify: bool,
    region: Option<&str>,
    start_num: usize,
    output_file: &str,
    width: usize
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .danger_accept_invalid_certs(!ssl_verify)
        .build()?;

    let search_results = search(
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

    let mut file = File::create(output_file)?;

    for (index, result) in search_results.into_iter().enumerate() {
        writeln!(file, "Result {}", index + 1)?;
        writeln!(file, "URL: {}", result.url)?;
        writeln!(file, "Title: {}", result.title)?;
        writeln!(file, "Description: {}", result.description)?;
        writeln!(file, "Content:")?;

        let resp = client.get(&result.url).send().await?;
        let html = resp.text().await?;

        // Convert HTML to rich text using html2text
        let rich_text: Vec<TaggedLine<Vec<RichAnnotation>>> = from_read_rich(
            html.as_bytes(),
            width
        );

        // Write rich text to file
        for line in rich_text {
            // Use the public method `into_string` to get the text
            let text = line.into_string();
            writeln!(file, "String: {}", text)?;
            // Tags are not accessible because `v` is private
            writeln!(file, "Tags: [Not Accessible]")?;
        }

        writeln!(file, "---")?;

        tokio::time::sleep(Duration::from_secs(sleep_interval)).await;
    }

    Ok(())
}
/// Removes all content after the "✕" character, if present.
fn preprocess_markdown(input: &str) -> String {
    if let Some(index) = input.find('✕') { input[..index].to_string() } else { input.to_string() }
}

pub fn clean_markdown(input_markdown: &str) -> String {
    // Parse markdown and segment into blocks
    let preprocessed_markdown = preprocess_markdown(input_markdown);

    // Parse markdown and segment into blocks
    let parser = MarkdownParser::new(&preprocessed_markdown);
    let blocks = parser.parse();

    // Score blocks
    let scorer = MarkdownScorer::new();
    let scored_blocks = scorer.score_blocks(&blocks);

    // Determine threshold
    let threshold = scorer.calculate_threshold(&scored_blocks);

    // Filter blocks
    let filtered_blocks: Vec<MarkdownBlock> = scored_blocks
        .into_iter()
        .filter(|&(_, score)| score >= threshold)
        .map(|(block, _)| block.clone()) // Clone the block here
        .collect();

    // Reassemble markdown
    let reassembler = MarkdownReassembler::new();
    reassembler.reassemble(&filtered_blocks)
}
// List of URLs known to block scraping
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

/// Checks if a given URL is in the scraping blocklist
pub fn is_blocked(url: &str) -> bool {
    SCRAPING_BLOCKLIST.iter().any(|&blocked| url.contains(blocked))
}
