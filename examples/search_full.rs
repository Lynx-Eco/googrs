use googrs::search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = search(
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

    println!("Full Search Results:");
    for (i, result) in results.iter().enumerate() {
        println!("{}. {}", i + 1, result.title);
        println!("   URL: {}", result.url);
        println!("   Description: {}", result.description);
        println!();
    }

    Ok(())
}
