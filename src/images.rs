use headless_chrome::{ Browser, LaunchOptionsBuilder, Tab };
use anyhow::{ Result, anyhow, Context };
use std::time::Duration;
use regex::Regex;

pub async fn get_google_image_urls(
    search_query: &str,
    num_images: usize,
    max_missed: usize
) -> Result<Vec<String>> {
    println!("Starting image search for query: {}", search_query);

    let launch_options = LaunchOptionsBuilder::default()
        .headless(true)
        .build()
        .context("Failed to build launch options")?;

    println!("Launching browser...");
    let browser = Browser::new(launch_options).context("Failed to create browser")?;

    println!("Waiting for initial tab...");
    let tab = browser.wait_for_initial_tab().context("Failed to get initial tab")?;

    let url = format!(
        "https://www.google.com/search?q={}&tbm=isch",
        urlencoding::encode(search_query)
    );

    println!("Navigating to URL: {}", url);
    tab.navigate_to(&url).context("Failed to navigate to URL")?;

    println!("Waiting for page to load...");
    tab.wait_for_element("img.rg_i").context("Failed to find any images")?;

    println!("Attempting to find images...");
    let image_urls = find_image_urls(&tab, num_images, max_missed).await?;

    println!("Search completed. Found {} image URLs", image_urls.len());
    Ok(image_urls)
}

async fn find_image_urls(tab: &Tab, num_images: usize, max_missed: usize) -> Result<Vec<String>> {
    let mut image_urls = Vec::new();
    let mut missed_count = 0;
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 10;

    let url_regex = Regex::new(r#"(https?://\S+\.(?:jpg|jpeg|png|gif|bmp))"#).unwrap();

    while image_urls.len() < num_images && missed_count < max_missed && attempts < MAX_ATTEMPTS {
        attempts += 1;
        println!("Attempt {} to extract image URLs...", attempts);

        // Scroll down to load more images
        tab.evaluate("window.scrollTo(0, document.body.scrollHeight);", false)?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let page_content = tab.get_content().context("Failed to get page content")?;

        let new_urls: Vec<String> = url_regex
            .captures_iter(&page_content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|url| !url.contains("encrypted") && !image_urls.contains(url))
            .collect();

        println!("Found {} new URLs", new_urls.len());

        if new_urls.is_empty() {
            missed_count += 1;
            println!("No new URLs found. Missed count: {}", missed_count);
        } else {
            missed_count = 0;
        }

        for url in new_urls {
            if image_urls.len() >= num_images {
                break;
            }
            println!("[INFO] #{} \t {}", image_urls.len() + 1, url);
            image_urls.push(url);
        }
    }

    if image_urls.is_empty() {
        Err(anyhow!("Failed to find any image URLs after {} attempts", MAX_ATTEMPTS))
    } else {
        Ok(image_urls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_google_image_urls() {
        let search_query = "rust programming language";
        let num_images = 10;
        let max_missed = 20;

        let result = get_google_image_urls(search_query, num_images, max_missed).await;

        match result {
            Ok(urls) => {
                assert!(!urls.is_empty(), "Should return at least one URL");
                assert!(urls.len() <= num_images, "Should return at most {} URLs", num_images);

                println!("Number of image URLs found: {}", urls.len());
                println!("Image URLs:");
                for (index, url) in urls.iter().enumerate() {
                    println!("{}. {}", index + 1, url);
                    assert!(url.starts_with("http"), "URL should start with http: {}", url);
                }
            }
            Err(e) => {
                eprintln!("Error occurred: {:?}", e);
                panic!("Function should return Ok result");
            }
        }
    }
}
