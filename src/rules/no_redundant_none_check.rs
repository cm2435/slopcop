use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoRedundantNoneCheck;

impl Rule for NoRedundantNoneCheck {
    fn name(&self) -> &'static str {
        "no-redundant-none-check"
    }

    fn severity(&self) -> Severity { Severity::Warning }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["comparison_operator"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(ident_name) = extract_none_comparison_ident(node, source) else {
            return;
        };

        let Some(func_node) = ancestors.iter().rev().find(|a| a.kind() == "function_definition") else {
            return;
        };

        // Try params first, then local annotated variables
        let type_text = func_node
            .child_by_field_name("parameters")
            .and_then(|params| find_param_type(&params, ident_name, source))
            .or_else(|| {
                func_node
                    .child_by_field_name("body")
                    .and_then(|body| find_local_annotated_type(&body, ident_name, node, source))
            });

        let Some(type_text) = type_text else { return };

        if type_includes_none(type_text) || type_text == "Any" {
            return;
        }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-redundant-none-check",
            severity: crate::rules::Severity::Error,
            message: format!(
                "Redundant `None` check on `{ident_name}` which is typed as `{type_text}` (not optional)"
            ),
        });
    }
}

/// Extract the identifier name from `X is None` or `X is not None` patterns.
/// Returns None if the comparison isn't an identity check against None.
fn extract_none_comparison_ident<'a>(
    node: &tree_sitter::Node,
    source: &'a [u8],
) -> Option<&'a str> {
    let child_count = node.child_count();
    if child_count != 3 {
        return None;
    }

    // tree-sitter-python CST:
    //   `x is None`     → identifier, "is",     none     (3 children)
    //   `x is not None` → identifier, "is not", none     (3 children, "is not" is compound)
    let first = node.child(0)?;
    let operator = node.child(1)?;
    let last = node.child(2)?;

    let op_text = operator.utf8_text(source).ok()?;
    if op_text != "is" && op_text != "is not" {
        return None;
    }

    if last.kind() != "none" {
        return None;
    }

    if first.kind() != "identifier" {
        return None;
    }

    first.utf8_text(source).ok()
}

/// Find the type annotation text for a parameter by name.
/// Handles both `typed_parameter` (no field names in grammar) and
/// `typed_default_parameter` (has field names).
fn find_param_type<'a>(
    params: &tree_sitter::Node,
    name: &str,
    source: &'a [u8],
) -> Option<&'a str> {
    for i in 0..params.child_count() {
        let Some(child) = params.child(i) else { continue };
        match child.kind() {
            "typed_default_parameter" => {
                // Has field names: name, type, value
                let Some(pname) = child.child_by_field_name("name") else { continue };
                if pname.utf8_text(source).unwrap_or("") != name { continue; }
                let Some(tnode) = child.child_by_field_name("type") else { continue };
                return tnode.utf8_text(source).ok();
            }
            "typed_parameter" => {
                // No field names -- find identifier and type children by kind
                let mut param_name = None;
                let mut type_text = None;
                for j in 0..child.child_count() {
                    let Some(gc) = child.child(j) else { continue };
                    match gc.kind() {
                        "identifier" if param_name.is_none() => {
                            param_name = gc.utf8_text(source).ok();
                        }
                        "type" => {
                            type_text = gc.utf8_text(source).ok();
                        }
                        _ => {}
                    }
                }
                if param_name == Some(name) {
                    return type_text;
                }
            }
            _ => {}
        }
    }
    None
}

/// Find a type annotation for a locally-annotated variable in the function body.
/// Scans `assignment` nodes like `x: str = value` that appear BEFORE the comparison node.
fn find_local_annotated_type<'a>(
    body: &tree_sitter::Node,
    name: &str,
    comparison_node: &tree_sitter::Node,
    source: &'a [u8],
) -> Option<&'a str> {
    let comparison_row = comparison_node.start_position().row;

    for i in 0..body.child_count() {
        let Some(stmt) = body.child(i) else { continue };

        // Only look at statements before the comparison
        if stmt.start_position().row >= comparison_row {
            break;
        }

        // Drill into expression_statement → assignment
        let assignment = if stmt.kind() == "expression_statement" {
            stmt.child(0)
        } else {
            None
        };
        let Some(assignment) = assignment else { continue };
        if assignment.kind() != "assignment" { continue; }

        // Look for: identifier, ":", type, "=", value
        let mut found_name = None;
        let mut found_type = None;
        for j in 0..assignment.child_count() {
            let Some(child) = assignment.child(j) else { continue };
            match child.kind() {
                "identifier" if found_name.is_none() => {
                    found_name = child.utf8_text(source).ok();
                }
                "type" => {
                    found_type = child.utf8_text(source).ok();
                }
                _ => {}
            }
        }

        if found_name == Some(name) {
            return found_type;
        }
    }
    None
}

/// Check if a type annotation text includes None or Optional.
fn type_includes_none(type_text: &str) -> bool {
    // Check for `None` as a standalone token in union types
    // or `Optional` as a wrapper
    for token in type_text.split(['|', '[', ']', ',', ' ']) {
        let trimmed = token.trim();
        if trimmed == "None" || trimmed == "Optional" {
            return true;
        }
    }
    false
}
