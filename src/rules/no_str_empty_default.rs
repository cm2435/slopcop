use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

pub struct NoStrEmptyDefault;

impl Rule for NoStrEmptyDefault {
    fn name(&self) -> &'static str {
        "no-str-empty-default"
    }

    fn severity(&self) -> Severity { Severity::Warning }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["typed_default_parameter", "assignment"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match node.kind() {
            "typed_default_parameter" => self.check_param(node, source, diagnostics),
            "assignment" => self.check_assignment(node, source, ancestors, diagnostics),
            _ => {}
        }
    }
}

impl NoStrEmptyDefault {
    fn check_param(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(type_node) = node.child_by_field_name("type") else { return };
        if !type_contains_str(&type_node, source) { return; }
        let Some(value_node) = node.child_by_field_name("value") else { return };
        if !is_empty_string(&value_node) { return; }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-str-empty-default",
            severity: crate::rules::Severity::Error,
            message: "Avoid `str = \"\"`; use `str | None = None` or make the parameter required"
                .to_string(),
        });
    }

    fn check_assignment(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Only flag annotated assignments inside class bodies (model fields)
        let in_class = ancestors.iter().any(|a| a.kind() == "class_definition");
        if !in_class { return; }

        // Find the type and value children by kind (assignment has no field names for these)
        let mut type_node = None;
        let mut value_node = None;
        let mut found_eq = false;

        for i in 0..node.child_count() {
            let Some(child) = node.child(i) else { continue };
            match child.kind() {
                "type" if type_node.is_none() => { type_node = Some(child); }
                "=" => { found_eq = true; }
                "string" if found_eq && value_node.is_none() => { value_node = Some(child); }
                _ => {}
            }
        }

        let Some(type_node) = type_node else { return };
        if !type_contains_str(&type_node, source) { return; }
        let Some(value_node) = value_node else { return };
        if !is_empty_string(&value_node) { return; }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-str-empty-default",
            severity: crate::rules::Severity::Error,
            message: "Avoid `str = \"\"` field default; use `str | None = None` or make the field required"
                .to_string(),
        });
    }
}

/// Check if a type annotation is or directly contains `str`.
fn type_contains_str(type_node: &tree_sitter::Node, source: &[u8]) -> bool {
    for i in 0..type_node.child_count() {
        let child = type_node.child(i).unwrap();
        if check_type_expr(&child, source) {
            return true;
        }
    }
    false
}

fn check_type_expr(node: &tree_sitter::Node, source: &[u8]) -> bool {
    match node.kind() {
        "identifier" => node.utf8_text(source).unwrap_or("") == "str",
        "binary_operator" => {
            if let Some(left) = node.child_by_field_name("left") {
                if check_type_expr(&left, source) { return true; }
            }
            if let Some(right) = node.child_by_field_name("right") {
                if check_type_expr(&right, source) { return true; }
            }
            false
        }
        _ => false,
    }
}

/// An empty string has kind "string" with string_start + string_end
/// but no string_content child.
fn is_empty_string(node: &tree_sitter::Node) -> bool {
    if node.kind() != "string" { return false; }
    for i in 0..node.child_count() {
        if node.child(i).unwrap().kind() == "string_content" { return false; }
    }
    true
}
