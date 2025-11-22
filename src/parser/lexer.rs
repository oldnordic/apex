//! APEX Lexer
//!
//! Tokenizes APEX input into block headers and content lines.

use crate::ast::{BlockKind, Span};
use crate::errors::ApexResult;

/// Token types produced by lexer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Block header (e.g., "TASK", "PLAN")
    BlockHeader(BlockKind, Span),
    /// Content line (non-header text)
    Line(String, Span),
    /// End of input
    Eof,
}

impl Token {
    /// Get span if token has one
    pub fn span(&self) -> Option<&Span> {
        match self {
            Token::BlockHeader(_, span) => Some(span),
            Token::Line(_, span) => Some(span),
            Token::Eof => None,
        }
    }
}

/// Parser mode per APEX v1.1 spec
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseMode {
    /// v1.0-conformant: headers MUST be uppercase, violations cause hard errors
    #[default]
    Strict,
    /// v1.1 tolerant: accept lowercase/mixed-case headers, record fixes
    Tolerant,
}

/// Parse fix recorded in tolerant mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFix {
    pub line: usize,
    pub description: String,
}

/// Lexer state
pub struct Lexer<'a> {
    /// Lines split from input
    lines: Vec<&'a str>,
    /// Current line index (0-based)
    line_idx: usize,
    /// Parser mode (strict or tolerant)
    mode: ParseMode,
    /// Fixes applied in tolerant mode
    pub fixes: Vec<ParseFix>,
    /// Phantom to preserve lifetime
    _phantom: std::marker::PhantomData<&'a str>,
}

impl<'a> Lexer<'a> {
    /// Create new lexer from input string (strict mode)
    pub fn new(input: &'a str) -> Self {
        Self::with_mode(input, ParseMode::Strict)
    }

    /// Create new lexer with specified mode
    pub fn with_mode(input: &'a str, mode: ParseMode) -> Self {
        let lines: Vec<&str> = input.lines().collect();
        Self {
            lines,
            line_idx: 0,
            mode,
            fixes: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Check if at end of input
    pub fn is_eof(&self) -> bool {
        self.line_idx >= self.lines.len()
    }

    /// Current line number (1-indexed for user display)
    pub fn current_line_number(&self) -> usize {
        self.line_idx + 1
    }

    /// Peek at current line without consuming
    pub fn peek_line(&self) -> Option<&'a str> {
        self.lines.get(self.line_idx).copied()
    }

    /// Check if line is a block header (strict mode - uppercase only)
    fn is_block_header_strict(line: &str) -> Option<BlockKind> {
        let trimmed = line.trim();

        // Block headers are uppercase identifiers alone on a line
        // Per EBNF: block_identifier = "TASK" | "GOALS" | ... ;
        if trimmed.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
            BlockKind::from_str(trimmed)
        } else {
            None
        }
    }

    /// Check if line is a block header (tolerant mode - any case)
    fn is_block_header_tolerant(line: &str) -> Option<(BlockKind, bool)> {
        let trimmed = line.trim();

        // In tolerant mode, accept any case
        if let Some(kind) = BlockKind::from_str(trimmed) {
            // Check if it needed case-fixing
            let was_fixed = !trimmed.chars().all(|c| c.is_ascii_uppercase() || c == '_');
            Some((kind, was_fixed))
        } else {
            None
        }
    }

    /// Check if line is a block header based on current mode
    fn check_block_header(&mut self, line: &str, line_num: usize) -> Option<BlockKind> {
        match self.mode {
            ParseMode::Strict => Self::is_block_header_strict(line),
            ParseMode::Tolerant => {
                if let Some((kind, was_fixed)) = Self::is_block_header_tolerant(line) {
                    if was_fixed {
                        self.fixes.push(ParseFix {
                            line: line_num,
                            description: format!(
                                "Normalized header '{}' to '{}'",
                                line.trim(),
                                kind.as_str()
                            ),
                        });
                    }
                    Some(kind)
                } else {
                    None
                }
            }
        }
    }

    /// Get next token
    pub fn next_token(&mut self) -> ApexResult<Token> {
        if self.is_eof() {
            return Ok(Token::Eof);
        }

        let line = self.lines[self.line_idx];
        let line_num = self.current_line_number();
        self.line_idx += 1;

        // Check if this is a block header
        if let Some(kind) = self.check_block_header(line, line_num) {
            return Ok(Token::BlockHeader(kind, Span::line(line_num)));
        }

        // Otherwise it's a content line
        Ok(Token::Line(line.to_string(), Span::line(line_num)))
    }

    /// Tokenize entire input into token vector
    pub fn tokenize_all(&mut self) -> ApexResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if matches!(token, Token::Eof) {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    /// Reset lexer to beginning
    pub fn reset(&mut self) {
        self.line_idx = 0;
        self.fixes.clear();
    }

    /// Get current parse mode
    pub fn mode(&self) -> ParseMode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_header_detection_strict() {
        assert_eq!(Lexer::is_block_header_strict("TASK"), Some(BlockKind::Task));
        assert_eq!(Lexer::is_block_header_strict("  PLAN  "), Some(BlockKind::Plan));
        assert_eq!(Lexer::is_block_header_strict("task"), None); // lowercase not valid in strict
        assert_eq!(Lexer::is_block_header_strict("TASK:"), None); // colon not valid
        assert_eq!(Lexer::is_block_header_strict("NOT_A_BLOCK"), None);
    }

    #[test]
    fn test_block_header_detection_tolerant() {
        // Tolerant mode accepts any case
        assert_eq!(Lexer::is_block_header_tolerant("TASK"), Some((BlockKind::Task, false)));
        assert_eq!(Lexer::is_block_header_tolerant("task"), Some((BlockKind::Task, true)));
        assert_eq!(Lexer::is_block_header_tolerant("Task"), Some((BlockKind::Task, true)));
        assert_eq!(Lexer::is_block_header_tolerant("  plan  "), Some((BlockKind::Plan, true)));
        assert_eq!(Lexer::is_block_header_tolerant("TASK:"), None); // colon still not valid
    }

    #[test]
    fn test_tolerant_mode_records_fixes() {
        let input = "task\nImplement feature\nplan\nStep 1";
        let mut lexer = Lexer::with_mode(input, ParseMode::Tolerant);
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens.len(), 5); // task, line, plan, line, EOF
        assert!(matches!(&tokens[0], Token::BlockHeader(BlockKind::Task, _)));
        assert!(matches!(&tokens[2], Token::BlockHeader(BlockKind::Plan, _)));

        // Check fixes were recorded
        assert_eq!(lexer.fixes.len(), 2);
        assert!(lexer.fixes[0].description.contains("task"));
        assert!(lexer.fixes[1].description.contains("plan"));
    }

    #[test]
    fn test_simple_tokenize() {
        let input = "TASK\nImplement feature\nPLAN\nStep 1\nStep 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens.len(), 6); // TASK, line, PLAN, line, line, EOF

        assert!(matches!(&tokens[0], Token::BlockHeader(BlockKind::Task, _)));
        assert!(matches!(&tokens[1], Token::Line(s, _) if s == "Implement feature"));
        assert!(matches!(&tokens[2], Token::BlockHeader(BlockKind::Plan, _)));
        assert!(matches!(&tokens[3], Token::Line(s, _) if s == "Step 1"));
        assert!(matches!(&tokens[4], Token::Line(s, _) if s == "Step 2"));
        assert!(matches!(&tokens[5], Token::Eof));
    }

    #[test]
    fn test_empty_lines_preserved() {
        let input = "TASK\n\nLine after empty";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(&tokens[1], Token::Line(s, _) if s.is_empty()));
    }
}
