use googrs::search_md;
use std::env;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let write_to_file = (args.len() > 1 && args[1] == "--write-to-file") || args.len() == 0;
    let remove_links = args.len() > 2 && args[2] == "--remove-links";

    let results = search_md(
        "Best neighborhoods in San Francisco",
        3,
        "en",
        None,
        2,
        10,
        "active",
        true,
        None,
        0,
        remove_links // New argument
    ).await?;

    let mut file = File::create("search_results.md")?;
    for (index, result) in results.iter().enumerate() {
        writeln!(file, "# Result {}", index + 1)?;
        writeln!(file, "URL: {}", result.url)?;
        writeln!(file, "Title: {}", result.title)?;
        writeln!(file, "Description: {}", result.description)?;
        writeln!(file, "## Content:")?;
        writeln!(file, "{}", result.content)?;
        writeln!(file, "---")?;
    }
    println!("Results written to search_results.md");

    Ok(())
}
