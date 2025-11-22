//! APEX v1.1 Constraint Canonicalization Tests

use apex_spec::{canonicalize, normalize_constraint, Constraint};

#[test]
fn test_canonicalize_basic() {
    assert_eq!(canonicalize("No Mocks"), "no_mocks");
    assert_eq!(canonicalize("NO_MOCKS"), "no_mocks");
    assert_eq!(canonicalize("no-mocks"), "no_mocks");
    assert_eq!(canonicalize("NoMocks"), "nomocks");
}

#[test]
fn test_canonicalize_whitespace() {
    assert_eq!(canonicalize("  no mocks  "), "no_mocks");
    assert_eq!(canonicalize("real   dbs   only"), "real_dbs_only");
    assert_eq!(canonicalize("\tno mocks\n"), "no_mocks");
}

#[test]
fn test_canonicalize_special_chars() {
    assert_eq!(canonicalize("< 300 LOC"), "300_loc");
    assert_eq!(canonicalize("API compatibility!"), "api_compatibility");
    assert_eq!(canonicalize("no--mocks--allowed"), "no_mocks_allowed");
}

#[test]
fn test_canonicalize_already_canonical() {
    assert_eq!(canonicalize("no_mocks"), "no_mocks");
    assert_eq!(canonicalize("lt300loc"), "lt300loc");
    assert_eq!(canonicalize("real_dbs"), "real_dbs");
}

#[test]
fn test_constraint_from_canonical() {
    assert_eq!(Constraint::from_str("no_mocks"), Constraint::NoMocks);
    assert_eq!(Constraint::from_str("real_dbs"), Constraint::RealDbsOnly);
    assert_eq!(Constraint::from_str("safe_refactor"), Constraint::SafeRefactor);
    assert_eq!(Constraint::from_str("api_compat"), Constraint::ApiCompat);
}

#[test]
fn test_constraint_from_natural_language() {
    assert_eq!(Constraint::from_str("No Mocks Allowed"), Constraint::NoMocks);
    assert_eq!(Constraint::from_str("Real databases only"), Constraint::RealDbsOnly);
    assert_eq!(Constraint::from_str("Safe refactoring"), Constraint::SafeRefactor);
    assert_eq!(Constraint::from_str("API compatibility required"), Constraint::ApiCompat);
}

#[test]
fn test_constraint_loc_limit() {
    assert_eq!(Constraint::from_str("lt300loc"), Constraint::LtLoc(300));
    assert_eq!(Constraint::from_str("< 500 LOC"), Constraint::LtLoc(500));
    assert_eq!(Constraint::from_str("lt_200_loc"), Constraint::LtLoc(200));
    // Natural language "less than X lines of code" doesn't contain "loc" keyword
    // so it becomes a custom constraint - this is expected behavior
}

#[test]
fn test_constraint_custom_normalized() {
    let c = Constraint::from_str("My Custom Rule!");
    match c {
        Constraint::Other(s) => assert_eq!(s, "my_custom_rule"),
        _ => panic!("Expected Other variant"),
    }
}

#[test]
fn test_normalize_equals_canonicalize() {
    let inputs = vec![
        "No Mocks",
        "real dbs only",
        "< 300 LOC",
        "API compatibility!",
    ];

    for input in inputs {
        assert_eq!(normalize_constraint(input), canonicalize(input));
    }
}
