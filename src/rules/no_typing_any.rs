use crate::diagnostic::Diagnostic;
use crate::rules::{Rule, Severity};

const ESCAPE_HATCHES: &[&str] = &["Any", "object"];

pub struct NoTypingAny;

impl Rule for NoTypingAny {
    fn name(&self) -> &'static str {
        "no-typing-any"
    }

    fn severity(&self) -> Severity { Severity::Warning }

    fn help(&self) -> &'static str {
        "`Any` and bare `object` disable meaningful type checking. Use a \
         specific type, a `Protocol`, `TypeVar`, or a union instead. For \
         `*args` / `**kwargs` this rule is auto-skipped. If the type is truly \
         unknowable (e.g. untyped third-party API), suppress with \
         `# slopcop: ignore[no-typing-any]`."
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["type"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if is_variadic_param_annotation(ancestors) {
            return;
        }
        find_escape_hatch_identifiers(node, source, diagnostics);
    }
}

/// Returns true if this type node annotates a `*args` or `**kwargs` parameter,
/// where `Any` is idiomatic and unavoidable in Python's type system.
fn is_variadic_param_annotation(ancestors: &[tree_sitter::Node]) -> bool {
    let parent = match ancestors.last() {
        Some(p) => p,
        None => return false,
    };
    if parent.kind() != "typed_parameter" {
        return false;
    }
    for i in 0..parent.child_count() {
        let child = parent.child(i).unwrap();
        let kind = child.kind();
        if kind == "list_splat_pattern" || kind == "dictionary_splat_pattern" {
            return true;
        }
    }
    false
}

/// Recursively find `identifier` nodes with escape-hatch type names (`Any`, `object`)
/// within a type annotation. Skips child `type` nodes since the engine dispatches
/// those independently.
fn find_escape_hatch_identifiers(
    node: &tree_sitter::Node,
    source: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
) {
    if node.kind() == "identifier" {
        let text = node.utf8_text(source).unwrap_or("");
        if ESCAPE_HATCHES.contains(&text) {
            let message = if text == "Any" {
                "Avoid `Any` in type annotations; use specific types or protocols".to_string()
            } else {
                format!(
                    "Avoid bare `{text}` in type annotations; use a specific type, Protocol, or TypeVar"
                )
            };
            diagnostics.push(Diagnostic {
                path: String::new(),
                line: node.start_position().row + 1,
                col: node.start_position().column,
                rule_id: "no-typing-any",
                severity: crate::rules::Severity::Error,
                message,
            });
            return;
        }
    }

    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        if child.kind() == "type" {
            continue;
        }
        find_escape_hatch_identifiers(&child, source, diagnostics);
    }
}
