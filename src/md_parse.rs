//! # Markdown Parser Module
//!
//! This module provides functionality to parse raw Markdown text into structured `MarkdownBlock`s.

use pulldown_cmark::{ Event, Parser, Tag };

/// Represents different block types in Markdown.
#[derive(Debug, Clone)]
pub enum MarkdownBlock {
    Heading {
        level: u32,
        text: String,
    },
    Paragraph(String),
    List {
        ordered: bool,
        items: Vec<String>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    BlockQuote(String),
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    // Additional block types can be added here
}

/// Struct for parsing Markdown text.
pub struct MarkdownParser<'a> {
    markdown: &'a str,
}

impl<'a> MarkdownParser<'a> {
    /// Creates a new `MarkdownParser`.
    ///
    /// # Arguments
    ///
    /// * `markdown` - The raw Markdown text to parse.
    pub fn new(markdown: &'a str) -> Self {
        MarkdownParser { markdown }
    }

    /// Parses the Markdown text into a vector of `MarkdownBlock`s.
    ///
    /// # Returns
    ///
    /// A `Vec<MarkdownBlock>` representing the parsed blocks.
    pub fn parse(&self) -> Vec<MarkdownBlock> {
        let parser = Parser::new_ext(self.markdown, pulldown_cmark::Options::all());

        let mut blocks = Vec::new();
        let mut current_block: Option<MarkdownBlock> = None;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match tag {
                        Tag::Heading { level, .. } => {
                            current_block = Some(MarkdownBlock::Heading {
                                level: level as u32,
                                text: String::new(),
                            });
                        }
                        Tag::Paragraph => {
                            current_block = Some(MarkdownBlock::Paragraph(String::new()));
                        }
                        Tag::List(ordered) => {
                            current_block = Some(MarkdownBlock::List {
                                ordered: ordered.is_some(),
                                items: Vec::new(),
                            });
                        }
                        Tag::CodeBlock(code_kind) => {
                            let language = match code_kind {
                                pulldown_cmark::CodeBlockKind::Fenced(lang) =>
                                    Some(lang.to_string()),
                                pulldown_cmark::CodeBlockKind::Indented => None,
                            };
                            current_block = Some(MarkdownBlock::CodeBlock {
                                language,
                                code: String::new(),
                            });
                        }
                        Tag::BlockQuote(_) => {
                            current_block = Some(MarkdownBlock::BlockQuote(String::new()));
                        }
                        Tag::Table(_alignments) => {
                            current_block = Some(MarkdownBlock::Table {
                                headers: Vec::new(),
                                rows: Vec::new(),
                            });
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    if let Some(ref mut block) = current_block {
                        match block {
                            | MarkdownBlock::Heading { text: ref mut t, .. }
                            | MarkdownBlock::Paragraph(ref mut t)
                            | MarkdownBlock::BlockQuote(ref mut t) => {
                                t.push_str(&text);
                            }
                            MarkdownBlock::List { items, .. } => {
                                items.push(text.to_string());
                            }
                            MarkdownBlock::CodeBlock { code, .. } => {
                                code.push_str(&text);
                            }
                            MarkdownBlock::Table { headers, rows } => {
                                let cells: Vec<String> = text
                                    .split('|')
                                    .map(str::trim)
                                    .filter(|s| !s.is_empty())
                                    .map(String::from)
                                    .collect();
                                if headers.is_empty() {
                                    headers.extend(cells);
                                } else {
                                    rows.push(cells);
                                }
                            }
                        }
                    }
                }
                Event::End(_tag) => {
                    if let Some(block) = current_block.take() {
                        blocks.push(block);
                    }
                }
                _ => {}
            }
        }

        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let markdown = "# Heading 1\nSome paragraph text.";
        let parser = MarkdownParser::new(markdown);
        let blocks = parser.parse();

        assert_eq!(blocks.len(), 2);
        match blocks[0] {
            MarkdownBlock::Heading { level, ref text } => {
                assert_eq!(level, 1);
                assert_eq!(text, "Heading 1");
            }
            _ => panic!("Expected Heading block"),
        }
    }

    #[test]
    fn test_parse_paragraph() {
        let markdown = "This is a paragraph.";
        let parser = MarkdownParser::new(markdown);
        let blocks = parser.parse();

        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Paragraph(text) => {
                assert_eq!(text, "This is a paragraph.");
            }
            _ => panic!("Expected Paragraph block"),
        }
    }

    // Add more tests as needed
}
