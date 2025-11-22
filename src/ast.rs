//! APEX Abstract Syntax Tree
//!
//! Core data structures for APEX documents, blocks, and spans.

use serde::{Deserialize, Serialize};

/// Source location span for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Start line (1-indexed)
    pub start_line: usize,
    /// End line (1-indexed, inclusive)
    pub end_line: usize,
    /// Start column (1-indexed)
    pub start_col: usize,
    /// End column (1-indexed)
    pub end_col: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
            start_col: 1,
            end_col: 1,
        }
    }

    /// Single-line span
    pub fn line(line: usize) -> Self {
        Self::new(line, line)
    }

    /// Merge two spans into one covering both
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start_line: self.start_line.min(other.start_line),
            end_line: self.end_line.max(other.end_line),
            start_col: if self.start_line <= other.start_line {
                self.start_col
            } else {
                other.start_col
            },
            end_col: if self.end_line >= other.end_line {
                self.end_col
            } else {
                other.end_col
            },
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

/// Block type identifiers (uppercase keywords)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockKind {
    /// TASK - Required. Single-line task description.
    Task,
    /// GOALS - Optional. Success criteria.
    Goals,
    /// PLAN - Optional. Ordered execution steps.
    Plan,
    /// CONSTRAINTS - Optional. Execution constraints.
    Constraints,
    /// VALIDATION - Optional. Post-execution checks.
    Validation,
    /// TOOLS - Optional. Tool declarations.
    Tools,
    /// DIFF - Optional. Expected file changes.
    Diff,
    /// CONTEXT - Optional. Pre-loaded context.
    Context,
    /// META - Optional. Metadata key-value pairs.
    Meta,
}

impl BlockKind {
    /// Parse block kind from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TASK" => Some(BlockKind::Task),
            "GOALS" => Some(BlockKind::Goals),
            "PLAN" => Some(BlockKind::Plan),
            "CONSTRAINTS" => Some(BlockKind::Constraints),
            "VALIDATION" => Some(BlockKind::Validation),
            "TOOLS" => Some(BlockKind::Tools),
            "DIFF" => Some(BlockKind::Diff),
            "CONTEXT" => Some(BlockKind::Context),
            "META" => Some(BlockKind::Meta),
            _ => None,
        }
    }

    /// Get canonical uppercase name
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockKind::Task => "TASK",
            BlockKind::Goals => "GOALS",
            BlockKind::Plan => "PLAN",
            BlockKind::Constraints => "CONSTRAINTS",
            BlockKind::Validation => "VALIDATION",
            BlockKind::Tools => "TOOLS",
            BlockKind::Diff => "DIFF",
            BlockKind::Context => "CONTEXT",
            BlockKind::Meta => "META",
        }
    }

    /// Check if block is required (must be present)
    pub fn is_required(&self) -> bool {
        matches!(self, BlockKind::Task)
    }

    /// Check if block can be empty
    pub fn allows_empty(&self) -> bool {
        matches!(self, BlockKind::Context | BlockKind::Meta)
    }
}

impl std::fmt::Display for BlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single block in an APEX document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// Block type
    pub kind: BlockKind,
    /// Raw content lines (without the header)
    pub lines: Vec<String>,
    /// Source location
    pub span: Span,
}

impl Block {
    /// Create a new block
    pub fn new(kind: BlockKind, lines: Vec<String>, span: Span) -> Self {
        Self { kind, lines, span }
    }

    /// Check if block content is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || self.lines.iter().all(|l| l.trim().is_empty())
    }

    /// Get non-empty trimmed lines
    pub fn content_lines(&self) -> Vec<&str> {
        self.lines
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Join content as single string
    pub fn content(&self) -> String {
        self.content_lines().join("\n")
    }
}

/// Complete APEX document (parsed AST)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApexDocument {
    /// All blocks in document order
    pub blocks: Vec<Block>,
    /// APEX version if specified in META
    pub version: Option<String>,
}

impl ApexDocument {
    /// Create empty document
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            version: None,
        }
    }

    /// Create document with blocks
    pub fn with_blocks(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            version: None,
        }
    }

    // --- Block accessors ---

    /// Get first block of given kind
    pub fn get_block(&self, kind: BlockKind) -> Option<&Block> {
        self.blocks.iter().find(|b| b.kind == kind)
    }

    /// Get all blocks of given kind
    pub fn get_blocks(&self, kind: BlockKind) -> Vec<&Block> {
        self.blocks.iter().filter(|b| b.kind == kind).collect()
    }

    /// Count blocks of given kind
    pub fn count_blocks(&self, kind: BlockKind) -> usize {
        self.blocks.iter().filter(|b| b.kind == kind).count()
    }

    // --- Convenience accessors ---

    pub fn task(&self) -> Option<&Block> {
        self.get_block(BlockKind::Task)
    }

    pub fn goals(&self) -> Option<&Block> {
        self.get_block(BlockKind::Goals)
    }

    pub fn plan(&self) -> Option<&Block> {
        self.get_block(BlockKind::Plan)
    }

    pub fn constraints(&self) -> Option<&Block> {
        self.get_block(BlockKind::Constraints)
    }

    pub fn validation(&self) -> Option<&Block> {
        self.get_block(BlockKind::Validation)
    }

    pub fn tools(&self) -> Option<&Block> {
        self.get_block(BlockKind::Tools)
    }

    pub fn diff(&self) -> Option<&Block> {
        self.get_block(BlockKind::Diff)
    }

    pub fn context(&self) -> Option<&Block> {
        self.get_block(BlockKind::Context)
    }

    pub fn meta(&self) -> Option<&Block> {
        self.get_block(BlockKind::Meta)
    }
}

impl Default for ApexDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_kind_from_str() {
        assert_eq!(BlockKind::from_str("TASK"), Some(BlockKind::Task));
        assert_eq!(BlockKind::from_str("task"), Some(BlockKind::Task));
        assert_eq!(BlockKind::from_str("Task"), Some(BlockKind::Task));
        assert_eq!(BlockKind::from_str("UNKNOWN"), None);
    }

    #[test]
    fn test_block_content() {
        let block = Block::new(
            BlockKind::Plan,
            vec![
                "  Step 1: Do something  ".to_string(),
                "".to_string(),
                "  Step 2: Do another  ".to_string(),
            ],
            Span::new(1, 3),
        );

        assert_eq!(block.content_lines(), vec!["Step 1: Do something", "Step 2: Do another"]);
        assert!(!block.is_empty());
    }

    #[test]
    fn test_document_accessors() {
        let doc = ApexDocument::with_blocks(vec![
            Block::new(BlockKind::Task, vec!["Implement feature".to_string()], Span::line(1)),
            Block::new(BlockKind::Plan, vec!["Step 1".to_string()], Span::line(3)),
        ]);

        assert!(doc.task().is_some());
        assert!(doc.plan().is_some());
        assert!(doc.goals().is_none());
        assert_eq!(doc.count_blocks(BlockKind::Task), 1);
    }
}
