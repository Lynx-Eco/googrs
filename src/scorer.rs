//! # Markdown Scorer Module
//!
//! This module provides functionality to score and evaluate `MarkdownBlock`s based on their importance.

use crate::md_parse::MarkdownBlock;
use regex::Regex;

/// Returns a default list of keywords used for scoring.
pub fn default_keywords() -> Vec<String> {
    vec![
        "fn",
        "let",
        "mut",
        "const",
        "static",
        "use",
        "mod",
        "struct",
        "enum",
        "trait",
        "impl",
        "pub",
        "crate",
        "super",
        "self",
        "where",
        "async",
        "await",
        "move",
        "type",
        "dyn",
        "for",
        "if",
        "else",
        "while",
        "loop",
        "match",
        "return",
        "break",
        "continue",
        "unsafe"
    ]
        .into_iter()
        .map(String::from)
        .collect()
}

/// Represents a scored Markdown block.
#[derive(Debug)]
pub struct ScoredBlock<'a> {
    pub block: &'a MarkdownBlock,
    pub score: f32,
}

/// Struct for scoring Markdown blocks.
pub struct MarkdownScorer {
    keywords: Vec<String>,
}

impl MarkdownScorer {
    /// Creates a new `MarkdownScorer` with default keywords.
    pub fn new() -> Self {
        MarkdownScorer {
            keywords: default_keywords(),
        }
    }

    /// Scores a list of `MarkdownBlock`s.
    ///
    /// # Arguments
    ///
    /// * `blocks` - A slice of `MarkdownBlock`s to score.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing references to `MarkdownBlock`s and their scores.
    pub fn score_blocks<'a>(&self, blocks: &'a [MarkdownBlock]) -> Vec<(&'a MarkdownBlock, f32)> {
        blocks
            .iter()
            .map(|block| (block, self.score_block(block)))
            .collect()
    }

    /// Calculates the threshold score based on the average score.
    ///
    /// # Arguments
    ///
    /// * `scored_blocks` - A slice of tuples containing `MarkdownBlock`s and their scores.
    ///
    /// # Returns
    ///
    /// A `f32` representing the threshold.
    pub fn calculate_threshold(&self, scored_blocks: &[(&MarkdownBlock, f32)]) -> f32 {
        let sum: f32 = scored_blocks
            .iter()
            .map(|&(_, score)| score)
            .sum();
        let count = scored_blocks.len() as f32;
        if count == 0.0 {
            0.0
        } else {
            (sum / count) * 0.5
        } // Threshold at 50% of average
    }

    /// Scores an individual `MarkdownBlock`.
    ///
    /// # Arguments
    ///
    /// * `block` - A reference to a `MarkdownBlock` to score.
    ///
    /// # Returns
    ///
    /// A `f32` representing the score.
    fn score_block(&self, block: &MarkdownBlock) -> f32 {
        let mut score = 0.0;

        // Text Length Score
        let text_length_score = match block {
            MarkdownBlock::Heading { text, .. } => text.len() as f32,
            MarkdownBlock::Paragraph(text) => text.len() as f32,
            MarkdownBlock::List { items, .. } => items.join(" ").len() as f32,
            MarkdownBlock::CodeBlock { code, .. } => code.len() as f32,
            MarkdownBlock::BlockQuote(text) => text.len() as f32,
            MarkdownBlock::Table { headers, rows } => {
                (headers.len() as f32) + (rows.len() as f32) * (headers.len() as f32)
            }
        };
        score += text_length_score;

        // Heading Importance Score
        let heading_score = match block {
            MarkdownBlock::Heading { level, .. } =>
                match level {
                    1 => 1000.0, // Very high score for top-level headings
                    2 => 10.0,
                    3 => 5.0,
                    _ => 2.0,
                }
            _ => 0.0,
        };
        score += heading_score;

        // Keyword Density
        let keyword_density = self.compute_keyword_density(block);
        score += keyword_density * 2.0; // Weight

        // Link Density Penalty
        let link_density = self.compute_link_density(block);
        score -= link_density * 2.0; // Penalty Weight

        // Formatting Indicators Bonus
        let emphasis_bonus = self.compute_emphasis_bonus(block);
        score += emphasis_bonus;

        // Content-Type Bonus/Penalty
        let content_type_score = self.content_type_bonus(block);
        score += content_type_score;

        score
    }

    /// Computes the keyword density of a `MarkdownBlock`.
    ///
    /// # Arguments
    ///
    /// * `block` - The `MarkdownBlock` to analyze.
    ///
    /// # Returns
    ///
    /// A `f32` representing the keyword density.
    fn compute_keyword_density(&self, block: &MarkdownBlock) -> f32 {
        let text = match block {
            | MarkdownBlock::Heading { text, .. }
            | MarkdownBlock::Paragraph(text)
            | MarkdownBlock::BlockQuote(text) => text,
            MarkdownBlock::List { items, .. } => &items.join(" "),
            MarkdownBlock::Table { headers, rows } => {
                let mut all_text = headers.join(" ");
                all_text.push_str(
                    &rows
                        .iter()
                        .flatten()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                &all_text.to_string()
            }
            MarkdownBlock::CodeBlock { .. } => {
                return 0.0;
            } // Assume code blocks have no keyword density
        };

        let total_words = text.split_whitespace().count();
        if total_words == 0 {
            return 0.0;
        }

        let keyword_count: usize = self.keywords
            .iter()
            .map(|keyword| text.to_lowercase().matches(&keyword.to_lowercase()).count())
            .sum();

        (keyword_count as f32) / (total_words as f32)
    }

    /// Computes the link density of a `MarkdownBlock`.
    ///
    /// # Arguments
    ///
    /// * `block` - The `MarkdownBlock` to analyze.
    ///
    /// # Returns
    ///
    /// A `f32` representing the link density.
    fn compute_link_density(&self, block: &MarkdownBlock) -> f32 {
        let link_regex = Regex::new(r"\[(?P<text>.*?)\]\(.*?\)").unwrap();

        let (link_length, text_length) = match block {
            | MarkdownBlock::Heading { text, .. }
            | MarkdownBlock::Paragraph(text)
            | MarkdownBlock::BlockQuote(text) => {
                process_text(text, &link_regex)
            }
            MarkdownBlock::List { items, .. } =>
                items
                    .iter()
                    .map(|item| process_text(item, &link_regex))
                    .fold((0, 0), |acc, (l, t)| (acc.0 + l, acc.1 + t)),
            MarkdownBlock::Table { headers, rows } => {
                let header_stats = headers
                    .iter()
                    .map(|h| process_text(h, &link_regex))
                    .fold((0, 0), |acc, (l, t)| (acc.0 + l, acc.1 + t));
                let row_stats = rows
                    .iter()
                    .flat_map(|row| row.iter().map(|cell| process_text(cell, &link_regex)))
                    .fold((0, 0), |acc, (l, t)| (acc.0 + l, acc.1 + t));
                (header_stats.0 + row_stats.0, header_stats.1 + row_stats.1)
            }
            MarkdownBlock::CodeBlock { .. } => (0, 0), // Assume code blocks have no link density
        };

        if text_length == 0 {
            0.0
        } else {
            (link_length as f32) / (text_length as f32)
        }
    }

    /// Computes the emphasis bonus for a `MarkdownBlock`.
    ///
    /// # Arguments
    ///
    /// * `block` - The `MarkdownBlock` to analyze.
    ///
    /// # Returns
    ///
    /// A `f32` representing the emphasis bonus.
    fn compute_emphasis_bonus(&self, block: &MarkdownBlock) -> f32 {
        match block {
            MarkdownBlock::Heading { text, level } => {
                let emphasis = match level {
                    1 => 3.0,
                    2 => 2.0,
                    _ => 1.0,
                };
                emphasis * (text.len() as f32) * 0.1
            }
            MarkdownBlock::Paragraph(text) | MarkdownBlock::BlockQuote(text) => {
                (text.len() as f32) * 0.05
            }
            MarkdownBlock::List { items, .. } => {
                items
                    .iter()
                    .map(|item| (item.len() as f32) * 0.075)
                    .sum()
            }
            _ => 0.0,
        }
    }

    /// Computes the content-type bonus for a `MarkdownBlock`.
    ///
    /// # Arguments
    ///
    /// * `block` - The `MarkdownBlock` to analyze.
    ///
    /// # Returns
    ///
    /// A `f32` representing the content-type bonus or penalty.
    fn content_type_bonus(&self, block: &MarkdownBlock) -> f32 {
        match block {
            MarkdownBlock::CodeBlock { .. } => 0.5, // Bonus for code blocks
            MarkdownBlock::List { .. } => 1.0, // Bonus for lists
            _ => 0.0,
        }
    }
}

/// Processes text to calculate link lengths and total text lengths.
///
/// # Arguments
///
/// * `text` - The text to process.
/// * `link_regex` - The compiled regex for link detection.
///
/// # Returns
///
/// A tuple containing the total length of link texts and the total text length.
pub fn process_text(text: &str, link_regex: &Regex) -> (usize, usize) {
    let mut link_length = 0;
    let text_length = text.len();

    for cap in link_regex.captures_iter(text) {
        if let Some(link_text) = cap.name("text") {
            link_length += link_text.as_str().len();
        }
    }

    (link_length, text_length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::md_parse::MarkdownBlock;

    #[test]
    fn test_compute_keyword_density() {
        let scorer = MarkdownScorer::new();
        let block = MarkdownBlock::Paragraph("fn main() { let x = 5; }".to_string());
        let density = scorer.compute_keyword_density(&block);
        assert_eq!(density, 4.0 / 6.0); // 4 keywords out of 6 words
    }

    #[test]
    fn test_compute_link_density() {
        let scorer = MarkdownScorer::new();
        let block = MarkdownBlock::Paragraph(
            "This is a [link](http://example.com) in text.".to_string()
        );
        let density = scorer.compute_link_density(&block);
        assert_eq!(density, 4.0 / 9.0); // "link" is 4 characters in 9 total
    }

    // Add more tests as needed
}
