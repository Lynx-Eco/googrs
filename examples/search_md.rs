use googrs::search_md;
use std::env;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let write_to_file = args.len() > 1 && args[1] == "--write-to-file";
    let remove_links = args.len() > 2 && args[2] == "--remove-links";

    let results = search_md(
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
        remove_links // New argument
    ).await?;

    if write_to_file {
        let mut file = File::create("search_results.txt")?;
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
    } else {
        for (index, result) in results.iter().enumerate() {
            println!("Result {}", index + 1);
            println!("URL: {}", result.url);
            println!("Title: {}", result.title);
            println!("Description: {}", result.description);
            println!("Content:");
            println!("{}", result.content);
            println!("---");
        }
    }

    Ok(())
}
