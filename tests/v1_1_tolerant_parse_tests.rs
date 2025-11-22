//! APEX v1.1 Tolerant Parse Mode Tests

use apex_spec::{parse_str_with_mode, ParseMode, BlockKind};

#[test]
fn test_tolerant_accepts_lowercase_headers() {
    let input = r#"task
Do something

plan
Step 1
Step 2
"#;
    let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();

    assert!(result.document.task().is_some());
    assert!(result.document.plan().is_some());
}

#[test]
fn test_tolerant_records_fixes() {
    let input = r#"task
Do something

plan
Step 1
"#;
    let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();

    assert!(!result.fixes.is_empty());
    assert!(result.fixes.iter().any(|f| f.description.contains("task")));
    assert!(result.fixes.iter().any(|f| f.description.contains("plan")));
}

#[test]
fn test_strict_rejects_lowercase() {
    let input = r#"task
Do something
"#;
    let result = parse_str_with_mode(input, ParseMode::Strict).unwrap();

    // Strict mode treats lowercase as content, not header
    assert!(result.document.task().is_none());
    assert!(result.fixes.is_empty());
}

#[test]
fn test_tolerant_mixed_case() {
    let input = r#"Task
Do something

Goals
Be successful

PLAN
Step 1
"#;
    let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();

    assert!(result.document.task().is_some());
    assert!(result.document.goals().is_some());
    assert!(result.document.plan().is_some());

    // Only Task and Goals needed fixes, PLAN was already uppercase
    let fixed_count = result.fixes.len();
    assert_eq!(fixed_count, 2);
}

#[test]
fn test_tolerant_preserves_content() {
    let input = r#"task
Do something important

constraints
no mocks
real dbs only
"#;
    let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();

    let task = result.document.task().unwrap();
    assert_eq!(task.content(), "Do something important");

    let constraints = result.document.constraints().unwrap();
    let lines = constraints.content_lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "no mocks");
}

#[test]
fn test_tolerant_still_requires_task() {
    let input = r#"plan
Step 1
Step 2
"#;
    let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();

    // Document parses but has no task (only plan)
    assert!(result.document.task().is_none());
    assert!(result.document.plan().is_some());
}

#[test]
fn test_fix_line_numbers() {
    let input = r#"task
Do something

goals
Be successful
"#;
    let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();

    // Check fixes have correct line numbers
    let task_fix = result.fixes.iter().find(|f| f.description.contains("task")).unwrap();
    assert_eq!(task_fix.line, 1);

    let goals_fix = result.fixes.iter().find(|f| f.description.contains("goals")).unwrap();
    assert_eq!(goals_fix.line, 4);
}

#[test]
fn test_tolerant_all_blocks_lowercase() {
    let input = r#"task
Test all blocks

goals
Goal 1

plan
Step 1

constraints
Constraint 1

validation
Check 1

tools
code_search

context
Some context

meta
version=1.1
"#;
    let result = parse_str_with_mode(input, ParseMode::Tolerant).unwrap();

    assert!(result.document.task().is_some());
    assert!(result.document.goals().is_some());
    assert!(result.document.plan().is_some());
    assert!(result.document.constraints().is_some());
    assert!(result.document.validation().is_some());
    assert!(result.document.tools().is_some());
    assert!(result.document.context().is_some());
    assert!(result.document.meta().is_some());

    // All 8 blocks had lowercase headers
    assert_eq!(result.fixes.len(), 8);
}

#[test]
fn test_strict_mode_no_fixes() {
    let input = r#"TASK
Do something

PLAN
Step 1
"#;
    let result = parse_str_with_mode(input, ParseMode::Strict).unwrap();

    assert!(result.fixes.is_empty());
    assert!(result.document.task().is_some());
    assert!(result.document.plan().is_some());
}
