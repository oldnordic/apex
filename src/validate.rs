//! APEX Validator
//!
//! Validates parsed AST against structural and semantic rules.
//!
//! ## v1.1 Validation Features
//!
//! - Version enforcement (version=1.1 required in strict mode)
//! - Constraint canonicalization
//! - Tool registry validation
//! - DIFF format marker detection

use crate::ast::{ApexDocument, Block, BlockKind};
use crate::errors::{ApexError, ApexResult};
use crate::sem::canonicalize;
use crate::tool_registry::{ToolRegistry, extract_tool_name};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation mode for v1.1 documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValidationMode {
    /// Strict: requires version=1.1, validates tools against registry
    #[default]
    Strict,
    /// Lenient: allows missing version, unknown tools produce warnings
    Lenient,
    /// Legacy: v1.0 behavior, no version checking
    Legacy,
}

// --- Validated View Types ---

/// Validated TASK view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskView {
    /// Single task description line
    pub line: String,
}

/// Validated GOALS view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalsView {
    /// Individual goal items
    pub goals: Vec<String>,
}

/// Validated PLAN view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanView {
    /// Ordered execution steps
    pub steps: Vec<String>,
}

/// Validated CONSTRAINTS view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintsView {
    /// Constraint rules
    pub rules: Vec<String>,
}

/// Validated VALIDATION view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationView {
    /// Validation conditions
    pub conditions: Vec<String>,
}

/// Validated TOOLS view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsView {
    /// Tool declarations
    pub tools: Vec<ToolDeclaration>,
}

/// Tool declaration parsed from TOOLS block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDeclaration {
    /// Tool name
    pub name: String,
    /// Raw argument string (unparsed)
    pub arguments: Option<String>,
    /// Original line
    pub raw: String,
}

/// DIFF format marker per APEX v1.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiffFormat {
    /// Unified diff format (can be machine-validated)
    Unified,
    /// Raw code or description
    Raw,
    /// No format marker (v1.0 behavior)
    #[default]
    Unspecified,
}

/// Validated DIFF view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffView {
    /// Format marker if present
    pub format: DiffFormat,
    /// Expected file changes (excluding format marker line)
    pub changes: Vec<String>,
}

/// Validated CONTEXT view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextView {
    /// Context lines
    pub lines: Vec<String>,
}

/// Validated META view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaView {
    /// Key-value metadata pairs
    pub entries: HashMap<String, String>,
}

impl MetaView {
    /// Get APEX version from META if present
    pub fn version(&self) -> Option<&str> {
        self.entries.get("version").map(|s| s.as_str())
    }

    /// Check if version is compatible with this implementation
    pub fn is_version_compatible(&self) -> bool {
        match self.version() {
            None => true, // No version = v1.0 assumed, compatible
            Some(v) => {
                // Parse major.minor
                let parts: Vec<&str> = v.split('.').collect();
                if parts.is_empty() {
                    return false;
                }
                match parts[0].parse::<u32>() {
                    Ok(1) => true, // v1.x is compatible
                    Ok(major) if major >= 2 => false, // v2+ not supported
                    _ => false,
                }
            }
        }
    }

    /// Get parse_fixes if recorded (from tolerant mode)
    pub fn parse_fixes(&self) -> Option<&str> {
        self.entries.get("parse_fixes").map(|s| s.as_str())
    }
}

/// Fully validated APEX document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedDocument {
    /// Original AST
    pub doc: ApexDocument,
    /// Required TASK (always present after validation)
    pub task: TaskView,
    /// Optional validated blocks
    pub goals: Option<GoalsView>,
    pub plan: Option<PlanView>,
    pub constraints: Option<ConstraintsView>,
    pub validation: Option<ValidationView>,
    pub tools: Option<ToolsView>,
    pub diff: Option<DiffView>,
    pub context: Option<ContextView>,
    pub meta: Option<MetaView>,
    /// Parse/validation fixes applied (v1.1 tolerant mode)
    #[serde(default)]
    pub meta_fixes: Vec<String>,
    /// Validation warnings (non-fatal issues)
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Validate parsed document (legacy mode - no version enforcement)
pub fn validate(doc: ApexDocument) -> ApexResult<ValidatedDocument> {
    validate_with_mode(doc, ValidationMode::Legacy, None)
}

/// Validate parsed document with mode and optional tool registry
pub fn validate_with_mode(
    doc: ApexDocument,
    mode: ValidationMode,
    registry: Option<&ToolRegistry>,
) -> ApexResult<ValidatedDocument> {
    let mut warnings = Vec::new();

    // Rule 1: Exactly one TASK block
    let task_count = doc.count_blocks(BlockKind::Task);
    if task_count == 0 {
        return Err(ApexError::missing_task());
    }
    if task_count > 1 {
        let second_task = doc.get_blocks(BlockKind::Task)[1];
        return Err(ApexError::multiple_tasks(second_task.span.start_line));
    }

    // Rule 2: Required blocks cannot be empty
    let task_block = doc.task().unwrap();
    if task_block.is_empty() {
        return Err(ApexError::empty_block("TASK", Some(task_block.span.start_line)));
    }

    // Rule 3: Non-empty check for blocks that don't allow empty
    for block in &doc.blocks {
        if !block.kind.allows_empty() && block.is_empty() && block.kind != BlockKind::Task {
            warnings.push(format!("Empty {} block", block.kind));
        }
    }

    // Build validated views
    let task = parse_task_view(task_block)?;
    let goals = doc.goals().map(parse_goals_view).transpose()?;
    let plan = doc.plan().map(parse_plan_view).transpose()?;
    let constraints = doc.constraints().map(|b| parse_constraints_view_canonical(b)).transpose()?;
    let validation = doc.validation().map(parse_validation_view).transpose()?;
    let tools = doc.tools().map(|b| parse_tools_view_with_registry(b, mode, registry, &mut warnings)).transpose()?;
    let diff = doc.diff().map(parse_diff_view).transpose()?;
    let context = doc.context().map(parse_context_view).transpose()?;
    let meta = doc.meta().map(parse_meta_view).transpose()?;

    // v1.1 version enforcement
    if mode == ValidationMode::Strict {
        if let Some(ref m) = meta {
            if let Some(version) = m.version() {
                if !m.is_version_compatible() {
                    return Err(ApexError::new(
                        crate::errors::ApexErrorKind::ValidationFailure,
                        format!("Unsupported APEX version: {}", version),
                    ));
                }
            } else {
                warnings.push("Missing version in META (v1.1 requires version=1.1)".to_string());
            }
        } else {
            warnings.push("Missing META block (v1.1 requires version=1.1)".to_string());
        }
    }

    Ok(ValidatedDocument {
        doc,
        task,
        goals,
        plan,
        constraints,
        validation,
        tools,
        diff,
        context,
        meta,
        meta_fixes: Vec::new(),
        warnings,
    })
}

// --- View Parsers ---

fn parse_task_view(block: &Block) -> ApexResult<TaskView> {
    // TASK should be a single line or joined as one
    let content = block.content();
    Ok(TaskView { line: content })
}

fn parse_goals_view(block: &Block) -> ApexResult<GoalsView> {
    let goals = block.content_lines().iter().map(|s| s.to_string()).collect();
    Ok(GoalsView { goals })
}

fn parse_plan_view(block: &Block) -> ApexResult<PlanView> {
    let steps = block.content_lines().iter().map(|s| s.to_string()).collect();
    Ok(PlanView { steps })
}

fn parse_constraints_view(block: &Block) -> ApexResult<ConstraintsView> {
    let rules = block.content_lines().iter().map(|s| s.to_string()).collect();
    Ok(ConstraintsView { rules })
}

/// Parse constraints with v1.1 canonicalization
fn parse_constraints_view_canonical(block: &Block) -> ApexResult<ConstraintsView> {
    let rules = block
        .content_lines()
        .iter()
        .map(|s| canonicalize(s))
        .collect();
    Ok(ConstraintsView { rules })
}

fn parse_validation_view(block: &Block) -> ApexResult<ValidationView> {
    let conditions = block.content_lines().iter().map(|s| s.to_string()).collect();
    Ok(ValidationView { conditions })
}

fn parse_tools_view(block: &Block) -> ApexResult<ToolsView> {
    let mut tools = Vec::new();

    for line in block.content_lines() {
        let tool = parse_tool_declaration(line)?;
        tools.push(tool);
    }

    Ok(ToolsView { tools })
}

/// Parse tools with optional registry validation (v1.1)
fn parse_tools_view_with_registry(
    block: &Block,
    mode: ValidationMode,
    registry: Option<&ToolRegistry>,
    warnings: &mut Vec<String>,
) -> ApexResult<ToolsView> {
    let mut tools = Vec::new();

    for line in block.content_lines() {
        let tool_name = extract_tool_name(line);

        // Validate against registry if provided
        if let Some(reg) = registry {
            if !reg.is_valid(tool_name) {
                match mode {
                    ValidationMode::Strict => {
                        return Err(ApexError::new(
                            crate::errors::ApexErrorKind::InvalidToolName,
                            format!("Unknown tool '{}' not in registry", tool_name),
                        ));
                    }
                    ValidationMode::Lenient => {
                        warnings.push(format!("Unknown tool '{}' (tool_degraded)", tool_name));
                    }
                    ValidationMode::Legacy => {
                        // No validation in legacy mode
                    }
                }
            }
        }

        let tool = parse_tool_declaration(line)?;
        tools.push(tool);
    }

    Ok(ToolsView { tools })
}

fn parse_tool_declaration(line: &str) -> ApexResult<ToolDeclaration> {
    // Format: tool_name or tool_name(args)
    let trimmed = line.trim();

    if let Some(paren_idx) = trimmed.find('(') {
        // Has arguments
        let name = trimmed[..paren_idx].trim().to_string();
        let rest = &trimmed[paren_idx..];

        // Find matching closing paren
        let args = if rest.ends_with(')') {
            Some(rest[1..rest.len() - 1].to_string())
        } else {
            Some(rest[1..].to_string())
        };

        Ok(ToolDeclaration {
            name,
            arguments: args,
            raw: line.to_string(),
        })
    } else {
        // No arguments
        Ok(ToolDeclaration {
            name: trimmed.to_string(),
            arguments: None,
            raw: line.to_string(),
        })
    }
}

fn parse_diff_view(block: &Block) -> ApexResult<DiffView> {
    let lines: Vec<&str> = block.content_lines();

    if lines.is_empty() {
        return Ok(DiffView {
            format: DiffFormat::Unspecified,
            changes: Vec::new(),
        });
    }

    // Check first line for format marker (v1.1)
    let first_line = lines[0].to_lowercase();
    let (format, skip_first) = match first_line.as_str() {
        "unified" => (DiffFormat::Unified, true),
        "raw" => (DiffFormat::Raw, true),
        _ => (DiffFormat::Unspecified, false),
    };

    let changes = if skip_first {
        lines[1..].iter().map(|s| s.to_string()).collect()
    } else {
        lines.iter().map(|s| s.to_string()).collect()
    };

    Ok(DiffView { format, changes })
}

fn parse_context_view(block: &Block) -> ApexResult<ContextView> {
    let lines = block.content_lines().iter().map(|s| s.to_string()).collect();
    Ok(ContextView { lines })
}

fn parse_meta_view(block: &Block) -> ApexResult<MetaView> {
    let mut entries = HashMap::new();

    for line in block.content_lines() {
        // Format: key=value or key: value
        if let Some(eq_idx) = line.find('=') {
            let key = line[..eq_idx].trim().to_string();
            let value = line[eq_idx + 1..].trim().to_string();
            entries.insert(key, value);
        } else if let Some(colon_idx) = line.find(':') {
            let key = line[..colon_idx].trim().to_string();
            let value = line[colon_idx + 1..].trim().to_string();
            entries.insert(key, value);
        }
        // Skip lines that don't match key=value or key: value format
    }

    Ok(MetaView { entries })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_str;

    #[test]
    fn test_valid_minimal() {
        let doc = parse_str("TASK\nDo the thing").unwrap();
        let validated = validate(doc).unwrap();

        assert_eq!(validated.task.line, "Do the thing");
    }

    #[test]
    fn test_missing_task() {
        let doc = parse_str("PLAN\nStep 1").unwrap();
        let result = validate(doc);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MissingTask"));
    }

    #[test]
    fn test_multiple_tasks() {
        let doc = parse_str("TASK\nFirst\nTASK\nSecond").unwrap();
        let result = validate(doc);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MultipleTasks"));
    }

    #[test]
    fn test_empty_task() {
        let doc = parse_str("TASK\n\nPLAN\nStep 1").unwrap();
        let result = validate(doc);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("EmptyRequiredBlock"));
    }

    #[test]
    fn test_tool_parsing() {
        let doc = parse_str("TASK\nDo it\nTOOLS\nread_file(path)\nwrite_file(path, content)\nsimple_tool").unwrap();
        let validated = validate(doc).unwrap();

        let tools = validated.tools.unwrap();
        assert_eq!(tools.tools.len(), 3);
        assert_eq!(tools.tools[0].name, "read_file");
        assert_eq!(tools.tools[0].arguments, Some("path".to_string()));
        assert_eq!(tools.tools[2].name, "simple_tool");
        assert_eq!(tools.tools[2].arguments, None);
    }

    #[test]
    fn test_meta_parsing() {
        let doc = parse_str("TASK\nDo it\nMETA\nversion=1.0\nauthor: Feanor\nformat = apex").unwrap();
        let validated = validate(doc).unwrap();

        let meta = validated.meta.unwrap();
        assert_eq!(meta.entries.get("version"), Some(&"1.0".to_string()));
        assert_eq!(meta.entries.get("author"), Some(&"Feanor".to_string()));
        assert_eq!(meta.entries.get("format"), Some(&"apex".to_string()));
    }
}
