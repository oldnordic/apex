APEX v1 — Full Specification Draft (RFC-Style)

APEX v1 — Agent Planning & Execution Specification
==================================================

Version: 1.0-draft
Status: Draft for Internal Use (SynCore / OdinCode)
Author: Feanor (APEX Designer)
Scope: LLM Reasoning & Tool-Orchestration DSL
Audience: Mini Model, Big Model, SynCore Runtime, OdinCode Agents

--------------------------------------------------
1. PURPOSE
--------------------------------------------------
APEX is a deterministic, text-based DSL for controlling reasoning,
planning, execution, and validation steps in agentic LLM systems.

APEX defines a stable grammar and semantics that:
- constrain large language models,
- enforce predictable planning,
- coordinate multi-step tool execution,
- map natural language into structured agent workflows,
- avoid hallucinations by bounding model behavior,
- ensure reproducibility through LTMC trace storage.

APEX is NOT a data format (like JSON/TOML/YAML).
APEX is NOT a transport encoding.
APEX is a "reasoning language" for AI agents.

--------------------------------------------------
2. DESIGN GOALS
--------------------------------------------------
APEX is intended to provide:
1. Deterministic block structure
2. Low-token overhead
3. Clear separation of intent vs execution
4. Validatable syntax and semantics
5. Machine-parseable grammar
6. Human readability
7. Model interpretability and stability
8. Round-trip transformation to/from NLP and JSON

Primary target users:
- The mini-model (planner)
- The big-model (executor)
- SynCore real MCP tools
- OdinCode development agents

--------------------------------------------------
3. DOCUMENT STRUCTURE OVERVIEW
--------------------------------------------------
An APEX document consists of a sequence of BLOCKS.
Each block begins with an IDENTIFIER on a new line.
Block order is partially flexible but recommended ordering is defined.

Core blocks:

    TASK           — description of the primary objective
    GOALS          — measurable, bounded desired outcomes
    PLAN           — step-by-step ordered execution plan
    CONSTRAINTS    — rules the agent MUST obey
    VALIDATION     — checks to confirm correctness
    TOOLS          — tool calls or mappings (optional)
    DIFF           — expected or generated code diff (optional)
    CONTEXT        — additional relevant information (optional)
    META           — version, author, timestamps (optional)

Only TASK is mandatory.

--------------------------------------------------
4. BLOCK DEFINITIONS
--------------------------------------------------

4.1 TASK (required)
-------------------
Defines the core objective.

Format:
    TASK <free text>

Example:
    TASK refactor hnsw search path to fix param mismatch

Rules:
- Must appear exactly once.
- Must be a single line.
- Represents the grounding intent for all other sections.

4.2 GOALS (optional but recommended)
------------------------------------
Describes desired outcomes and success criteria.

Format:
    GOALS
    <free text lines>

Example:
    GOALS
    improve recall
    maintain memory footprint
    ensure no regression in vector similarity

Rules:
- Any number of lines allowed.
- No required syntax for bullet points.

4.3 PLAN (recommended)
----------------------
A sequential list of steps.

Format:
    PLAN
    step text...
    step text...
    step text...

Example:
    PLAN
    audit code
    patch missing search param
    run real tests
    verify neo4j graph fusion

Rules:
- Order matters.
- Each line is interpreted as one step.

4.4 CONSTRAINTS (strongly enforced)
-----------------------------------
System rules the agent MUST NOT violate.

Format:
    CONSTRAINTS
    <rules>

Recommended constraints:
- lt300loc
- real dbs only
- no mocks
- safe refactor only
- maintain api compatibility

Rules:
- Violation should produce an error or refusal.

4.5 VALIDATION (required for tool execution)
--------------------------------------------
Defines objective checks the agent must perform.

Format:
    VALIDATION
    <conditions>

Examples:
    VALIDATION
    run e2e tests
    ensure hnsw recall >= baseline
    diff code before vs after

Rules:
- Models use these to self-evaluate output.
- SynCore may enforce validation gates.

4.6 TOOLS (optional)
--------------------
Maps plan steps to specific SynCore MCP tools.

Format:
    TOOLS
    toolname arguments...

Example:
    TOOLS
    code_search query="hnsw param"
    neo4j_query "match (n) return count(n)"

Rules:
- Optional for simple tasks.
- Required for deterministic tool pipelines.

4.7 DIFF (optional)
-------------------
Contains code patches or expected diffs.

Format:
    DIFF
    <patch content>

Rules:
- Interpreted as unified diff or raw modification block.

4.8 CONTEXT + META (optional)
-----------------------------
Free-form notes or metadata.

Format:
    CONTEXT
    META

--------------------------------------------------
5. SYNTAX RULES
--------------------------------------------------
1. Block identifiers MUST be uppercase.
2. A block identifier MUST be the only text on its line.
3. Block content continues until the next identifier or EOF.
4. TASK MUST appear once.
5. Blocks SHOULD appear in recommended order but not required.
6. Empty blocks are invalid (except META/CONTEXT).

--------------------------------------------------
6. SEMANTIC RULES
--------------------------------------------------
1. PLAN semantics:
   - Steps must be executed in given order.
   - Steps are deterministic.

2. CONSTRAINTS semantics:
   - These override PLAN.
   - Any step violating a constraint is invalid.

3. VALIDATION semantics:
   - All validation tasks must succeed for the plan to be "complete".
   - Big model must verify outputs satisfy conditions.

4. TOOL semantics:
   - Tools must map to known SynCore tools.
   - Invalid tool name = validation error.

5. DIFF semantics:
   - If DIFF is present, big model must generate consistent patches.

6. NLP ↔ APEX semantics:
   - NLP input must be fully converted to APEX before execution.

--------------------------------------------------
7. ERROR RULES
--------------------------------------------------
APEX parsers must detect:
- Missing TASK
- Empty PLAN (if PLAN used)
- Unknown identifiers
- Malformed block transitions
- Invalid tool names
- Constraint conflicts

Parser modes:
- strict: reject invalid structure
- tolerant: warn and continue

--------------------------------------------------
8. RECOMMENDED ORDER (not enforced)
--------------------------------------------------
    TASK
    GOALS
    PLAN
    CONSTRAINTS
    VALIDATION
    TOOLS
    DIFF
    CONTEXT
    META

--------------------------------------------------
9. TOKEN-EFFICIENT MODE (APEX-TINY)
--------------------------------------------------
For low-token environments:
- Remove ":" after identifiers
- Remove bullets
- Single-line TASK
- No indentation

Example Tiny APEX:
    TASK fix hnsw param mismatch
    GOALS improve recall maintain memory
    PLAN audit code fix param run tests
    CONSTRAINTS real dbs no mocks lt300loc
    VALIDATION e2e tests diff check

--------------------------------------------------
10. VERSIONING
--------------------------------------------------
APEX v1.0-draft defines:
- Core blocks
- Required grammar
- Minimal token format
- Semantics for execution

APEX v1.x will extend:
- optional modifiers
- macro blocks
- tool schemas

APEX v2.x may introduce:
- typed arguments
- nested plans
- composable tasks (APEX modules)

--------------------------------------------------
11. EXAMPLE FULL DOCUMENT
--------------------------------------------------
TASK fix hnsw search parameter mismatch
GOALS
improve recall
preserve memory footprint
PLAN
audit code
fix param mismatch
update tests
validate recall
CONSTRAINTS
real dbs
no mocks
lt300loc
safe refactor
VALIDATION
run e2e tests
compare recall baseline
TOOLS
code_search "hnsw"
vector_search "search param"
META
version 1.0

--------------------------------------------------
END OF SPEC
--------------------------------------------------

