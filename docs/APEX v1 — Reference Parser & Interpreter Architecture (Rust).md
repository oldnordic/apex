APEX v1 — Reference Parser & Interpreter Architecture (Rust)

APEX v1 — Reference Parser & Interpreter Architecture (Rust)
============================================================

Version: 1.0-draft
Scope: Rust crate design for parsing, validating, and interpreting APEX
Crate name (recommended): apex_spec
Target: SynCore, OdinCode, mini-model tooling

------------------------------------------------------------
1. OVERVIEW
------------------------------------------------------------
The `apex_spec` crate provides:

1) APEX AST (Abstract Syntax Tree)
2) APEX parser (string → AST)
3) APEX validator (AST → ValidatedDocument or Error)
4) APEX interpreter scaffold (AST → ExecutionPlan)
5) Error types (syntax, semantic, runtime)
6) Round-trip helpers (APEX ↔ JSON/NLP scaffolds)
7) Test suite (unit + integration)

This crate is **pure Rust**, with no external runtime dependencies,
so it can be embedded in SynCore, OdinCode, CLI tools, and tests.

------------------------------------------------------------
2. CRATE LAYOUT
------------------------------------------------------------

Suggested file structure:

apex_spec/
  ├── src/
  │   ├── lib.rs
  │   ├── ast.rs
  │   ├── parser/
  │   │   ├── mod.rs
  │   │   ├── lexer.rs
  │   │   └── parser.rs
  │   ├── sem.rs
  │   ├── validate.rs
  │   ├── interpreter.rs
  │   ├── errors.rs
  │   └── json_interop.rs
  ├── tests/
  │   ├── parser_smoke_tests.rs
  │   ├── validation_tests.rs
  │   ├── interpreter_plan_tests.rs
  │   └── roundtrip_tests.rs
  └── Cargo.toml

High-level responsibilities:

- `ast.rs`         : Core data structures for APEX blocks and document.
- `parser/*`       : Lexer + parser implementing EBNF grammar → AST.
- `sem.rs`         : Semantic model & precedence rules.
- `validate.rs`    : Structural and semantic validation.
- `interpreter.rs` : Conversion of AST into an executable plan.
- `errors.rs`      : Unified error model.
- `json_interop.rs`: Helpers for APEX ↔ JSON (tool plans, etc.)

------------------------------------------------------------
3. AST DESIGN (ast.rs)
------------------------------------------------------------

Core types:

- `ApexDocument`
- `Block`
- `BlockKind`
- `TaskBlock`
- `GoalsBlock`
- `PlanBlock`
- `ConstraintsBlock`
- `ValidationBlock`
- `ToolsBlock`
- `DiffBlock`
- `ContextBlock`
- `MetaBlock`

Example skeleton:

pub struct ApexDocument {
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub kind: BlockKind,
    pub lines: Vec<String>,
    pub span: Span, // line/column meta for errors
}

pub enum BlockKind {
    Task,
    Goals,
    Plan,
    Constraints,
    Validation,
    Tools,
    Diff,
    Context,
    Meta,
}

pub struct Span {
    pub start_line: usize,
    pub end_line: usize,
}

Convenience accessors (recommended):

impl ApexDocument {
    pub fn task(&self) -> Option<&Block> { /* ... */ }
    pub fn goals(&self) -> Option<&Block> { /* ... */ }
    pub fn plan(&self) -> Option<&Block> { /* ... */ }
    pub fn constraints(&self) -> Option<&Block> { /* ... */ }
    pub fn validation(&self) -> Option<&Block> { /* ... */ }
    pub fn tools(&self) -> Option<&Block> { /* ... */ }
    pub fn diff(&self) -> Option<&Block> { /* ... */ }
    pub fn context(&self) -> Option<&Block> { /* ... */ }
    pub fn meta(&self) -> Option<&Block> { /* ... */ }
}

------------------------------------------------------------
4. PARSER MODULE (parser/mod.rs, lexer.rs, parser.rs)
------------------------------------------------------------

4.1 Goals
- Implement the EBNF grammar from spec B.
- Minimal tokenization (block headers + lines).
- Deterministic, whitespace-tolerant.
- Return `ApexDocument` or `ApexError`.

4.2 Public API

pub mod parser {
    use crate::ast::ApexDocument;
    use crate::errors::ApexError;

    pub fn parse_str(input: &str) -> Result<ApexDocument, ApexError> {
        // entry point: string → AST
    }
}

4.3 Lexer Design (lexer.rs)

- Tokenize into:
  - `Token::BlockHeader(BlockKind, Span)`
  - `Token::Line(String, Span)`
  - `Token::Newline(Span)`
  - `Token::EOF`

- Recognize block headers:
  - "TASK"
  - "GOALS"
  - "PLAN"
  - "CONSTRAINTS"
  - "VALIDATION"
  - "TOOLS"
  - "DIFF"
  - "CONTEXT"
  - "META"

- Any non-header lines → `Token::Line`.

4.4 Parser Design (parser.rs)

- Single-pass scan:
  - Expect a `BlockHeader`
  - Read all subsequent `Line` tokens until next `BlockHeader` or `EOF`
  - Construct `Block` with `kind` and `lines`

- Result:
  - `ApexDocument { blocks }`

- Parser **does not** enforce semantics (no “1 TASK only” check here).
  That is the job of `validate.rs`.

------------------------------------------------------------
5. VALIDATION MODULE (validate.rs)
------------------------------------------------------------

5.1 Purpose
- Apply structural and semantic rules from spec C.
- Produce `ValidatedDocument` or `ApexError`.

5.2 Types

pub struct ValidatedDocument {
    pub doc: ApexDocument,
    pub task: TaskView,
    pub goals: Option<GoalsView>,
    pub plan: Option<PlanView>,
    pub constraints: Option<ConstraintsView>,
    pub validation: Option<ValidationView>,
    pub tools: Option<ToolsView>,
    pub diff: Option<DiffView>,
    pub context: Option<ContextView>,
    pub meta: Option<MetaView>,
}

Each `*View` is a parsed, domain-specific representation of raw lines:

pub struct TaskView {
    pub line: String,
}

pub struct PlanView {
    pub steps: Vec<String>,
}

pub struct ConstraintsView {
    pub rules: Vec<String>,
}

// etc.

5.3 Public API

use crate::ast::ApexDocument;
use crate::errors::ApexError;

pub fn validate(doc: ApexDocument) -> Result<ValidatedDocument, ApexError> {
    // 1. Ensure exactly one TASK block
    // 2. Ensure no unknown block kinds (already handled in parser)
    // 3. Ensure non-empty required blocks
    // 4. Parse lines into views
    // 5. Apply precedence and sanity checks
}

5.4 Rules enforced here:
- Exactly one TASK block present.
- Empty blocks (except CONTEXT/META) → error.
- PLAN present if execution mode requires it (configurable).
- TOOL names syntactically valid (basic sanity).
- Constraints parsed into a normalized set of strings.

------------------------------------------------------------
6. SEMANTICS MODULE (sem.rs)
------------------------------------------------------------

Purpose:
- Provide higher-level semantics and precedence helpers.

Typical content:
- Enums for constraint priorities
- Helpers for semantic queries
- Utility functions to check conflicts

Example:

pub enum Constraint {
    RealDbsOnly,
    NoMocks,
    Lt300Loc,
    SafeRefactor,
    ApiCompat,
    Other(String),
}

pub struct Semantics {
    pub constraints: Vec<Constraint>,
}

impl Semantics {
    pub fn from_validated(doc: &ValidatedDocument) -> Self {
        // parse constraints lines into structured Constraint enums
    }

    pub fn forbids_mocks(&self) -> bool { /* ... */ }
    pub fn limits_loc(&self) -> bool { /* ... */ }
}

------------------------------------------------------------
7. INTERPRETER MODULE (interpreter.rs)
------------------------------------------------------------

7.1 Purpose
- Convert a `ValidatedDocument` into an `ExecutionPlan`.
- Provide minimal but stable contract for SynCore.

7.2 Types

pub struct ExecutionStep {
    pub description: String,
    pub maybe_tool: Option<ToolInvocation>,
}

pub struct ToolInvocation {
    pub name: String,
    pub raw_arguments: String, // can be parsed downstream
}

pub struct ExecutionPlan {
    pub task: String,
    pub goals: Vec<String>,
    pub constraints: Vec<String>,
    pub steps: Vec<ExecutionStep>,
    pub validation: Vec<String>,
}

7.3 Public API

use crate::validate::ValidatedDocument;
use crate::errors::ApexError;

pub fn build_execution_plan(doc: &ValidatedDocument) -> Result<ExecutionPlan, ApexError> {
    // 1. Use TASK as core task description.
    // 2. Extract GOALS as strings.
    // 3. Extract PLAN as ordered ExecutionSteps.
    // 4. Match TOOL names from TOOLS block to steps (best-effort).
    // 5. Attach constraints and validation conditions.
}

Notes:
- Mapping PLAN ↔ TOOLS can be simple:
  - 1:1 by line index
  - or heuristic: if tool line contains keywords from step.

- SynCore can later extend this for smarter mapping.

------------------------------------------------------------
8. ERRORS MODULE (errors.rs)
------------------------------------------------------------

8.1 Purpose
- Unified error handling across parse, validate, interpret.

8.2 Types

use std::fmt;

pub enum ApexErrorKind {
    ParseError,
    LexError,
    MissingTask,
    MultipleTasks,
    EmptyRequiredBlock,
    UnknownBlock,
    InvalidToolName,
    ConstraintViolation,
    ValidationFailure,
    InternalError,
}

pub struct ApexError {
    pub kind: ApexErrorKind,
    pub message: String,
    pub line: Option<usize>,
}

impl fmt::Display for ApexError { /* ... */ }
impl std::error::Error for ApexError {}

Helper constructors:

impl ApexError {
    pub fn parse(msg: impl Into<String>, line: Option<usize>) -> Self { /* ... */ }
    pub fn missing_task() -> Self { /* ... */ }
    pub fn multiple_tasks() -> Self { /* ... */ }
    pub fn empty_block(name: &str, line: Option<usize>) -> Self { /* ... */ }
}

------------------------------------------------------------
9. JSON INTEROP MODULE (json_interop.rs)
------------------------------------------------------------

Purpose:
- Provide optional utilities to bridge between APEX and JSON
  for tool execution in SynCore.

Types (suggestion):

pub struct ToolCallJson {
    pub name: String,
    pub arguments: serde_json::Value,
}

pub struct ExecutionPlanJson {
    pub task: String,
    pub goals: Vec<String>,
    pub constraints: Vec<String>,
    pub steps: Vec<serde_json::Value>, // or a structured type
    pub validation: Vec<String>,
}

Public API:

pub fn plan_to_json(plan: &ExecutionPlan) -> serde_json::Value { /* ... */ }
pub fn json_to_apex(json: &serde_json::Value) -> Result<ApexDocument, ApexError> { /* ... */ }

(These can be incremental and only implemented where actually used.)

------------------------------------------------------------
10. TESTING STRATEGY (tests/*.rs)
------------------------------------------------------------

10.1 Parser Smoke Tests (parser_smoke_tests.rs)
- Test minimal documents:
  - Only TASK
  - TASK + PLAN
  - Full example from spec A
- Test random whitespace
- Test unknown header → error

10.2 Validation Tests (validation_tests.rs)
- Missing TASK → ApexError::MissingTask
- Two TASK blocks → ApexError::MultipleTasks
- Empty PLAN → ApexError::EmptyRequiredBlock
- Empty CONSTRAINTS → ApexError::EmptyRequiredBlock
- CONTEXT / META allowed empty.

10.3 Interpreter Tests (interpreter_plan_tests.rs)
- Given a full APEX doc, ensure ExecutionPlan has:
  - correct task
  - Plan steps in correct order
  - constraints present
  - validation lines passed through

10.4 Roundtrip Tests (roundtrip_tests.rs)
- apex_str → parse → validate → build_plan
- Optionally, plan → JSON → (future) → APEX

------------------------------------------------------------
11. PUBLIC LIB API (lib.rs)
------------------------------------------------------------

Suggested public surface:

mod ast;
mod parser;
mod validate;
mod sem;
mod interpreter;
mod errors;
mod json_interop;

pub use ast::{ApexDocument, Block, BlockKind};
pub use errors::{ApexError, ApexErrorKind};
pub use interpreter::{ExecutionPlan, ExecutionStep, ToolInvocation};
pub use parser::parse_str;
pub use validate::{ValidatedDocument, validate};
pub use sem::Semantics;

High-level helper:

pub fn parse_and_validate(input: &str) -> Result<ValidatedDocument, ApexError> {
    let doc = parse_str(input)?;
    validate(doc)
}

pub fn parse_full(input: &str) -> Result<ExecutionPlan, ApexError> {
    let validated = parse_and_validate(input)?;
    interpreter::build_execution_plan(&validated)
}

------------------------------------------------------------
12. NON-GOALS
------------------------------------------------------------
- No automatic tool invocation in this crate.
- No I/O or network access.
- No SynCore-specific DB logic.
- No LLM calls.

This crate is a **pure spec and structure layer**:
- string → AST → validated doc → execution plan,
ready for higher-level runtimes.

------------------------------------------------------------
END OF APEX PARSER & INTERPRETER ARCHITECTURE
------------------------------------------------------------
