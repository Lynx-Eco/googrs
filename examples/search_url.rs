use googrs::search_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = search_url(
        "Best restaurants in San Francisco",
        5,
        "en",
        None,
        2,
        10,
        "active",
        true,
        None,
        0
    ).await?;

    println!("Search Results (URLs only):");
    for (i, url) in results.iter().enumerate() {
        println!("{}. {}", i + 1, url);
    }

    Ok(())
}
