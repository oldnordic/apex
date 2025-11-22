APEX v1.1 — Hardening Addendum
==============================

Status: Addendum to APEX v1.0-draft (backwards-compatible)
Scope: Error recovery, tool safety, constraints, execution semantics

------------------------------------------------------------
0. RELATIONSHIP TO APEX v1.0
------------------------------------------------------------
This document extends the existing APEX v1.0 specification and does NOT
change the core grammar or block set.

- EBNF in v1.0 remains valid.
- All valid v1.0 documents are valid v1.1 documents.
- v1.1 adds:
  * version declaration rules,
  * constraint normalization,
  * clarified TOOLS semantics,
  * defined parser modes (strict vs tolerant),
  * execution state semantics (checkpointing),
  * minimal DIFF format markers,
  * explicit non-goals for v1.x.

Where conflicts arise, v1.1 rules take precedence.

------------------------------------------------------------
1. VERSION DECLARATION
------------------------------------------------------------
Problem:
- v1.0 defines spec versions but documents do not declare which version
  they use, making forward-compat and debugging harder.

Rule 1.1.1 — META version REQUIRED for production
- For production use, APEX documents MUST declare a version in META:
    META
    version=1.1
- For experimentation, absence of version is allowed but SHOULD be
  treated as v1.0.

Rule 1.1.2 — Version Parsing
- `version` value is a string in the form "<major>.<minor>".
- Unknown major version (>=2) MUST cause a hard error in strict mode.
- Unknown minor version MAY be treated as the closest known minor
  within the same major, but MUST emit a warning.

------------------------------------------------------------
2. CONSTRAINT NORMALIZATION
------------------------------------------------------------
Problem:
- v1.0 treats constraint lines as free text, leading to ambiguity
  ("no mocks" vs "NO_MOCKS" vs "NoMocks").

Rule 2.1 — Canonical Constraint Identifiers
- Constraints are interpreted as canonical identifiers derived from
  their line text by:
  1) trimming leading/trailing whitespace,
  2) converting to lowercase,
  3) replacing any sequence of non-alphanumeric characters with "_".

- Example mappings:
  "no mocks"     -> "no_mocks"
  "NO_MOCKS"     -> "no_mocks"
  "lt300loc"     -> "lt300loc"
  "real dbs only"-> "real_dbs_only"

Rule 2.2 — Known Standard Constraints
- The following identifiers are RESERVED with standard semantics:
  - "no_mocks"        : forbid mock databases or mock services
  - "real_dbs"        : require real databases / backends
  - "lt300loc"        : prefer modifications under ~300 LOC per file
  - "safe_refactor"   : only behavior-preserving changes allowed
  - "api_compat"      : public API signatures must not break

- Implementations MAY extend this list but MUST NOT alter the meaning
  of reserved identifiers.

Rule 2.3 — Constraint Precedence Reminder
- Precedence remains:
    1) CONSTRAINTS
    2) TASK
    3) GOALS
    4) PLAN
- If PLAN or TOOLS would violate a canonical constraint, the
  interpreter MUST either:
  - reject the document, or
  - adjust execution to satisfy the constraint and record this in META.

------------------------------------------------------------
3. TOOL SAFETY & ARGUMENT SEMANTICS
------------------------------------------------------------
Problem:
- v1.0 TOOLS lines are free-form; LLMs can hallucinate tool names or
  pass invalid arguments. Real-world tool calling shows frequent
  issues with incorrect tools, malformed arguments, and fabricated
  outputs.

3.1 Tool Name Validation
------------------------
Rule 3.1.1 — Known Tool Registry
- The runtime MUST maintain a registry of valid tool names.
- TOOLS lines MUST be validated against this registry.

Rule 3.1.2 — Unknown Tools
- If a TOOLS line references an unknown tool:
  - strict mode: MUST be a hard error.
  - tolerant mode: MAY drop that tool and mark the document as
    "tool_degraded" in META.

3.2 Tool Argument Handling
--------------------------
Rule 3.2.1 — APEX-Level Arguments Are Hints
- TOOLS block arguments are treated as HUMANS/LLM HINTS, not as
  fully-typed schemas. Example:
    TOOLS
    code_search "hnsw param mismatch"
- The APEX spec does NOT require the arguments here to be JSON or
  schema-valid. They are high-level intents.

Rule 3.2.2 — Schema Lives in Runtime
- Precise argument schemas (types, required fields) are owned by the
  runtime (e.g., SynCore MCP tool schema), not by APEX syntax.
- The interpreter MAY:
  - ignore TOOLS arguments and let the runtime construct arguments, or
  - use them as free-text inputs to schema-aware tooling.

Rule 3.2.3 — Argument Validation & Retry
- If the runtime detects argument schema violations for a tool call:
  - it MUST treat this as a recoverable error and MAY:
    - request a refined APEX document, or
    - re-prompt a model with the tool schema and the failing call.
- APEX itself does not encode the retry strategy but REQUIRES that
  such failures are not silently ignored.

3.3 Tool Hallucination Mitigation
---------------------------------
Rule 3.3.1 — No Ad-Hoc Tools
- The interpreter MUST NOT allow execution of any tool not explicitly
  present in the runtime registry, even if mentioned in NATURAL
  LANGUAGE within PLAN or CONTEXT.

Rule 3.3.2 — Tool-PLAN Alignment
- If PLAN mentions a tool-like operation (e.g. "run code_search") that
  has no matching TOOLS entry or registry entry:
  - strict mode: error.
  - tolerant mode: MAY insert a matching TOOLS entry if unambiguous,
    and MUST record the auto-fix in META.

------------------------------------------------------------
4. PARSER MODES & ERROR RECOVERY
------------------------------------------------------------
Problem:
- v1.0 only defines a de-facto strict parser; LLMs frequently produce
  slightly malformed APEX (missing newline, lowercase headers, etc.),
  which should be recoverable without adding new syntax.

4.1 Parse Modes
----------------
Rule 4.1.1 — Two Modes
- Implementations MUST support:
  - Strict mode   : v1.0-conformant parsing.
  - Tolerant mode : best-effort recovery with clearly defined limits.

Rule 4.1.2 — Strict Mode
- Strict mode behavior is unchanged from v1.0:
  - headers MUST be uppercase,
  - headers MUST be alone on their line,
  - blocks end only at next valid header or EOF,
  - violations cause hard errors.

4.2 Tolerant Mode (Recoverable Faults)
--------------------------------------
Rule 4.2.1 — Header Leniency
- In tolerant mode, the parser MAY:
  - accept lowercase or mixed-case block identifiers,
  - accept trailing spaces after identifiers.

Rule 4.2.2 — Unknown Blocks
- Unknown headers MAY be treated as COMMENT blocks and ignored.

Rule 4.2.3 — Error Reporting
- Any recovery performed in tolerant mode MUST:
  - be recorded as a list of "parse_fixes" in META,
  - NOT change the logical meaning (no invented blocks).

Rule 4.2.4 — Non-Recoverable Errors
- Documents with:
  - no TASK,
  - multiple TASK blocks,
  - overlapping blocks,
  - or ambiguous block boundaries
  MUST still be rejected even in tolerant mode.

------------------------------------------------------------
5. EXECUTION PLAN & STREAMING SEMANTICS
------------------------------------------------------------
Problem:
- v1.0 defines PLAN as ordered steps but does not specify how to
  represent partial execution, checkpointing, or resuming. Agent
  systems need to stop and continue long trajectories safely.

5.1 Execution State Model (Out-of-Band)
---------------------------------------
Rule 5.1.1 — Immutable APEX, Mutable State
- APEX documents remain immutable specifications.
- Execution state (progress, results) is stored OUTSIDE the APEX
  text (e.g., in LTMC or a runtime state store).

Rule 5.1.2 — Step Statuses
- Runtimes SHOULD represent each PLAN step with a status:
  - pending
  - running
  - complete
  - failed
  - skipped

- This state is not expressed in APEX syntax but in runtime metadata.

Rule 5.1.3 — Checkpoint Semantics
- A checkpoint is defined as:
  - the index of the last COMPLETED step,
  - the associated tool results,
  - validation outcomes so far.

- Runtimes MAY resume execution from the checkpoint without changing
  the APEX document, provided CONSTRAINTS are still respected.

5.2 PLAN Order Clarification
----------------------------
Rule 5.2.1 — Sequential Execution
- v1.1 reaffirms that PLAN steps are executed strictly in order, with
  no implicit parallelism and no DAG semantics.
- Complex dependency graphs MUST be modeled as multiple APEX documents
  or out-of-band orchestration.

------------------------------------------------------------
6. DIFF FORMAT GUIDANCE
------------------------------------------------------------
Problem:
- v1.0 DIFF is free text, which cannot be machine-validated.

Rule 6.1 — Optional Format Marker
- If DIFF is present, the first non-empty line MAY be a format marker:
  - "unified"  : content is unified diff
  - "raw"      : content is raw code or description

Example:
    DIFF
    unified
    --- a/src/lib.rs
    +++ b/src/lib.rs
    ...

Rule 6.2 — Validation Based on Format
- If format is "unified":
  - runtimes MAY attempt to apply the patch and verify it compiles.
- If format is "raw" or no marker:
  - DIFF is treated as opaque text as in v1.0.

------------------------------------------------------------
7. MULTI-APEX COMPOSITION & PLAN DEPENDENCIES
------------------------------------------------------------
Problem:
- v1.0 has no explicit multi-APEX composition or DAG PLAN semantics.
  Adding them at the DSL level could increase complexity and ambiguity.

Rule 7.1 — Explicit Non-Goals for v1.x
- APEX v1.x explicitly DOES NOT define:
  - INCLUDE semantics,
  - nested or hierarchical APEX documents,
  - PLAN dependency graphs (DAGs),
  - cross-document references.

Rule 7.2 — Recommended Orchestration Pattern
- Multi-step, multi-APEX workflows SHOULD be handled by an external
  orchestrator that:
  - generates one APEX document per TASK,
  - records dependencies in its own metadata,
  - enforces ordering at the orchestration layer.

- APEX remains a specification for a SINGLE, SEQUENTIAL task.

------------------------------------------------------------
8. SUMMARY OF v1.1 HARDENING
------------------------------------------------------------
APEX v1.1 addresses practical LLM weaknesses while keeping the DSL
simple:

- Version is now explicit (META version=1.1).
- Constraints are normalized and canonical ("no_mocks", "real_dbs", ...).
- TOOLS lines are validated against a runtime registry; arguments are
  treated as hints while schemas live in the runtime.
- Parser gains strict vs tolerant modes with clearly bounded recovery.
- Execution state and checkpointing are defined out-of-band without
  adding new syntax.
- DIFF can be optionally structured via a minimal "unified"/"raw"
  marker.
- Multi-APEX composition and DAG PLANs are explicitly out-of-scope
  for v1.x to avoid complexity and ambiguity.

------------------------------------------------------------
END OF APEX v1.1 HARDENING ADDENDUM
------------------------------------------------------------
