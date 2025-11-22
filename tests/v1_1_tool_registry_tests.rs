//! APEX v1.1 Tool Registry Validation Tests

use apex_spec::{
    parse_str, validate_with_mode, ValidationMode,
    ToolRegistry, VALID_TOOLS, extract_tool_name,
};

#[test]
fn test_default_registry_tools() {
    let registry = ToolRegistry::new();

    // Check standard tools are valid
    assert!(registry.is_valid("code_search"));
    assert!(registry.is_valid("vector_search"));
    assert!(registry.is_valid("graph_query"));
    assert!(registry.is_valid("memory.query"));
    assert!(registry.is_valid("unix_action"));
}

#[test]
fn test_registry_rejects_unknown() {
    let registry = ToolRegistry::new();

    assert!(!registry.is_valid("fake_tool"));
    assert!(!registry.is_valid("hallucinated_api"));
}

#[test]
fn test_mcp_tools_always_valid() {
    let registry = ToolRegistry::new();

    assert!(registry.is_valid("mcp__jenkins__build_job"));
    assert!(registry.is_valid("mcp__github__create_pr"));
    assert!(registry.is_valid("mcp__any__tool"));
}

#[test]
fn test_strict_mode_rejects_unknown_tool() {
    let input = r#"TASK
Do something

TOOLS
fake_unknown_tool(args)

META
version=1.1
"#;
    let doc = parse_str(input).unwrap();
    let registry = ToolRegistry::new();
    let result = validate_with_mode(doc, ValidationMode::Strict, Some(&registry));

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown tool"));
}

#[test]
fn test_lenient_mode_warns_unknown_tool() {
    let input = r#"TASK
Do something

TOOLS
fake_unknown_tool(args)

META
version=1.1
"#;
    let doc = parse_str(input).unwrap();
    let registry = ToolRegistry::new();
    let validated = validate_with_mode(doc, ValidationMode::Lenient, Some(&registry)).unwrap();

    assert!(validated.warnings.iter().any(|w| w.contains("tool_degraded")));
}

#[test]
fn test_legacy_mode_ignores_unknown_tool() {
    let input = r#"TASK
Do something

TOOLS
fake_unknown_tool(args)
"#;
    let doc = parse_str(input).unwrap();
    let registry = ToolRegistry::new();
    let validated = validate_with_mode(doc, ValidationMode::Legacy, Some(&registry)).unwrap();

    // Legacy mode does not add tool warnings
    assert!(!validated.warnings.iter().any(|w| w.contains("tool")));
}

#[test]
fn test_permissive_registry() {
    let registry = ToolRegistry::permissive();

    assert!(registry.is_valid("any_tool"));
    assert!(registry.is_valid("completely_made_up"));
}

#[test]
fn test_custom_registry() {
    let mut registry = ToolRegistry::empty();
    registry.add_tool("my_custom_tool");

    assert!(registry.is_valid("my_custom_tool"));
    assert!(!registry.is_valid("code_search")); // Not in custom registry
}

#[test]
fn test_extract_tool_name_simple() {
    assert_eq!(extract_tool_name("code_search"), "code_search");
    assert_eq!(extract_tool_name("  vector_search  "), "vector_search");
}

#[test]
fn test_extract_tool_name_with_args() {
    assert_eq!(extract_tool_name("code_search(query)"), "code_search");
    assert_eq!(extract_tool_name("read_file(path, opts)"), "read_file");
}

#[test]
fn test_extract_tool_name_with_string_arg() {
    assert_eq!(extract_tool_name("code_search \"pattern\""), "code_search");
    assert_eq!(extract_tool_name("grep \"error\""), "grep");
}

#[test]
fn test_valid_tools_constant() {
    assert!(VALID_TOOLS.contains(&"code_search"));
    assert!(VALID_TOOLS.contains(&"memory.query"));
    assert!(!VALID_TOOLS.is_empty());
}
