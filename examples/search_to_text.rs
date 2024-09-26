use googrs::search_to_text;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    search_to_text(
        "Best restaurants in San Francisco",
        1,
        "en",
        None,
        2,
        10,
        "active",
        true,
        None,
        0,
        "search_results2.md",
        80
    ).await?;

    println!("Search results have been written to search_results.txt");

    Ok(())
}
