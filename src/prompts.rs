//! APEX Prompts Module
//!
//! Embeds the official APEX v1.1 generator and executor prompts at compile time.
//! These prompts can be used to instruct LLMs how to generate and execute
//! valid APEX v1.1 documents.

/// APEX v1.1 Generator Prompt
///
/// This prompt instructs LLMs how to convert natural language requests
/// into valid APEX v1.1 documents.
pub const APEX_GENERATOR_V1_1: &str = include_str!("../prompts/generator_v1_1.txt");

/// APEX v1.1 Executor Prompt
///
/// This prompt instructs LLMs how to execute APEX v1.1 documents,
/// following constraints, respecting precedence, and validating outputs.
pub const APEX_EXECUTOR_V1_1: &str = include_str!("../prompts/executor_v1_1.txt");

/// APEX v1.1 Specification Addendum
///
/// The complete v1.1 hardening addendum specification.
pub const APEX_SPEC_V1_1: &str = include_str!("../spec/apex_v1_1_addendum.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_loaded() {
        assert!(!APEX_GENERATOR_V1_1.is_empty());
        assert!(!APEX_EXECUTOR_V1_1.is_empty());
        assert!(!APEX_SPEC_V1_1.is_empty());
    }

    #[test]
    fn test_spec_contains_version() {
        assert!(APEX_SPEC_V1_1.contains("v1.1"));
        assert!(APEX_SPEC_V1_1.contains("Hardening Addendum"));
    }
}
