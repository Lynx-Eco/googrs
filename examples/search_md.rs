use googrs::SearchMdBuilder;
use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let write_to_file = (args.len() > 1 && args[1] == "--write-to-file") || args.is_empty();

    let start_time = Instant::now();

    let search_md = SearchMdBuilder::new()
        .num_results(2)
        .lang("en".to_string())
        .sleep_interval(0)
        .timeout(10)
        .safe("active".to_string())
        .ssl_verify(true)
        .start_num(0)
        .build();

    let results = search_md.search_md("Best neighborhoods in San Francisco").await?;
    let elapsed_time = start_time.elapsed();
    println!("Search completed in {:.2?}", elapsed_time);

    if true {
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
    } else {
        for (index, result) in results.iter().enumerate() {
            println!("# Result {}", index + 1);
            println!("URL: {}", result.url);
            println!("Title: {}", result.title);
            println!("Description: {}", result.description);
            println!("## Content:");
            println!("{}", result.content);
            println!("---");
        }
    }

    Ok(())
}
