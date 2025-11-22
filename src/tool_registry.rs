//! APEX Tool Registry
//!
//! Provides validation of tool names against a known registry.
//! Per APEX v1.1, tools must be validated against a runtime registry.

use std::collections::HashSet;

/// Default valid tools in the APEX ecosystem
pub static VALID_TOOLS: &[&str] = &[
    // Code intelligence
    "code_search",
    "code_edit",
    "code_read",
    "code_write",
    // Vector/embedding operations
    "vector_search",
    "vector_store",
    "vector_delete",
    // Graph operations
    "graph_query",
    "graph_store",
    "graph_delete",
    // Memory operations (LTMC)
    "memory.query",
    "memory.store",
    "memory.delete",
    "memory.consolidate",
    // Unix/system operations
    "unix_action",
    "bash",
    "shell",
    // File operations
    "read_file",
    "write_file",
    "edit_file",
    "glob",
    "grep",
    // Web operations
    "web_fetch",
    "web_search",
    // Generic tool patterns
    "mcp_tool",
];

/// Tool registry for validating tool names
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    tools: HashSet<String>,
    allow_unknown: bool,
}

impl ToolRegistry {
    /// Create a new registry with default tools
    pub fn new() -> Self {
        let tools = VALID_TOOLS.iter().map(|s| s.to_string()).collect();
        Self {
            tools,
            allow_unknown: false,
        }
    }

    /// Create an empty registry
    pub fn empty() -> Self {
        Self {
            tools: HashSet::new(),
            allow_unknown: false,
        }
    }

    /// Create a permissive registry that allows any tool
    pub fn permissive() -> Self {
        Self {
            tools: HashSet::new(),
            allow_unknown: true,
        }
    }

    /// Add a tool to the registry
    pub fn add_tool(&mut self, name: &str) {
        self.tools.insert(name.to_string());
    }

    /// Add multiple tools to the registry
    pub fn add_tools(&mut self, names: &[&str]) {
        for name in names {
            self.tools.insert(name.to_string());
        }
    }

    /// Check if a tool is valid
    pub fn is_valid(&self, name: &str) -> bool {
        if self.allow_unknown {
            return true;
        }
        // Exact match
        if self.tools.contains(name) {
            return true;
        }
        // Check for prefix patterns (e.g., "mcp__server__tool")
        if name.starts_with("mcp__") {
            return true;
        }
        false
    }

    /// Validate a tool name, returning an error message if invalid
    pub fn validate(&self, name: &str) -> Result<(), String> {
        if self.is_valid(name) {
            Ok(())
        } else {
            Err(format!("Unknown tool '{}' not in registry", name))
        }
    }

    /// Get all registered tools
    pub fn tools(&self) -> &HashSet<String> {
        &self.tools
    }

    /// Set whether unknown tools are allowed
    pub fn set_allow_unknown(&mut self, allow: bool) {
        self.allow_unknown = allow;
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract tool name from a TOOLS block line
///
/// Handles formats like:
/// - `tool_name`
/// - `tool_name(args)`
/// - `tool_name "query"`
pub fn extract_tool_name(line: &str) -> &str {
    let trimmed = line.trim();

    // Check for parentheses
    if let Some(paren_idx) = trimmed.find('(') {
        return trimmed[..paren_idx].trim();
    }

    // Check for space (arguments)
    if let Some(space_idx) = trimmed.find(' ') {
        return trimmed[..space_idx].trim();
    }

    // Check for quotes
    if let Some(quote_idx) = trimmed.find('"') {
        return trimmed[..quote_idx].trim();
    }

    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.is_valid("code_search"));
        assert!(registry.is_valid("vector_search"));
        assert!(registry.is_valid("memory.query"));
        assert!(!registry.is_valid("nonexistent_tool"));
    }

    #[test]
    fn test_mcp_tools_allowed() {
        let registry = ToolRegistry::new();
        assert!(registry.is_valid("mcp__server__tool"));
        assert!(registry.is_valid("mcp__jenkins__build_job"));
    }

    #[test]
    fn test_permissive_registry() {
        let registry = ToolRegistry::permissive();
        assert!(registry.is_valid("any_tool"));
        assert!(registry.is_valid("completely_unknown"));
    }

    #[test]
    fn test_custom_registry() {
        let mut registry = ToolRegistry::empty();
        registry.add_tool("my_custom_tool");
        assert!(registry.is_valid("my_custom_tool"));
        assert!(!registry.is_valid("code_search")); // Default not included
    }

    #[test]
    fn test_extract_tool_name() {
        assert_eq!(extract_tool_name("code_search"), "code_search");
        assert_eq!(extract_tool_name("code_search(query)"), "code_search");
        assert_eq!(extract_tool_name("code_search \"pattern\""), "code_search");
        assert_eq!(extract_tool_name("  vector_search  "), "vector_search");
    }
}
