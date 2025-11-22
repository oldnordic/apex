//! APEX v1.1 Version Enforcement Tests

use apex_spec::{parse_str, validate_with_mode, ValidationMode, ToolRegistry};

#[test]
fn test_legacy_mode_no_version_required() {
    let input = r#"TASK
Do something
"#;
    let doc = parse_str(input).unwrap();
    let validated = validate_with_mode(doc, ValidationMode::Legacy, None).unwrap();

    assert!(validated.warnings.is_empty());
}

#[test]
fn test_strict_mode_warns_missing_version() {
    let input = r#"TASK
Do something

META
author=test
"#;
    let doc = parse_str(input).unwrap();
    let validated = validate_with_mode(doc, ValidationMode::Strict, None).unwrap();

    assert!(validated.warnings.iter().any(|w| w.contains("Missing version")));
}

#[test]
fn test_strict_mode_warns_missing_meta() {
    let input = r#"TASK
Do something
"#;
    let doc = parse_str(input).unwrap();
    let validated = validate_with_mode(doc, ValidationMode::Strict, None).unwrap();

    assert!(validated.warnings.iter().any(|w| w.contains("Missing META")));
}

#[test]
fn test_valid_v1_version() {
    let input = r#"TASK
Do something

META
version=1.1
"#;
    let doc = parse_str(input).unwrap();
    let validated = validate_with_mode(doc, ValidationMode::Strict, None).unwrap();

    // Should not have version warning
    assert!(!validated.warnings.iter().any(|w| w.contains("Missing version")));
}

#[test]
fn test_v1_0_version_compatible() {
    let input = r#"TASK
Do something

META
version=1.0
"#;
    let doc = parse_str(input).unwrap();
    let validated = validate_with_mode(doc, ValidationMode::Strict, None).unwrap();

    assert!(validated.meta.as_ref().unwrap().is_version_compatible());
}

#[test]
fn test_v2_version_incompatible() {
    let input = r#"TASK
Do something

META
version=2.0
"#;
    let doc = parse_str(input).unwrap();
    let result = validate_with_mode(doc, ValidationMode::Strict, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported"));
}

#[test]
fn test_lenient_mode_allows_missing_version() {
    let input = r#"TASK
Do something
"#;
    let doc = parse_str(input).unwrap();
    let validated = validate_with_mode(doc, ValidationMode::Lenient, None).unwrap();

    // Lenient mode should not add version warnings
    assert!(!validated.warnings.iter().any(|w| w.contains("version")));
}

#[test]
fn test_meta_view_version_accessor() {
    let input = r#"TASK
Do something

META
version=1.1
author=test
"#;
    let doc = parse_str(input).unwrap();
    let validated = validate_with_mode(doc, ValidationMode::Legacy, None).unwrap();

    let meta = validated.meta.unwrap();
    assert_eq!(meta.version(), Some("1.1"));
    assert!(meta.is_version_compatible());
}
