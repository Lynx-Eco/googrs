use pulldown_cmark::{ Parser, Event, Tag };
use std::str::FromStr;

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
    // Add other block types as needed
}

pub struct MarkdownParser<'a> {
    markdown: &'a str,
}

impl<'a> MarkdownParser<'a> {
    pub fn new(markdown: &'a str) -> Self {
        MarkdownParser { markdown }
    }

    pub fn parse(&self) -> Vec<MarkdownBlock> {
        let parser = Parser::new_ext(self.markdown, pulldown_cmark::Options::all());

        let mut blocks = Vec::new();
        let mut current_block = None;

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
                    if let Some(block) = &mut current_block {
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
                                // Simplified: Assign all text to cells
                                // Ideally, need to parse table cells properly
                                let cells: Vec<String> = text
                                    .split('|')
                                    .map(str::trim)
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
                Event::End(tag) => {
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
