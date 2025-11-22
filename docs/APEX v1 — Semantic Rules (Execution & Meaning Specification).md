APEX v1 — Semantic Rules (Execution & Meaning Specification)

APEX v1 — Semantic Rules Specification
======================================

Version: 1.0-draft
Scope: Interpretation rules, meaning, constraints, agent behavior
Applies to: All mini-models, big-models, SynCore interpreters

------------------------------------------------------------
1. PURPOSE OF SEMANTICS
------------------------------------------------------------
Syntax defines how APEX *looks*.  
Semantics define how APEX *behaves*.

These rules govern:
- how APEX blocks affect reasoning,
- how steps are executed,
- how constraints override plans,
- how validation determines correctness,
- how Parsers and Agents must interpret APEX.

Semantics ensure deterministic agent behavior.

------------------------------------------------------------
2. DOCUMENT SEMANTICS
------------------------------------------------------------
1. One TASK block is REQUIRED.
2. Other blocks are OPTIONAL but recommended.
3. Blocks must be interpreted in relation to the TASK.
4. The order of blocks does not change semantics.
5. Block boundaries define scope of interpretation.

------------------------------------------------------------
3. BLOCK SEMANTICS
------------------------------------------------------------

----------------------------
3.1 TASK Block
----------------------------
Purpose:
- Defines the *primary objective*.
- All reasoning MUST align with this objective.

Rules:
- Single-line.
- Must appear once.
- If multiple TASK blocks appear, interpreter MUST error.
- If missing, interpreter MUST error.

Meaning:
- TASK anchors the entire document.
- Overrides ambiguous PLAN or GOALS entries.

----------------------------
3.2 GOALS Block
----------------------------
Purpose:
- States high-level desired outcomes.
- Used to evaluate PLAN completeness.

Rules:
- Optional.
- Zero or more lines.
- Not required to be structured.

Meaning:
- GOALS refine the intent of TASK.
- GOALS guide PLAN creation.
- GOALS may add non-functional objectives (perf, quality, safety).

Interpretation precedence:
1) TASK  
2) GOALS  
3) PLAN  

----------------------------
3.3 PLAN Block
----------------------------
Purpose:
- Defines ordered execution steps.

Rules:
- Optional but strongly recommended.
- Each line is one atomic step.
- Steps MUST be executed in listed order.

Meaning:
- PLAN is the deterministic procedure to achieve TASK/GOALS.
- If PLAN contradicts TASK → PLAN is invalid.
- If PLAN violates CONSTRAINTS → interpreter MUST adjust or reject.

Execution rules:
- Agents must follow steps sequentially.
- Each step corresponds to either:
  * reasoning,
  * tool selection,
  * code modification,
  * validation preparation.

----------------------------
3.4 CONSTRAINTS Block
----------------------------
Purpose:
- Hard rules that override all other blocks.
- Safety and correctness guards.

Rules:
- Optional but critical.
- Violation MUST cause failure or task rejection.

Examples:
- "real dbs"
- "no mocks"
- "lt300loc"
- "safe refactor only"
- "api compatibility retained"

Meaning:
- CONSTRAINTS override PLAN.
- CONSTRAINTS override GOALS.
- CONSTRAINTS MUST be applied before execution.

Priority:
1) CONSTRAINTS  
2) TASK  
3) GOALS  
4) PLAN  

----------------------------
3.5 VALIDATION Block
----------------------------
Purpose:
- Specifies MUST-PASS checks for task completion.

Rules:
- Optional but required for tool workflows where correctness matters.
- Interpreter MUST run all listed validations.
- If any validation fails → task invalid.

Meaning:
- VALIDATION ensures correct execution.
- Agents must self-check output.
- Interpreters must confirm conditions are met.

Validation types:
- test execution
- performance benchmarks
- code diffs
- database checks
- embedding search correctness
- graph consistency checks

----------------------------
3.6 TOOLS Block
----------------------------
Purpose:
- Maps PLAN steps to concrete tool invocations.

Rules:
- Optional.
- Each line MUST reference a known SynCore MCP tool.
- Arguments may be free-form.

Meaning:
- Tools provide deterministic execution.
- Interpreter must parse tool lines and match them to real tool calls.
- Unknown tools MUST cause validation error.

Tool Semantics:
- Tool lines override LLM decision-making for step execution.
- If a PLAN step references a tool not declared, interpreter may allow or deny.

----------------------------
3.7 DIFF Block
----------------------------
Purpose:
- Holds expected or produced code modifications.

Rules:
- Optional.
- May contain unified diff or raw code.
- Interpreters do NOT apply semantics to DIFF; they treat it as opaque.

Meaning:
- DIFF expresses explicit changes from PLAN execution.
- DIFF must satisfy VALIDATION conditions.

----------------------------
3.8 CONTEXT Block
----------------------------
Purpose:
- Stores supplemental context for reasoning.

Rules:
- Optional.
- No semantics imposed.

Meaning:
- CONTEXT influences reasoning but not constraints or execution.

----------------------------
3.9 META Block
----------------------------
Purpose:
- Metadata, tracing, versioning, timestamps.

Rules:
- Optional.

Meaning:
- META does not influence reasoning or execution.
- Useful for LTMC trace storage.

------------------------------------------------------------
4. INTER-BLOCK SEMANTICS
------------------------------------------------------------

----------------------------
4.1 Precedence Rules
----------------------------
When blocks conflict:

1. CONSTRAINTS override all
2. TASK overrides GOALS, PLAN
3. GOALS override PLAN
4. PLAN defines procedural steps

Example:
- If PLAN says: "use mock db"
- But CONSTRAINTS says "no mocks"
→ Interpreter MUST reject or rewrite PLAN.

----------------------------
4.2 Execution Flow
----------------------------
Document interpretation order:

1. Load blocks
2. Validate structure
3. Apply CONSTRAINTS
4. Check TASK
5. Align GOALS with TASK
6. Convert PLAN into executable steps
7. Resolve TOOLS mapping
8. Execute steps in order
9. Perform VALIDATION
10. Produce DIFF (optional)
11. Emit final result + LTMC trace

----------------------------
4.3 Error Conditions
----------------------------
Interpreter MUST error on:
- missing TASK
- multiple TASK blocks
- PLAN inconsistent with TASK
- PLAN violating CONSTRAINTS
- unknown block identifiers
- cyclic PLAN steps (rare)
- invalid tool names
- validation failure

Error MUST be explicit and machine-readable.

------------------------------------------------------------
5. NLP ↔ APEX SEMANTICS
------------------------------------------------------------
--------------------------------
5.1 NLP → APEX
--------------------------------
Rules for conversion:
- All user natural language MUST become APEX before execution.
- No reasoning outside APEX is allowed.
- Interpret user intent → TASK.
- Extract desired outcomes → GOALS.
- Generate structured steps → PLAN.
- Apply system safety → CONSTRAINTS.
- Add required correctness checks → VALIDATION.

--------------------------------
5.2 APEX → NLP
--------------------------------
Rules for conversion:
- APEX blocks may be expanded into natural-language explanations.
- PLAN steps → paragraphs.
- GOALS → summary description.
- TASK → title sentence.

Used for human readability, not execution.

------------------------------------------------------------
6. JSON ↔ APEX SEMANTICS
------------------------------------------------------------
--------------------------------
6.1 APEX → JSON (Tool Calls)
--------------------------------
- TOOLS block lines become structured JSON arguments.
- PLAN steps referencing tools map to a JSON execution plan.

--------------------------------
6.2 JSON → APEX
--------------------------------
- JSON tool input should be converted into APEX’s TOOLS block.
- JSON "steps" convert into PLAN items.

------------------------------------------------------------
7. TOKEN-EFFICIENT SEMANTICS (APEX-TINY)
------------------------------------------------------------
APEX may be written with:
- no colons
- minimal whitespace
- single-line TASK
- combined GOALS
- compressed PLAN

Semantics remain unchanged.

Interpretation rule:
- Content after identifier until next identifier is that block.

Example:
    TASK fix hnsw param mismatch
    PLAN audit fix test
Meaning is identical to full version.

------------------------------------------------------------
8. SEMANTICS FOR AGENTIC SYSTEMS
------------------------------------------------------------
For SynCore and OdinCode:

1. Mini-model MUST produce valid APEX.
2. Big-model MUST obey APEX strictly.
3. Interpreters MUST enforce constraints.
4. LTMC MUST store:
    - original APEX
    - interpreted plan
    - executed tool calls
    - validation results
    - DIFF (if present)

Agents MUST NOT:
- skip constraints
- reorder steps
- modify APEX blocks without revalidation

Agents MUST:
- respect determinism
- keep reasoning inside APEX boundaries
- use APEX for retry/recovery

------------------------------------------------------------
9. FORMAL COMPLETION RULE
------------------------------------------------------------
An APEX-driven task is considered "COMPLETE" only if:

1. PLAN executed successfully
2. No CONSTRAINTS violated
3. VALIDATION passed
4. DIFF matches expected outcome (if DIFF exists)
5. No errors encountered
6. Final state stored in LTMC

------------------------------------------------------------
END OF SEMANTIC SPEC
------------------------------------------------------------
