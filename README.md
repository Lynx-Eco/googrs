# googlesearch-rs

A Rust library for searching Google, inspired by the Python `googlesearch` library.

## Features

- Search Google and retrieve results asynchronously
- Customizable search parameters (language, region, safe search, etc.)
- Proxy support
- Configurable sleep intervals between requests
- SSL verification options

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
googlesearch-rs = "0.1.0"
```
## Examples 

```bash
cargo run --example search_full
cargo run --example search_url
```

## Usage

```rust
use googlesearch_rs::search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = search(
        "Rust programming",
        10,
        "en",
        None,
        2,
        5,
        "active",
        true,
        None,
        0
    ).await?;

    for result in results {
        println!("URL: {}", result.url);
        println!("Title: {}", result.title);
        println!("Description: {}", result.description);
        println!("---");
    }

    Ok(())
}

## Parameters

- `term`: The search query string
- `num_results`: Number of results to retrieve
- `lang`: Language code (e.g., "en" for English)
- `proxy`: Optional proxy URL
- `sleep_interval`: Time to wait between requests (in seconds)
- `timeout`: Request timeout (in seconds)
- `safe`: Safe search setting ("active" or "off")
- `ssl_verify`: Whether to verify SSL certificates
- `region`: Optional region code for localized results
- `start_num`: Starting index for search results

## Usage

To remove links from the Markdown output, use the `remove_links` option:

```rust
let
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This library is for educational purposes only. Please read and respect Google's Terms of Service.
