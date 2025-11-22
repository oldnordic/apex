//! APEX Interpreter
//!
//! Converts validated documents into executable plans.
//!
//! ## v1.1 Execution State Model
//!
//! Per APEX v1.1, execution state is stored out-of-band (not in APEX syntax).
//! This module provides types for tracking step status and checkpointing.

use crate::errors::ApexResult;
use crate::validate::{ValidatedDocument, ToolDeclaration};
use serde::{Deserialize, Serialize};

// ============================================================
// v1.1 Execution State Model
// ============================================================

/// Status of a single execution step (v1.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StepStatus {
    /// Step has not started
    #[default]
    Pending,
    /// Step is currently executing
    Running,
    /// Step completed successfully
    Complete,
    /// Step failed with an error
    Failed,
    /// Step was skipped (due to earlier failure or constraint)
    Skipped,
}

impl StepStatus {
    /// Check if step is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, StepStatus::Complete | StepStatus::Failed | StepStatus::Skipped)
    }

    /// Check if step can be resumed
    pub fn can_resume(&self) -> bool {
        matches!(self, StepStatus::Pending | StepStatus::Failed)
    }
}

/// Execution state for tracking plan progress (v1.1)
///
/// This is stored out-of-band, not in APEX syntax.
/// Runtimes use this to checkpoint and resume execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    /// Status of each step (indexed by step_number - 1)
    pub step_states: Vec<StepStatus>,
    /// Index of last completed step (0 = none completed)
    pub checkpoint: usize,
    /// Tool results for completed steps
    pub tool_results: Vec<Option<String>>,
    /// Validation outcomes
    pub validation_outcomes: Vec<bool>,
    /// Whether execution is paused
    pub paused: bool,
    /// Error message if execution failed
    pub error: Option<String>,
}

impl ExecutionState {
    /// Create initial state for a plan with n steps
    pub fn new(num_steps: usize) -> Self {
        Self {
            step_states: vec![StepStatus::Pending; num_steps],
            checkpoint: 0,
            tool_results: vec![None; num_steps],
            validation_outcomes: Vec::new(),
            paused: false,
            error: None,
        }
    }

    /// Get current step index (0-based)
    pub fn current_step(&self) -> usize {
        self.checkpoint
    }

    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        self.step_states.iter().all(|s| s.is_terminal())
    }

    /// Check if execution failed
    pub fn is_failed(&self) -> bool {
        self.step_states.iter().any(|s| matches!(s, StepStatus::Failed))
    }

    /// Mark a step as running
    pub fn start_step(&mut self, step: usize) {
        if step < self.step_states.len() {
            self.step_states[step] = StepStatus::Running;
        }
    }

    /// Mark a step as complete with optional result
    pub fn complete_step(&mut self, step: usize, result: Option<String>) {
        if step < self.step_states.len() {
            self.step_states[step] = StepStatus::Complete;
            self.tool_results[step] = result;
            self.checkpoint = step + 1;
        }
    }

    /// Mark a step as failed with error
    pub fn fail_step(&mut self, step: usize, error: String) {
        if step < self.step_states.len() {
            self.step_states[step] = StepStatus::Failed;
            self.error = Some(error);
        }
    }

    /// Skip a step
    pub fn skip_step(&mut self, step: usize) {
        if step < self.step_states.len() {
            self.step_states[step] = StepStatus::Skipped;
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Tool invocation in execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    /// Tool name
    pub name: String,
    /// Raw arguments (unparsed)
    pub raw_arguments: Option<String>,
    /// Parsed arguments as JSON (optional)
    pub arguments: Option<serde_json::Value>,
}

impl ToolInvocation {
    /// Create from tool declaration
    pub fn from_declaration(decl: &ToolDeclaration) -> Self {
        Self {
            name: decl.name.clone(),
            raw_arguments: decl.arguments.clone(),
            arguments: None,
        }
    }
}

/// Single execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step number (1-indexed)
    pub step_number: usize,
    /// Step description from PLAN
    pub description: String,
    /// Associated tool invocation (if any)
    pub tool: Option<ToolInvocation>,
    /// Dependencies (step numbers that must complete first)
    pub depends_on: Vec<usize>,
}

impl ExecutionStep {
    /// Create a new step
    pub fn new(step_number: usize, description: String) -> Self {
        Self {
            step_number,
            description,
            tool: None,
            depends_on: Vec::new(),
        }
    }

    /// Add tool invocation
    pub fn with_tool(mut self, tool: ToolInvocation) -> Self {
        self.tool = Some(tool);
        self
    }

    /// Add dependency
    pub fn depends_on(mut self, step: usize) -> Self {
        self.depends_on.push(step);
        self
    }
}

/// Complete execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Task description
    pub task: String,
    /// Goals to achieve
    pub goals: Vec<String>,
    /// Constraints to respect
    pub constraints: Vec<String>,
    /// Ordered execution steps
    pub steps: Vec<ExecutionStep>,
    /// Validation conditions to check after execution
    pub validation: Vec<String>,
    /// Available tools
    pub available_tools: Vec<ToolInvocation>,
}

impl ExecutionPlan {
    /// Check if plan is empty (no steps)
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get total step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get steps that have no dependencies (can start immediately)
    pub fn initial_steps(&self) -> Vec<&ExecutionStep> {
        self.steps.iter().filter(|s| s.depends_on.is_empty()).collect()
    }

    /// Get steps that depend on a given step
    pub fn dependents(&self, step_number: usize) -> Vec<&ExecutionStep> {
        self.steps
            .iter()
            .filter(|s| s.depends_on.contains(&step_number))
            .collect()
    }
}

/// Build execution plan from validated document
pub fn build_execution_plan(doc: &ValidatedDocument) -> ApexResult<ExecutionPlan> {
    let task = doc.task.line.clone();

    let goals = doc
        .goals
        .as_ref()
        .map(|g| g.goals.clone())
        .unwrap_or_default();

    let constraints = doc
        .constraints
        .as_ref()
        .map(|c| c.rules.clone())
        .unwrap_or_default();

    let validation = doc
        .validation
        .as_ref()
        .map(|v| v.conditions.clone())
        .unwrap_or_default();

    // Parse available tools
    let available_tools: Vec<ToolInvocation> = doc
        .tools
        .as_ref()
        .map(|t| t.tools.iter().map(ToolInvocation::from_declaration).collect())
        .unwrap_or_default();

    // Build steps from PLAN
    let steps = build_steps(doc, &available_tools)?;

    Ok(ExecutionPlan {
        task,
        goals,
        constraints,
        steps,
        validation,
        available_tools,
    })
}

/// Build execution steps from plan and match with tools
fn build_steps(doc: &ValidatedDocument, tools: &[ToolInvocation]) -> ApexResult<Vec<ExecutionStep>> {
    let mut steps = Vec::new();

    if let Some(ref plan) = doc.plan {
        for (i, step_desc) in plan.steps.iter().enumerate() {
            let step_number = i + 1;
            let mut step = ExecutionStep::new(step_number, step_desc.clone());

            // Try to match tool to step
            // Strategy 1: 1:1 index matching if tools count == steps count
            if tools.len() == plan.steps.len() {
                step.tool = Some(tools[i].clone());
            } else {
                // Strategy 2: Heuristic matching by keyword
                step.tool = match_tool_to_step(step_desc, tools);
            }

            // Simple sequential dependencies (each step depends on previous)
            if step_number > 1 {
                step.depends_on.push(step_number - 1);
            }

            steps.push(step);
        }
    }

    Ok(steps)
}

/// Heuristic tool matching based on step description keywords
fn match_tool_to_step(step_desc: &str, tools: &[ToolInvocation]) -> Option<ToolInvocation> {
    let lower = step_desc.to_lowercase();

    for tool in tools {
        let tool_name_lower = tool.name.to_lowercase();

        // Check if step mentions tool name
        if lower.contains(&tool_name_lower) {
            return Some(tool.clone());
        }

        // Check common patterns
        if lower.contains("read") && tool_name_lower.contains("read") {
            return Some(tool.clone());
        }
        if lower.contains("write") && tool_name_lower.contains("write") {
            return Some(tool.clone());
        }
        if lower.contains("search") && tool_name_lower.contains("search") {
            return Some(tool.clone());
        }
        if lower.contains("edit") && tool_name_lower.contains("edit") {
            return Some(tool.clone());
        }
    }

    None
}

/// Configuration for plan building
#[derive(Debug, Clone)]
pub struct InterpreterConfig {
    /// Allow empty plans (TASK without PLAN)
    pub allow_empty_plan: bool,
    /// Strict tool matching (error if tool not found for step)
    pub strict_tool_matching: bool,
    /// Infer sequential dependencies
    pub infer_dependencies: bool,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            allow_empty_plan: true,
            strict_tool_matching: false,
            infer_dependencies: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_str;
    use crate::validate::validate;

    fn parse_and_validate(input: &str) -> ValidatedDocument {
        let doc = parse_str(input).unwrap();
        validate(doc).unwrap()
    }

    #[test]
    fn test_minimal_plan() {
        let validated = parse_and_validate("TASK\nDo the thing");
        let plan = build_execution_plan(&validated).unwrap();

        assert_eq!(plan.task, "Do the thing");
        assert!(plan.steps.is_empty());
        assert!(plan.goals.is_empty());
    }

    #[test]
    fn test_full_plan() {
        let input = r#"TASK
Implement feature X

GOALS
Feature works
Tests pass

PLAN
Step 1: Read requirements
Step 2: Write code
Step 3: Run tests

CONSTRAINTS
No breaking changes

VALIDATION
All tests pass
"#;
        let validated = parse_and_validate(input);
        let plan = build_execution_plan(&validated).unwrap();

        assert_eq!(plan.task, "Implement feature X");
        assert_eq!(plan.goals.len(), 2);
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.constraints.len(), 1);
        assert_eq!(plan.validation.len(), 1);

        // Check sequential dependencies
        assert!(plan.steps[0].depends_on.is_empty());
        assert_eq!(plan.steps[1].depends_on, vec![1]);
        assert_eq!(plan.steps[2].depends_on, vec![2]);
    }

    #[test]
    fn test_tool_matching_1_to_1() {
        let input = r#"TASK
Do something

PLAN
Read the file
Write the output

TOOLS
read_file(path)
write_file(path, content)
"#;
        let validated = parse_and_validate(input);
        let plan = build_execution_plan(&validated).unwrap();

        assert_eq!(plan.steps.len(), 2);
        assert!(plan.steps[0].tool.is_some());
        assert_eq!(plan.steps[0].tool.as_ref().unwrap().name, "read_file");
        assert!(plan.steps[1].tool.is_some());
        assert_eq!(plan.steps[1].tool.as_ref().unwrap().name, "write_file");
    }

    #[test]
    fn test_tool_matching_heuristic() {
        let input = r#"TASK
Analyze code

PLAN
Search for function definitions
Read the main file
Edit the config

TOOLS
grep_search(pattern)
read_file(path)
edit_file(path, changes)
extra_tool()
"#;
        let validated = parse_and_validate(input);
        let plan = build_execution_plan(&validated).unwrap();

        // Heuristic matching should work
        assert!(plan.steps[0].tool.is_some()); // "search" -> grep_search
        assert!(plan.steps[1].tool.is_some()); // "read" -> read_file
        assert!(plan.steps[2].tool.is_some()); // "edit" -> edit_file
    }
}
