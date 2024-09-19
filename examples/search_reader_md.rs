use googrs::{ search_reader, ReaderResult };
use htmd::{ HtmlToMarkdown, options::Options };
use std::env;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let write_to_file = args.len() > 1 && args[1] == "--write-to-file";

    let reader_results = search_reader(
        "Best restaurants in San Francisco",
        1,
        "en",
        None,
        2,
        10,
        "active",
        true,
        None,
        0
    ).await?;

    let markdown_results: Vec<ReaderResult> = reader_results
        .into_iter()
        .map(|result| {
            let converter = HtmlToMarkdown::builder()
                .options(Options {
                    link_style: htmd::options::LinkStyle::Referenced,
                    link_reference_style: htmd::options::LinkReferenceStyle::Shortcut,
                    ..Default::default()
                })
                .skip_tags(vec!["script", "style", "iframe", "img", "svg", "path"])
                .build();

            let markdown_content = converter
                .convert(&result.content)
                .unwrap_or_else(|_| result.content.clone());

            ReaderResult {
                url: result.url,
                title: result.title,
                description: result.description,
                content: markdown_content,
            }
        })
        .collect();

    if write_to_file {
        let mut file = File::create("reader_md_results.txt")?;
        for (index, result) in markdown_results.iter().enumerate() {
            writeln!(file, "# Result {}", index + 1)?;
            writeln!(file, "URL: {}", result.url)?;
            writeln!(file, "Title: {}", result.title)?;
            writeln!(file, "Description: {}", result.description)?;
            writeln!(file, "## Content:")?;
            writeln!(file, "{}", result.content)?;
            writeln!(file, "---")?;
        }
        println!("Results written to reader_md_results.txt");
    } else {
        for (index, result) in markdown_results.iter().enumerate() {
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
