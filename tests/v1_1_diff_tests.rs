//! APEX v1.1 DIFF Format Marker Tests

use apex_spec::{parse_str, validate, DiffFormat};

#[test]
fn test_diff_unified_marker() {
    let input = r#"TASK
Apply patch

DIFF
unified
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,4 @@
+// New comment
 fn main() {}
"#;
    let validated = apex_spec::parse_and_validate(input).unwrap();
    let diff = validated.diff.unwrap();

    assert_eq!(diff.format, DiffFormat::Unified);
    assert!(!diff.changes.is_empty());
    assert!(diff.changes[0].starts_with("---")); // First line after marker
}

#[test]
fn test_diff_raw_marker() {
    let input = r#"TASK
Apply changes

DIFF
raw
Some raw code here
More code
"#;
    let validated = apex_spec::parse_and_validate(input).unwrap();
    let diff = validated.diff.unwrap();

    assert_eq!(diff.format, DiffFormat::Raw);
    assert_eq!(diff.changes.len(), 2);
}

#[test]
fn test_diff_no_marker() {
    let input = r#"TASK
Apply changes

DIFF
--- a/file.rs
+++ b/file.rs
"#;
    let validated = apex_spec::parse_and_validate(input).unwrap();
    let diff = validated.diff.unwrap();

    assert_eq!(diff.format, DiffFormat::Unspecified);
    // First line is part of content when no marker
    assert!(diff.changes[0].starts_with("---"));
}

#[test]
fn test_diff_empty() {
    let input = r#"TASK
Do something

DIFF

"#;
    let validated = apex_spec::parse_and_validate(input).unwrap();
    let diff = validated.diff.unwrap();

    assert_eq!(diff.format, DiffFormat::Unspecified);
    assert!(diff.changes.is_empty());
}

#[test]
fn test_diff_unified_case_insensitive() {
    let input = r#"TASK
Apply patch

DIFF
UNIFIED
--- a/file.rs
+++ b/file.rs
"#;
    let validated = apex_spec::parse_and_validate(input).unwrap();
    let diff = validated.diff.unwrap();

    assert_eq!(diff.format, DiffFormat::Unified);
}

#[test]
fn test_diff_format_default() {
    assert_eq!(DiffFormat::default(), DiffFormat::Unspecified);
}
