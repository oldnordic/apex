//! APEX Parser
//!
//! Parses token stream into ApexDocument AST.

use crate::ast::{ApexDocument, Block, Span};
use crate::errors::ApexResult;
use crate::parser::lexer::{Lexer, Token, ParseMode, ParseFix};

/// Parse APEX string into document AST (strict mode)
pub fn parse_str(input: &str) -> ApexResult<ApexDocument> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize_all()?;
    parse_tokens(&tokens)
}

/// Parse result with fixes from tolerant mode
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub document: ApexDocument,
    pub fixes: Vec<ParseFix>,
}

/// Parse APEX string with specified mode, returning fixes if any
pub fn parse_str_with_mode(input: &str, mode: ParseMode) -> ApexResult<ParseResult> {
    let mut lexer = Lexer::with_mode(input, mode);
    let tokens = lexer.tokenize_all()?;
    let document = parse_tokens(&tokens)?;
    Ok(ParseResult {
        document,
        fixes: lexer.fixes,
    })
}

/// Parse token stream into document AST
fn parse_tokens(tokens: &[Token]) -> ApexResult<ApexDocument> {
    let mut blocks = Vec::new();
    let mut idx = 0;

    while idx < tokens.len() {
        match &tokens[idx] {
            Token::Eof => break,

            Token::BlockHeader(kind, header_span) => {
                // Start collecting block content
                let start_line = header_span.start_line;
                let mut lines = Vec::new();
                let mut end_line = start_line;
                idx += 1;

                // Collect all lines until next header or EOF
                while idx < tokens.len() {
                    match &tokens[idx] {
                        Token::Line(content, span) => {
                            lines.push(content.clone());
                            end_line = span.end_line;
                            idx += 1;
                        }
                        Token::BlockHeader(_, _) | Token::Eof => break,
                    }
                }

                let span = Span::new(start_line, end_line);
                blocks.push(Block::new(*kind, lines, span));
            }

            Token::Line(content, _span) => {
                // Lines before first header - skip or error?
                // Per spec, we'll skip leading non-block content (whitespace, comments)
                if !content.trim().is_empty() {
                    // Non-empty line before any block - this is likely an error
                    // but for tolerant parsing, we skip it
                    // TODO: Consider strict mode that errors here
                }
                idx += 1;
            }
        }
    }

    Ok(ApexDocument::with_blocks(blocks))
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Allow unknown block types (skip them)
    pub allow_unknown_blocks: bool,
    /// Allow content before first block
    pub allow_leading_content: bool,
    /// Strict mode - fail on any irregularity
    pub strict: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            allow_unknown_blocks: false,
            allow_leading_content: true,
            strict: false,
        }
    }
}

impl ParserConfig {
    /// Strict parsing mode
    pub fn strict() -> Self {
        Self {
            allow_unknown_blocks: false,
            allow_leading_content: false,
            strict: true,
        }
    }

    /// Tolerant parsing mode
    pub fn tolerant() -> Self {
        Self {
            allow_unknown_blocks: true,
            allow_leading_content: true,
            strict: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_document() {
        let input = "TASK\nImplement the thing";
        let doc = parse_str(input).unwrap();

        assert_eq!(doc.blocks.len(), 1);
        assert_eq!(doc.task().unwrap().content(), "Implement the thing");
    }

    #[test]
    fn test_multi_block_document() {
        let input = r#"TASK
Implement feature X

GOALS
Feature works correctly
All tests pass

PLAN
Step 1: Analyze requirements
Step 2: Write code
Step 3: Test

CONSTRAINTS
No breaking changes
Must be backward compatible"#;

        let doc = parse_str(input).unwrap();

        assert_eq!(doc.blocks.len(), 4);
        assert!(doc.task().is_some());
        assert!(doc.goals().is_some());
        assert!(doc.plan().is_some());
        assert!(doc.constraints().is_some());

        let plan = doc.plan().unwrap();
        assert_eq!(plan.content_lines().len(), 3);
    }

    #[test]
    fn test_all_block_types() {
        let input = r#"TASK
Do something

GOALS
Succeed

PLAN
Step 1

CONSTRAINTS
Be safe

VALIDATION
Check results

TOOLS
tool_name(arg)

DIFF
file.rs: +10 -5

CONTEXT
Background info

META
version=1.0"#;

        let doc = parse_str(input).unwrap();

        assert!(doc.task().is_some());
        assert!(doc.goals().is_some());
        assert!(doc.plan().is_some());
        assert!(doc.constraints().is_some());
        assert!(doc.validation().is_some());
        assert!(doc.tools().is_some());
        assert!(doc.diff().is_some());
        assert!(doc.context().is_some());
        assert!(doc.meta().is_some());
    }

    #[test]
    fn test_leading_whitespace() {
        let input = "\n\n  \nTASK\nDo it";
        let doc = parse_str(input).unwrap();

        assert_eq!(doc.blocks.len(), 1);
        assert!(doc.task().is_some());
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let doc = parse_str(input).unwrap();
        assert!(doc.blocks.is_empty());
    }
}
