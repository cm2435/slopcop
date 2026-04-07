use crate::diagnostic::Diagnostic;
use crate::rules::Rule;

const MARKERS: &[&str] = &["TODO", "FIXME", "HACK", "XXX"];

pub struct NoTodoComment;

impl Rule for NoTodoComment {
    fn name(&self) -> &'static str {
        "no-todo-comment"
    }

    fn node_kinds(&self) -> &'static [&'static str] {
        &["comment"]
    }

    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        _ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let text = node.utf8_text(source).unwrap_or("");

        // Strip the leading `#` and optional space so we match markers anywhere
        // in the comment body, but not inside identifier-like words
        // (e.g. "TODOLIST" should not trigger).
        let body = text.strip_prefix('#').unwrap_or(text);

        for &marker in MARKERS {
            if let Some(offset) = body.find(marker) {
                let after = offset + marker.len();
                let before = offset.wrapping_sub(1);

                let char_after = body.as_bytes().get(after).copied();
                let char_before = body.as_bytes().get(before).copied();

                if is_word_boundary(char_before) && is_word_boundary(char_after) {
                    let pos = node.start_position();
                    diagnostics.push(Diagnostic {
                        path: String::new(),
                        line: pos.row + 1,
                        col: pos.column,
                        rule_id: "no-todo-comment",
                        message: format!(
                            "Found `{marker}` comment; resolve or track in an issue"
                        ),
                    });
                    return;
                }
            }
        }
    }
}

fn is_word_boundary(byte: Option<u8>) -> bool {
    match byte {
        None => true,
        Some(b) => !b.is_ascii_alphanumeric() && b != b'_',
    }
}
