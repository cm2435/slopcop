mod helpers;
use helpers::lint_with_rule;

#[test]
fn todo_comment() {
    let d = lint_with_rule("# TODO: fix this", "no-todo-comment");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
    assert!(d[0].message.contains("TODO"));
}

#[test]
fn fixme_comment() {
    let d = lint_with_rule("# FIXME: broken", "no-todo-comment");
    assert_eq!(d.len(), 1);
    assert!(d[0].message.contains("FIXME"));
}

#[test]
fn hack_comment() {
    let d = lint_with_rule("# HACK: workaround", "no-todo-comment");
    assert_eq!(d.len(), 1);
    assert!(d[0].message.contains("HACK"));
}

#[test]
fn xxx_comment() {
    let d = lint_with_rule("# XXX: needs attention", "no-todo-comment");
    assert_eq!(d.len(), 1);
    assert!(d[0].message.contains("XXX"));
}

#[test]
fn todo_no_colon() {
    let d = lint_with_rule("# TODO fix this", "no-todo-comment");
    assert_eq!(d.len(), 1);
}

#[test]
fn todo_lowercase_ignored() {
    let d = lint_with_rule("# todo: not caught", "no-todo-comment");
    assert_eq!(d.len(), 0);
}

#[test]
fn todo_in_word_ignored() {
    let d = lint_with_rule("# TODOLIST: check items", "no-todo-comment");
    assert_eq!(d.len(), 0);
}

#[test]
fn todo_suffix_in_word_ignored() {
    let d = lint_with_rule("# MYTODO: check items", "no-todo-comment");
    assert_eq!(d.len(), 0);
}

#[test]
fn normal_comment_ok() {
    let d = lint_with_rule("# This function computes the sum", "no-todo-comment");
    assert_eq!(d.len(), 0);
}

#[test]
fn code_no_comment_ok() {
    let d = lint_with_rule("x = 1", "no-todo-comment");
    assert_eq!(d.len(), 0);
}

#[test]
fn todo_in_string_ok() {
    let d = lint_with_rule("x = \"# TODO: not a comment\"", "no-todo-comment");
    assert_eq!(d.len(), 0);
}

#[test]
fn inline_todo_comment() {
    let d = lint_with_rule("x = 1  # TODO: fix later", "no-todo-comment");
    assert_eq!(d.len(), 1);
}

#[test]
fn todo_at_end_of_comment() {
    let d = lint_with_rule("# needs a TODO", "no-todo-comment");
    assert_eq!(d.len(), 1);
}

#[test]
fn multiple_todo_comments() {
    let source = "# TODO: first\nx = 1\n# FIXME: second";
    let d = lint_with_rule(source, "no-todo-comment");
    assert_eq!(d.len(), 2);
    assert_eq!(d[0].line, 1);
    assert_eq!(d[1].line, 3);
}

#[test]
fn todo_inside_function() {
    let source = "def f():\n    # TODO: implement";
    let d = lint_with_rule(source, "no-todo-comment");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn todo_with_parens() {
    let d = lint_with_rule("# TODO(alice): fix this", "no-todo-comment");
    assert_eq!(d.len(), 1);
}

#[test]
fn fixme_in_multiline_string_ok() {
    let d = lint_with_rule("x = '''FIXME: not a comment'''", "no-todo-comment");
    assert_eq!(d.len(), 0);
}

#[test]
fn only_one_diagnostic_per_comment() {
    let d = lint_with_rule("# TODO FIXME HACK XXX all at once", "no-todo-comment");
    assert_eq!(d.len(), 1, "should emit one diagnostic per comment, not per marker");
}
