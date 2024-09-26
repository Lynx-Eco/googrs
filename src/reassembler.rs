//! # Markdown Reassembler Module
//!
//! This module provides functionality to reassemble `MarkdownBlock`s into clean Markdown text.

use crate::md_parse::MarkdownBlock;

/// Struct for reassembling Markdown blocks into Markdown text.
pub struct MarkdownReassembler;

impl MarkdownReassembler {
    /// Creates a new `MarkdownReassembler`.
    pub fn new() -> Self {
        MarkdownReassembler
    }

    /// Reassembles a slice of `MarkdownBlock`s into a Markdown `String`.
    ///
    /// # Arguments
    ///
    /// * `blocks` - A slice of `MarkdownBlock`s to reassemble.
    ///
    /// # Returns
    ///
    /// A `String` containing the reassembled Markdown.
    pub fn reassemble(&self, blocks: &[MarkdownBlock]) -> String {
        let mut markdown = String::new();

        for block in blocks {
            match block {
                MarkdownBlock::Heading { level, text } => {
                    markdown.push_str(&format!("{} {}\n\n", "#".repeat(*level as usize), text));
                }
                MarkdownBlock::Paragraph(text) => {
                    markdown.push_str(&format!("{}\n\n", text));
                }
                MarkdownBlock::List { ordered, items } => {
                    for (i, item) in items.iter().enumerate() {
                        if *ordered {
                            markdown.push_str(&format!("{}. {}\n", i + 1, item));
                        } else {
                            markdown.push_str(&format!("- {}\n", item));
                        }
                    }
                    markdown.push('\n');
                }
                MarkdownBlock::CodeBlock { language, code } => {
                    if let Some(lang) = language {
                        markdown.push_str(&format!("```{}\n{}\n```\n\n", lang, code));
                    } else {
                        markdown.push_str(&format!("```\n{}\n```\n\n", code));
                    }
                }
                MarkdownBlock::BlockQuote(text) => {
                    markdown.push_str(&format!("> {}\n\n", text));
                }
                MarkdownBlock::Table { headers, rows } => {
                    if !headers.is_empty() {
                        markdown.push_str(&format!("| {} |\n", headers.join(" | ")));
                        markdown.push_str(
                            &format!(
                                "|{}|\n",
                                headers
                                    .iter()
                                    .map(|_| "---")
                                    .collect::<Vec<_>>()
                                    .join("|")
                            )
                        );
                    }
                    for row in rows {
                        markdown.push_str(&format!("| {} |\n", row.join(" | ")));
                    }
                    markdown.push('\n');
                }
            }
        }

        markdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::md_parse::MarkdownBlock;

    #[test]
    fn test_reassemble_heading() {
        let reassembler = MarkdownReassembler::new();
        let blocks = vec![MarkdownBlock::Heading {
            level: 2,
            text: "Subheading".to_string(),
        }];
        let markdown = reassembler.reassemble(&blocks);
        assert_eq!(markdown, "## Subheading\n\n");
    }

    #[test]
    fn test_reassemble_paragraph() {
        let reassembler = MarkdownReassembler::new();
        let blocks = vec![MarkdownBlock::Paragraph("This is a paragraph.".to_string())];
        let markdown = reassembler.reassemble(&blocks);
        assert_eq!(markdown, "This is a paragraph.\n\n");
    }

    #[test]
    fn test_reassemble_list() {
        let reassembler = MarkdownReassembler::new();
        let blocks = vec![MarkdownBlock::List {
            ordered: false,
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }];
        let markdown = reassembler.reassemble(&blocks);
        assert_eq!(markdown, "- Item 1\n- Item 2\n\n");
    }

    // Add more tests as needed
}
