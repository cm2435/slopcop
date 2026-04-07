use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

pub struct NoStrEmptyDefault;

impl Rule for NoStrEmptyDefault {
    fn name(&self) -> &'static str {
        "no-str-empty-default"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["typed_default_parameter"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Some(type_node) = node.child_by_field_name("type") else {
            return;
        };
        if !type_contains_str(&type_node, source) {
            return;
        }

        let Some(value_node) = node.child_by_field_name("value") else {
            return;
        };
        if !is_empty_string(&value_node) {
            return;
        }

        diagnostics.push(Diagnostic {
            path: String::new(),
            line: node.start_position().row + 1,
            col: node.start_position().column,
            rule_id: "no-str-empty-default",
            message: "Avoid `str = \"\"`; use `str | None = None` or make the parameter required"
                .to_string(),
        });
    }
}

/// Check if a type annotation is or directly contains `str`.
/// Matches: `str`, `str | None`, `None | str`, `str | int` etc.
/// Does NOT match: `list[str]`, `Optional[str]` (subscript forms where
/// `str` is a type argument, not the top-level type).
fn type_contains_str(type_node: &tree_sitter::Node, source: &[u8]) -> bool {
    // The `type` field wraps the actual type expression
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
            // str | None, None | str, str | int, etc.
            if let Some(left) = node.child_by_field_name("left") {
                if check_type_expr(&left, source) {
                    return true;
                }
            }
            if let Some(right) = node.child_by_field_name("right") {
                if check_type_expr(&right, source) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// An empty string has kind "string" with string_start + string_end
/// but no string_content child.
fn is_empty_string(node: &tree_sitter::Node) -> bool {
    if node.kind() != "string" {
        return false;
    }
    for i in 0..node.child_count() {
        if node.child(i).unwrap().kind() == "string_content" {
            return false;
        }
    }
    true
}
