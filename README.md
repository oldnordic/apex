# APEX Spec — Agent Planning & Execution DSL

APEX (Agent Planning & Execution Specification) is a deterministic,
human-readable DSL for controlling LLM reasoning, tool orchestration,
validation steps, and reproducible execution workflows.

This crate provides the **reference Rust implementation** of APEX v1.1:
- Parser (string → AST)
- Validator (AST → validated document)
- Interpreter (validated document → ExecutionPlan)
- Constraint canonicalization (v1.1)
- Parse modes (Strict / Tolerant)
- DIFF format awareness (unified/raw)
- Tool registry validation
- Execution state types (StepStatus, ExecutionState)
- Embedded APEX generator & executor prompts for LLMs

APEX is designed for reliability:
- deterministic structure
- explicit precedence rules
- validation-driven workflows
- zero tool hallucinations when combined with a tool registry

## Features

- **APEX v1.1-compliant parser & validator**
- **Strict/Tolerant modes** for LLM-bound parsing
- **Constraint canonicalization** (no_mocks, real_dbs, safe_refactor, etc.)
- **Tool registry enforcement**
- **ExecutionPlan builder**
- **Integrated APEX generator/executor prompts**
- **Zero dependencies outside std**

## Installation

```sh
cargo add apex_spec
```

## Minimal Example

```rust
use apex_spec::{parse_full, ExecutionPlan};

let input = r#"
TASK
fix search param

GOALS
improve recall

PLAN
scan code
fix param
run tests

CONSTRAINTS
no_mocks
real_dbs

VALIDATION
cargo test

TOOLS
code_search "hnsw"

META
version=1.1
"#;

let plan: ExecutionPlan = parse_full(input).unwrap();
println!("{:#?}", plan);
```

## End-to-End Example (Rust + LLM)

```rust
use apex_spec::{
    parse_full,
    prompts::{APEX_GENERATOR_V1_1, APEX_EXECUTOR_V1_1},
};

// Step 1: Ask an LLM to produce APEX
let user_query = "Fix the param mismatch in HNSW search and validate recall.";
let generator_prompt = format!("{}\n\n{}", APEX_GENERATOR_V1_1, user_query);

// send `generator_prompt` to your LLM of choice
let apex_document = call_llm(&generator_prompt)?;

// Step 2: Validate & interpret the APEX document
let plan = parse_full(&apex_document)?;

// Step 3: Execute step-by-step with an LLM or agent loop
let executor_prompt = format!(
    "{}\n\n{}",
    APEX_EXECUTOR_V1_1,
    apex_document
);

// send `executor_prompt` + tool results to your agent loop
// interpret tool calls from the model and feed results back

println!("APEX execution initialized: {:?}", plan.task);
```

This pattern works with:
- Claude Code
- GLM4/GLM4-Air (function or text mode)
- GPT models
- Qwen 2.5
- Any local model with prompt support

## What APEX Does

APEX is a simple, human-readable format for expressing:
- **What** an AI agent should do (TASK)
- **Why** it should do it (GOALS)
- **How** it should do it (PLAN)
- **What it must not violate** (CONSTRAINTS)
- **How to verify success** (VALIDATION)

```
TASK
Implement user authentication

GOALS
Users can log in securely
Session tokens expire properly

PLAN
1. Design auth schema
2. Implement login endpoint
3. Add session management
4. Write tests

CONSTRAINTS
No plaintext passwords
Use bcrypt or argon2
No breaking API changes

VALIDATION
All tests pass
No security warnings from cargo audit
```

## Why APEX Exists

### The Problem

When you give an LLM a task like "implement authentication", it has no guardrails:
- It might use MD5 for passwords
- It might skip tests
- It might break your existing API
- It might ignore your coding standards

Current solutions are either:
- **Too vague**: Natural language prompts that LLMs interpret inconsistently
- **Too rigid**: JSON schemas that are token-expensive and hard to read
- **Too implicit**: System prompts that get ignored under pressure

### The Solution

APEX provides **explicit, parseable constraints** that:
1. Are human-readable (you can audit what the agent will do)
2. Are machine-parseable (tools can validate compliance)
3. Have defined precedence (CONSTRAINTS always win)
4. Are token-efficient (minimal syntax overhead)

## Block Types

| Block | Required | Description |
|-------|----------|-------------|
| TASK | Yes | Single-line task description |
| GOALS | No | Success criteria |
| PLAN | No | Ordered execution steps |
| CONSTRAINTS | No | Hard rules that cannot be violated |
| VALIDATION | No | Post-execution checks |
| TOOLS | No | Available tool declarations |
| DIFF | No | Expected file changes |
| CONTEXT | No | Pre-loaded context |
| META | No | Metadata key-value pairs |

## Precedence

```
CONSTRAINTS > TASK > GOALS > PLAN > CONTEXT
```

If there's a conflict, higher precedence wins. Constraints are absolute.

## v1.1 Features

- **Parser Modes**: Strict (v1.0 behavior) and Tolerant (recovers from lowercase headers, records fixes)
- **Constraint Normalization**: "No Mocks" → "no_mocks" canonical form
- **Version Declaration**: `version=1.1` in META block
- **DIFF Format Markers**: Optional `unified` or `raw` markers for machine validation
- **Parse Fix Recording**: Tolerant mode records all corrections in META
- **Tool Registry**: Validation against known tool names to prevent hallucination
- **Execution State**: Types for checkpointing long-running plans

## Usage

```rust
use apex_spec::{parse_full, Semantics, ParseMode, parse_str_with_mode};

let apex = r#"
TASK
Refactor database module

CONSTRAINTS
No mocks
Real databases only
< 300 LOC per function

PLAN
Analyze current structure
Extract common patterns
Write integration tests

META
version=1.1
"#;

let plan = parse_full(apex)?;

// Check constraints programmatically
let validated = apex_spec::parse_and_validate(apex)?;
let sem = Semantics::from_validated(&validated);

if sem.forbids_mocks() {
    println!("This plan requires real implementations");
}

if let Some(limit) = sem.loc_limit() {
    println!("LOC limit: {}", limit);
}

// Tolerant parsing (v1.1) - recovers from malformed input
let result = parse_str_with_mode(apex, ParseMode::Tolerant)?;
if !result.fixes.is_empty() {
    println!("Recovered {} issues", result.fixes.len());
}
```

## Building

```bash
cargo build --release
cargo test
```

## APEX v1.1 Quick Start Guide

APEX defines a deterministic document structure:

```
TASK           — primary objective
GOALS          — desired outcomes
PLAN           — sequential steps
CONSTRAINTS    — rules that override PLAN
VALIDATION     — success checks
TOOLS          — allowed tool calls
META           — version and metadata
```

### 1. Minimal APEX document

```
TASK
fix index mismatch

GOALS
improve recall

PLAN
scan
patch
validate

CONSTRAINTS
no_mocks

VALIDATION
cargo test

TOOLS
code_search "index"

META
version=1.1
```

### 2. Constraint Precedence

```
CONSTRAINTS > TASK > GOALS > PLAN
```

### 3. Parse Modes

- **Strict** — exact EBNF compliance
- **Tolerant** — repairs lowercase headers, missing breaks, minor issues

### 4. DIFF Handling

First line of DIFF block may specify:
- `unified` — produce unified patch
- `raw` — free-form

### 5. Tool Registry Enforcement

Invalid or hallucinated tools cause validation failure in strict mode.

### 6. LLM Integration

Use embedded prompts:
- `APEX_GENERATOR_V1_1` — to create APEX documents
- `APEX_EXECUTOR_V1_1` — to execute APEX documents

### 7. Execution State Types

Use `ExecutionState` + `StepStatus` to implement checkpointing in your agent.

## Limitations

- **Not a runtime** — APEX defines plans, doesn't execute them
- **No enforcement** — LLMs can still ignore constraints (but you can detect it)
- **v1.x is minimal** — Missing features like includes, dependencies, typed schemas
- **Rust only** — No Python/JS bindings yet

## Roadmap

### v1.1 (Current)
- Parser modes (strict/tolerant)
- Constraint normalization
- Version validation
- DIFF format markers
- Tool registry validation
- Execution state model

### v2.x (Planned)
- Tool argument schemas
- INCLUDE directive for composition
- Step dependency graphs
- Streaming execution protocol

## License

GPL-3.0

## Contributing

Open issues for:
- Real-world usage feedback
- Constraint patterns that would help
- Integration challenges

PRs welcome for bug fixes and tests. Feature PRs should have an issue first.
