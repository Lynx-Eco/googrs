use crate::md_parse::MarkdownBlock;

pub struct MarkdownReassembler;

impl MarkdownReassembler {
    pub fn new() -> Self {
        MarkdownReassembler
    }

    pub fn reassemble(&self, blocks: &[MarkdownBlock]) -> String {
        let mut markdown = String::new();

        for block in blocks {
            match block {
                MarkdownBlock::Heading { level, text } => {
                    for _ in 0..*level {
                        markdown.push('#');
                    }
                    markdown.push(' ');
                    markdown.push_str(text);
                    markdown.push_str("\n\n");
                }
                MarkdownBlock::Paragraph(text) => {
                    markdown.push_str(text);
                    markdown.push_str("\n\n");
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
                    markdown.push_str("```");
                    if let Some(lang) = language {
                        markdown.push_str(lang);
                    }
                    markdown.push('\n');
                    markdown.push_str(code);
                    markdown.push('\n');
                    markdown.push_str("```\n\n");
                }
                MarkdownBlock::BlockQuote(text) => {
                    markdown.push_str(&format!("> {}\n\n", text));
                }
                MarkdownBlock::Table { headers, rows } => {
                    // Simplified table reassembly
                    let header_line = headers.join(" | ");
                    let separator = headers
                        .iter()
                        .map(|_| "---")
                        .collect::<Vec<_>>()
                        .join(" | ");
                    markdown.push_str(&format!("| {} |\n", header_line));
                    markdown.push_str(&format!("| {} |\n", separator));
                    for row in rows {
                        let row_line = row.join(" | ");
                        markdown.push_str(&format!("| {} |\n", row_line));
                    }
                    markdown.push('\n');
                }
            }
        }

        markdown
    }
}
