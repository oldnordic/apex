//! # APEX Spec (v1.1)
//!
//! APEX (Agent Planning & Execution Specification) is a deterministic,
//! human-readable planning DSL for structuring LLM reasoning,
//! tool invocation sequences, and validation gates.
//!
//! This crate provides the reference implementation of the APEX v1.1
//! grammar, parser, validator, and interpreter.
//!
//! ## Core APIs
//!
//! ### Parse entire document
//!
//! ```rust
//! use apex_spec::parse_full;
//!
//! let input = r#"
//! TASK
//! Fix search parameter
//!
//! GOALS
//! Improve recall
//!
//! PLAN
//! Scan code
//! Fix param
//! Run tests
//!
//! CONSTRAINTS
//! no_mocks
//!
//! VALIDATION
//! cargo test
//!
//! TOOLS
//! code_search "query"
//!
//! META
//! version=1.1
//! "#;
//!
//! let plan = parse_full(input).unwrap();
//! println!("Task: {}", plan.task);
//! ```
//!
//! ### Parse & validate only
//!
//! ```rust
//! use apex_spec::parse_and_validate;
//!
//! let validated = parse_and_validate("TASK\nDo something").unwrap();
//! assert_eq!(validated.task.line, "Do something");
//! ```
//!
//! ### Access execution plan
//!
//! ```rust
//! use apex_spec::{ExecutionPlan, ExecutionStep, parse_full};
//!
//! let plan = parse_full("TASK\nBuild feature\nPLAN\nStep 1\nStep 2").unwrap();
//! for step in &plan.steps {
//!     println!("Step: {}", step.description);
//! }
//! ```
//!
//! ## Prompts for LLMs
//!
//! Embedded prompts for generating and executing APEX:
//!
//! - [`prompts::APEX_GENERATOR_V1_1`] - Converts natural language to APEX documents
//! - [`prompts::APEX_EXECUTOR_V1_1`] - Executes APEX documents step-by-step
//! - [`prompts::APEX_SPEC_V1_1`] - The v1.1 specification addendum
//!
//! These define the full LLM-side behavior required to produce or execute
//! APEX documents safely and deterministically.
//!
//! ```rust
//! use apex_spec::prompts::{APEX_GENERATOR_V1_1, APEX_EXECUTOR_V1_1};
//!
//! // Prepend to user query for APEX generation
//! let generator_prompt = format!("{}\n\n{}", APEX_GENERATOR_V1_1, "Fix the search bug");
//!
//! // Prepend to APEX document for execution
//! let apex_document = "TASK\nDo something";
//! let executor_prompt = format!("{}\n\n{}", APEX_EXECUTOR_V1_1, apex_document);
//! ```
//!
//! ## Parse Modes
//!
//! - [`ParseMode::Strict`] - Exact EBNF compliance (v1.0 behavior)
//! - [`ParseMode::Tolerant`] - Repairs lowercase headers, records fixes
//!
//! ```rust
//! use apex_spec::{parse_str_with_mode, ParseMode};
//!
//! let input = "task\nDo something\nplan\nStep 1";
//! let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();
//! assert!(!result.fixes.is_empty()); // Recorded header case fixes
//! ```
//!
//! ## Constraint Canonicalization
//!
//! Constraints are normalized to lowercase with underscores:
//! - "No Mocks" → `no_mocks`
//! - "REAL DBS" → `real_dbs`
//! - "< 300 LOC" → `300_loc`
//!
//! ```rust
//! use apex_spec::canonicalize;
//!
//! assert_eq!(canonicalize("No Mocks"), "no_mocks");
//! assert_eq!(canonicalize("REAL DBS"), "real_dbs");
//! ```
//!
//! ## Tool Registry
//!
//! APEX validates all tools against a fixed registry to prevent
//! hallucinated tool usage:
//!
//! ```rust
//! use apex_spec::{ToolRegistry, validate_with_mode, ValidationMode, parse_str};
//!
//! let registry = ToolRegistry::new();
//! assert!(registry.is_valid("code_search"));
//! assert!(!registry.is_valid("fake_tool"));
//!
//! // MCP tools (mcp__*) are always valid
//! assert!(registry.is_valid("mcp__jenkins__build_job"));
//! ```
//!
//! ## Execution State
//!
//! Models for recording step progress, enabling checkpointing:
//!
//! ```rust
//! use apex_spec::{ExecutionState, StepStatus};
//!
//! let mut state = ExecutionState::new(3); // 3-step plan
//! state.step_states[0] = StepStatus::Complete;
//! state.checkpoint = 1;
//! ```
//!
//! ## Block Types
//!
//! | Block | Required | Description |
//! |-------|----------|-------------|
//! | TASK | Yes | Single-line task description |
//! | GOALS | No | Success criteria |
//! | PLAN | No | Ordered execution steps |
//! | CONSTRAINTS | No | Execution constraints |
//! | VALIDATION | No | Post-execution checks |
//! | TOOLS | No | Tool declarations |
//! | DIFF | No | Expected file changes |
//! | CONTEXT | No | Pre-loaded context |
//! | META | No | Metadata key-value pairs |
//!
//! ## Precedence
//!
//! ```text
//! CONSTRAINTS > TASK > GOALS > PLAN > CONTEXT
//! ```
//!
//! Constraints always win in conflict resolution.
//!
//! ## Validation Modes
//!
//! - [`ValidationMode::Strict`] - Requires version, rejects unknown tools
//! - [`ValidationMode::Lenient`] - Warns but allows unknown tools
//! - [`ValidationMode::Legacy`] - v1.0 behavior, no version required
//!
//! This crate is dependency-free and designed for integration
//! with any agent runtime.

pub mod ast;
pub mod errors;
pub mod interpreter;
pub mod parser;
pub mod prompts;
pub mod sem;
pub mod tool_registry;
pub mod validate;

// Re-exports for convenience
pub use ast::{ApexDocument, Block, BlockKind, Span};
pub use errors::{ApexError, ApexErrorKind, ApexResult};
pub use interpreter::{
    ExecutionPlan, ExecutionStep, ExecutionState, StepStatus,
    ToolInvocation, build_execution_plan
};
pub use parser::{parse_str, parse_str_with_mode, ParseMode, ParseFix};
pub use prompts::{APEX_GENERATOR_V1_1, APEX_EXECUTOR_V1_1, APEX_SPEC_V1_1};
pub use sem::{Constraint, Precedence, Semantics, normalize_constraint, canonicalize};
pub use tool_registry::{ToolRegistry, VALID_TOOLS, extract_tool_name};
pub use validate::{ValidatedDocument, validate, validate_with_mode, DiffFormat, ValidationMode};

/// Parse and validate APEX input in one call
pub fn parse_and_validate(input: &str) -> ApexResult<ValidatedDocument> {
    let doc = parse_str(input)?;
    validate(doc)
}

/// Full pipeline: parse → validate → interpret
pub fn parse_full(input: &str) -> ApexResult<ExecutionPlan> {
    let validated = parse_and_validate(input)?;
    build_execution_plan(&validated)
}

/// APEX format version supported by this crate
pub const APEX_VERSION: &str = "1.1";

/// Minimum supported APEX version
pub const APEX_MIN_VERSION: &str = "1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_validate() {
        let input = "TASK\nDo something important";
        let validated = parse_and_validate(input).unwrap();
        assert_eq!(validated.task.line, "Do something important");
    }

    #[test]
    fn test_parse_full() {
        let input = r#"TASK
Build feature

PLAN
Step 1
Step 2

CONSTRAINTS
Be safe
"#;
        let plan = parse_full(input).unwrap();
        assert_eq!(plan.task, "Build feature");
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.constraints.len(), 1);
    }

    #[test]
    fn test_round_trip() {
        let input = r#"TASK
Implement caching layer

GOALS
Reduce latency
Improve throughput

PLAN
Analyze current performance
Identify hot paths
Implement cache
Benchmark results

CONSTRAINTS
No breaking API changes
Must pass existing tests

VALIDATION
Latency reduced by 50%

TOOLS
read_file(path)
write_file(path, content)
benchmark_run()

META
version=1.0
author=test
"#;
        let plan = parse_full(input).unwrap();

        assert_eq!(plan.task, "Implement caching layer");
        assert_eq!(plan.goals.len(), 2);
        assert_eq!(plan.steps.len(), 4);
        assert_eq!(plan.constraints.len(), 2);
        assert_eq!(plan.validation.len(), 1);
        assert_eq!(plan.available_tools.len(), 3);
    }

    #[test]
    fn test_semantics_extraction() {
        let input = r#"TASK
Refactor module

CONSTRAINTS
No mocks allowed
Real databases only
< 300 LOC per file
Safe refactoring
API compatibility required
"#;
        let validated = parse_and_validate(input).unwrap();
        let sem = Semantics::from_validated(&validated);

        assert!(sem.forbids_mocks());
        assert!(sem.requires_real_dbs());
        assert_eq!(sem.loc_limit(), Some(300));
        assert!(sem.requires_safe_refactor());
        assert!(sem.requires_api_compat());
    }

    #[test]
    fn test_error_handling() {
        // Missing TASK
        let result = parse_and_validate("PLAN\nStep 1");
        assert!(result.is_err());

        // Multiple TASKs
        let result = parse_and_validate("TASK\nFirst\nTASK\nSecond");
        assert!(result.is_err());

        // Empty TASK
        let result = parse_and_validate("TASK\n\nPLAN\nStep 1");
        assert!(result.is_err());
    }
}
