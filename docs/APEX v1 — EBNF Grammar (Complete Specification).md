APEX v1 — EBNF Grammar (Complete Specification) 

APEX Grammar (EBNF)
===================

Version: 1.0-draft
Purpose: Deterministic grammar for APEX agent-planning DSL
Notation: ISO EBNF

------------------------------------------------------------
1. LEXICAL ELEMENTS
------------------------------------------------------------

letter          = "A"…"Z" | "a"…"z" ;
digit           = "0"…"9" ;
alphanum        = letter | digit | "_" | "-" | "/" | "." ;
space           = " " ;
newline         = "\n" | "\r\n" ;
text_char       = ? any printable character except newline ? ;

identifier_char = letter | digit | "_" ;

------------------------------------------------------------
2. IDENTIFIERS (BLOCK HEADERS)
------------------------------------------------------------

block_identifier =
      "TASK"
    | "GOALS"
    | "PLAN"
    | "CONSTRAINTS"
    | "VALIDATION"
    | "TOOLS"
    | "DIFF"
    | "CONTEXT"
    | "META"
    ;

------------------------------------------------------------
3. DOCUMENT STRUCTURE
------------------------------------------------------------

apex_document =
    { block } ;

block =
    block_header newline
    block_body
;

block_header =
    block_identifier
;

block_body =
    { block_line }
;

block_line =
    line_text newline
  | newline
;

line_text =
    { text_char }
;

------------------------------------------------------------
4. TASK BLOCK
------------------------------------------------------------

task_block =
    "TASK" newline
    task_line
;

task_line =
    line_text newline
;

------------------------------------------------------------
5. GOALS BLOCK
------------------------------------------------------------

goals_block =
    "GOALS" newline
    { goal_line }
;

goal_line =
    line_text newline
;

------------------------------------------------------------
6. PLAN BLOCK
------------------------------------------------------------

plan_block =
    "PLAN" newline
    { plan_step }
;

plan_step =
    line_text newline
;

------------------------------------------------------------
7. CONSTRAINTS BLOCK
------------------------------------------------------------

constraints_block =
    "CONSTRAINTS" newline
    { constraint_line }
;

constraint_line =
    line_text newline
;

------------------------------------------------------------
8. VALIDATION BLOCK
------------------------------------------------------------

validation_block =
    "VALIDATION" newline
    { validation_line }
;

validation_line =
    line_text newline
;

------------------------------------------------------------
9. TOOLS BLOCK
------------------------------------------------------------

tools_block =
    "TOOLS" newline
    { tool_line }
;

tool_line =
    line_text newline
;

------------------------------------------------------------
10. DIFF BLOCK
------------------------------------------------------------

diff_block =
    "DIFF" newline
    { diff_line }
;

diff_line =
    line_text newline
;

------------------------------------------------------------
11. CONTEXT BLOCK
------------------------------------------------------------

context_block =
    "CONTEXT" newline
    { context_line }
;

context_line =
    line_text newline
;

------------------------------------------------------------
12. META BLOCK
------------------------------------------------------------

meta_block =
    "META" newline
    { meta_line }
;

meta_line =
    line_text newline
;

------------------------------------------------------------
13. DOCUMENT CONSTRAINTS (APPLIED AT VALIDATION TIME)
------------------------------------------------------------

(These are semantic, not syntax rules; parsers apply them.)

constraints:
    - Exactly one TASK block must exist.
    - Blocks must appear at block boundaries only.
    - No inline block identifiers allowed inside block_body.
    - Identifiers must be uppercase.
    - Empty blocks are invalid except META/CONTEXT.
    - PLAN lines represent sequential ordered steps.
    - Block content ends at the next block_identifier.

------------------------------------------------------------
END OF APEX v1 EBNF GRAMMAR
------------------------------------------------------------
