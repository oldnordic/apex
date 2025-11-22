# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build              # Build the library
cargo test               # Run all tests (33 unit + 2 doctests)
cargo test test_name     # Run single test by name
cargo test --lib         # Run only library tests (skip doctests)
cargo doc --open         # Generate and view documentation
```

## Project Overview

APEX (Agent Planning & Execution Specification) is a DSL for constraining LLM behavior in agentic workflows. It defines a human-readable, machine-parseable format for expressing agent tasks, constraints, and execution plans.

**Current Version: 1.1** (backwards-compatible with v1.0)

The core pipeline is: **String → AST → ValidatedDocument → ExecutionPlan**

## Architecture

### Processing Pipeline

1. **Lexer** (`src/parser/lexer.rs`) - Tokenizes input into `BlockHeader` and `Line` tokens
2. **Parser** (`src/parser/parser.rs`) - Builds `ApexDocument` AST from tokens
3. **Validator** (`src/validate.rs`) - Validates structure, produces typed views (TaskView, PlanView, etc.)
4. **Semantics** (`src/sem.rs`) - Parses constraints into typed enum, applies precedence rules
5. **Interpreter** (`src/interpreter.rs`) - Builds `ExecutionPlan` with steps and tool matching

### Key Types

- `ApexDocument` - Raw parsed AST with `Vec<Block>`
- `ValidatedDocument` - Validated document with typed views (TaskView, GoalsView, etc.)
- `ExecutionPlan` - Final executable plan with steps, constraints, tools
- `Constraint` - Typed constraint enum (NoMocks, RealDbsOnly, LtLoc(u32), etc.)
- `Precedence` - Conflict resolution ordering: CONSTRAINTS > TASK > GOALS > PLAN > CONTEXT

### Entry Points

```rust
// Full pipeline (most common)
let plan = apex_spec::parse_full(input)?;

// Parse + validate only
let validated = apex_spec::parse_and_validate(input)?;

// Check semantic constraints
let sem = Semantics::from_validated(&validated);
if sem.forbids_mocks() { /* ... */ }
```

## APEX Format

Required block: `TASK` (exactly one, non-empty)

Optional blocks: `GOALS`, `PLAN`, `CONSTRAINTS`, `VALIDATION`, `TOOLS`, `DIFF`, `CONTEXT`, `META`

Block headers must be uppercase identifiers alone on a line. Content follows until next header or EOF.

```
TASK
Implement user authentication

CONSTRAINTS
No mocks
Real databases only
< 300 LOC

PLAN
Design auth schema
Implement login endpoint
Write tests
```

## Constraint Parsing (v1.1 Normalization)

All constraints are normalized to canonical form via `normalize_constraint()`:
1. Trim whitespace
2. Convert to lowercase
3. Replace non-alphanumeric sequences with `_`

Example: `"No Mocks Allowed!"` → `"no_mocks_allowed"`

The `Constraint::from_str` function maps to known types:
- `"no_mocks"`, `"no mocks"`, `"NO MOCKS"` → `Constraint::NoMocks`
- `"real_dbs"`, `"real databases only"` → `Constraint::RealDbsOnly`
- `"lt300loc"`, `"< 300 LOC"` → `Constraint::LtLoc(300)`
- `"safe_refactor"` → `Constraint::SafeRefactor`
- `"api_compat"` → `Constraint::ApiCompat`

## v1.1 Parser Modes

```rust
use apex_spec::{parse_str_with_mode, ParseMode};

// Strict mode (default) - v1.0 behavior
let doc = parse_str(input)?;

// Tolerant mode - recovers from lowercase headers
let result = parse_str_with_mode(input, ParseMode::Tolerant)?;
for fix in &result.fixes {
    println!("Line {}: {}", fix.line, fix.description);
}
```

## v1.1 DIFF Format Markers

DIFF blocks can optionally start with a format marker:
- `unified` - Content is unified diff (machine-validatable)
- `raw` - Content is raw code/description

```
DIFF
unified
--- a/src/lib.rs
+++ b/src/lib.rs
...
```
