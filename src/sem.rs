//! APEX Semantics
//!
//! Higher-level semantic analysis and precedence rules.
//!
//! ## v1.1 Constraint Normalization
//!
//! Per APEX v1.1, constraints are normalized to canonical identifiers:
//! 1. Trim leading/trailing whitespace
//! 2. Convert to lowercase
//! 3. Replace any sequence of non-alphanumeric characters with "_"
//!
//! Example: "No Mocks Allowed!" -> "no_mocks_allowed"

use crate::validate::ValidatedDocument;
use serde::{Deserialize, Serialize};

/// Canonicalize a constraint string per APEX v1.1 spec
///
/// This is the official v1.1 canonicalization function.
/// Algorithm:
/// 1. Trim leading/trailing whitespace
/// 2. Convert to lowercase
/// 3. Replace any sequence of non-alphanumeric characters with "_"
/// 4. Collapse repeated "__" to single "_"
/// 5. Trim leading/trailing underscores
///
/// # Examples
/// ```
/// use apex_spec::sem::canonicalize;
/// assert_eq!(canonicalize("No Mocks"), "no_mocks");
/// assert_eq!(canonicalize("NO_MOCKS"), "no_mocks");
/// assert_eq!(canonicalize("real dbs only"), "real_dbs_only");
/// assert_eq!(canonicalize("< 300 LOC"), "300_loc");
/// ```
pub fn canonicalize(s: &str) -> String {
    normalize_constraint(s)
}

/// Normalize a constraint string to canonical form per APEX v1.1
///
/// Algorithm:
/// 1. Trim leading/trailing whitespace
/// 2. Convert to lowercase
/// 3. Replace any sequence of non-alphanumeric characters with "_"
/// 4. Trim leading/trailing underscores
///
/// # Examples
/// ```
/// use apex_spec::sem::normalize_constraint;
/// assert_eq!(normalize_constraint("No Mocks"), "no_mocks");
/// assert_eq!(normalize_constraint("NO_MOCKS"), "no_mocks");
/// assert_eq!(normalize_constraint("real dbs only"), "real_dbs_only");
/// assert_eq!(normalize_constraint("< 300 LOC"), "300_loc");
/// ```
pub fn normalize_constraint(s: &str) -> String {
    let trimmed = s.trim().to_lowercase();

    // Replace any sequence of non-alphanumeric characters with "_"
    let mut result = String::with_capacity(trimmed.len());
    let mut last_was_separator = true; // Start true to skip leading separators

    for c in trimmed.chars() {
        if c.is_ascii_alphanumeric() {
            result.push(c);
            last_was_separator = false;
        } else if !last_was_separator {
            result.push('_');
            last_was_separator = true;
        }
    }

    // Trim trailing underscore
    if result.ends_with('_') {
        result.pop();
    }

    result
}

/// Known constraint types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Constraint {
    /// Only use real databases, no mocks
    RealDbsOnly,
    /// No mock objects allowed
    NoMocks,
    /// Lines of code limit
    LtLoc(u32),
    /// Safe refactoring only (no breaking changes)
    SafeRefactor,
    /// API compatibility required
    ApiCompat,
    /// No stubs allowed
    NoStubs,
    /// Require tests
    RequireTests,
    /// Custom constraint
    Other(String),
}

impl Constraint {
    /// Parse constraint from string using v1.1 normalization
    pub fn from_str(s: &str) -> Self {
        let canonical = normalize_constraint(s);

        // Match known canonical identifiers (v1.1 standard constraints)
        match canonical.as_str() {
            "no_mocks" => return Constraint::NoMocks,
            "real_dbs" | "real_dbs_only" | "real_databases" | "real_databases_only" => {
                return Constraint::RealDbsOnly
            }
            "no_stubs" => return Constraint::NoStubs,
            "safe_refactor" | "safe_refactoring" => return Constraint::SafeRefactor,
            "api_compat" | "api_compatibility" | "api_compatibility_required" => {
                return Constraint::ApiCompat
            }
            "require_tests" | "tests_required" => return Constraint::RequireTests,
            _ => {}
        }

        // Check for LOC limit pattern: "lt300loc", "lt_300_loc", etc.
        if canonical.contains("loc") {
            // Try to extract number
            let digits: String = canonical.chars().filter(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = digits.parse::<u32>() {
                return Constraint::LtLoc(num);
            }
        }

        // Fallback: check original text for fuzzy patterns
        let lower = s.to_lowercase();
        if lower.contains("real") && (lower.contains("db") || lower.contains("database")) {
            return Constraint::RealDbsOnly;
        }
        if lower.contains("no") && lower.contains("mock") {
            return Constraint::NoMocks;
        }
        if lower.contains("no") && lower.contains("stub") {
            return Constraint::NoStubs;
        }
        if lower.contains("safe") && lower.contains("refactor") {
            return Constraint::SafeRefactor;
        }
        if lower.contains("api") && lower.contains("compat") {
            return Constraint::ApiCompat;
        }
        if lower.contains("require") && lower.contains("test") {
            return Constraint::RequireTests;
        }

        Constraint::Other(canonical)
    }

    /// Get canonical string representation
    pub fn as_str(&self) -> String {
        match self {
            Constraint::RealDbsOnly => "real_dbs_only".to_string(),
            Constraint::NoMocks => "no_mocks".to_string(),
            Constraint::LtLoc(n) => format!("lt_{}_loc", n),
            Constraint::SafeRefactor => "safe_refactor".to_string(),
            Constraint::ApiCompat => "api_compat".to_string(),
            Constraint::NoStubs => "no_stubs".to_string(),
            Constraint::RequireTests => "require_tests".to_string(),
            Constraint::Other(s) => s.clone(),
        }
    }
}

/// Semantic analysis of validated document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semantics {
    /// Parsed constraints
    pub constraints: Vec<Constraint>,
    /// Whether execution mode requires PLAN
    pub requires_plan: bool,
    /// Estimated complexity (1-5)
    pub complexity: u8,
}

impl Semantics {
    /// Build semantics from validated document
    pub fn from_validated(doc: &ValidatedDocument) -> Self {
        let constraints = if let Some(ref cv) = doc.constraints {
            cv.rules.iter().map(|r| Constraint::from_str(r)).collect()
        } else {
            Vec::new()
        };

        // Estimate complexity based on plan steps
        let complexity = if let Some(ref plan) = doc.plan {
            match plan.steps.len() {
                0..=2 => 1,
                3..=5 => 2,
                6..=10 => 3,
                11..=20 => 4,
                _ => 5,
            }
        } else {
            1
        };

        // Plan is required if we have complex goals or multiple steps implied
        let requires_plan = doc.goals.as_ref().is_some_and(|g| g.goals.len() > 1);

        Self {
            constraints,
            requires_plan,
            complexity,
        }
    }

    // --- Constraint Queries ---

    /// Check if mocks are forbidden
    pub fn forbids_mocks(&self) -> bool {
        self.constraints.iter().any(|c| matches!(c, Constraint::NoMocks))
    }

    /// Check if stubs are forbidden
    pub fn forbids_stubs(&self) -> bool {
        self.constraints.iter().any(|c| matches!(c, Constraint::NoStubs))
    }

    /// Check if real databases are required
    pub fn requires_real_dbs(&self) -> bool {
        self.constraints.iter().any(|c| matches!(c, Constraint::RealDbsOnly))
    }

    /// Check if tests are required
    pub fn requires_tests(&self) -> bool {
        self.constraints.iter().any(|c| matches!(c, Constraint::RequireTests))
    }

    /// Get LOC limit if specified
    pub fn loc_limit(&self) -> Option<u32> {
        for c in &self.constraints {
            if let Constraint::LtLoc(n) = c {
                return Some(*n);
            }
        }
        None
    }

    /// Check if refactoring must be safe
    pub fn requires_safe_refactor(&self) -> bool {
        self.constraints.iter().any(|c| matches!(c, Constraint::SafeRefactor))
    }

    /// Check if API compatibility is required
    pub fn requires_api_compat(&self) -> bool {
        self.constraints.iter().any(|c| matches!(c, Constraint::ApiCompat))
    }

    /// Get all custom constraints
    pub fn custom_constraints(&self) -> Vec<&str> {
        self.constraints
            .iter()
            .filter_map(|c| {
                if let Constraint::Other(s) = c {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Precedence level for conflict resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    /// Lowest priority
    Context = 1,
    /// Plan steps
    Plan = 2,
    /// Goals override plan
    Goals = 3,
    /// Task overrides goals
    Task = 4,
    /// Constraints override everything
    Constraints = 5,
}

impl Precedence {
    /// Get precedence for block kind
    pub fn for_block(kind: crate::ast::BlockKind) -> Self {
        match kind {
            crate::ast::BlockKind::Constraints => Precedence::Constraints,
            crate::ast::BlockKind::Task => Precedence::Task,
            crate::ast::BlockKind::Goals => Precedence::Goals,
            crate::ast::BlockKind::Plan => Precedence::Plan,
            _ => Precedence::Context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_constraint() {
        // Basic normalization
        assert_eq!(normalize_constraint("No Mocks"), "no_mocks");
        assert_eq!(normalize_constraint("NO_MOCKS"), "no_mocks");
        assert_eq!(normalize_constraint("no-mocks"), "no_mocks");
        assert_eq!(normalize_constraint("  no mocks  "), "no_mocks");

        // Multiple separators collapse to single underscore
        assert_eq!(normalize_constraint("real   dbs   only"), "real_dbs_only");
        assert_eq!(normalize_constraint("real--dbs--only"), "real_dbs_only");

        // Special characters
        assert_eq!(normalize_constraint("< 300 LOC"), "300_loc");
        assert_eq!(normalize_constraint("API compatibility!"), "api_compatibility");

        // Already canonical
        assert_eq!(normalize_constraint("no_mocks"), "no_mocks");
        assert_eq!(normalize_constraint("lt300loc"), "lt300loc");
    }

    #[test]
    fn test_constraint_parsing() {
        // Canonical forms (v1.1)
        assert_eq!(Constraint::from_str("no_mocks"), Constraint::NoMocks);
        assert_eq!(Constraint::from_str("real_dbs"), Constraint::RealDbsOnly);
        assert_eq!(Constraint::from_str("safe_refactor"), Constraint::SafeRefactor);
        assert_eq!(Constraint::from_str("api_compat"), Constraint::ApiCompat);

        // Natural language forms
        assert_eq!(Constraint::from_str("no mocks"), Constraint::NoMocks);
        assert_eq!(Constraint::from_str("NO MOCKS ALLOWED"), Constraint::NoMocks);
        assert_eq!(Constraint::from_str("real databases only"), Constraint::RealDbsOnly);
        assert_eq!(Constraint::from_str("safe refactoring"), Constraint::SafeRefactor);

        // LOC limits
        assert_eq!(Constraint::from_str("< 300 LOC"), Constraint::LtLoc(300));
        assert_eq!(Constraint::from_str("lt300loc"), Constraint::LtLoc(300));
        assert_eq!(Constraint::from_str("lt_500_loc"), Constraint::LtLoc(500));

        // Custom constraints get normalized
        let custom = Constraint::from_str("Custom Rule Here!");
        assert!(matches!(custom, Constraint::Other(s) if s == "custom_rule_here"));
    }

    #[test]
    fn test_precedence_ordering() {
        assert!(Precedence::Constraints > Precedence::Task);
        assert!(Precedence::Task > Precedence::Goals);
        assert!(Precedence::Goals > Precedence::Plan);
        assert!(Precedence::Plan > Precedence::Context);
    }
}
