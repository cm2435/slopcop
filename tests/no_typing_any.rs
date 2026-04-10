mod helpers;
use helpers::lint_with_rule;

// -- Usage in type annotations --

#[test]
fn param_typed_any() {
    let d = lint_with_rule("def f(x: Any):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn param_typed_any_with_default() {
    let d = lint_with_rule("def f(x: Any = None):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn return_type_any() {
    let d = lint_with_rule("def f() -> Any:\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn variable_annotation_any() {
    let d = lint_with_rule("x: Any = 5", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn generic_any() {
    let d = lint_with_rule("def f(x: dict[str, Any]):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

#[test]
fn union_any() {
    let d = lint_with_rule("def f(x: str | Any):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 1);
}

// -- False positive avoidance --

#[test]
fn import_not_flagged() {
    // Import alone is not flagged -- only annotation usage
    let d = lint_with_rule("from typing import Any", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn variable_named_any_ok() {
    let d = lint_with_rule("Any = 42\nprint(Any)", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn string_containing_any_ok() {
    let d = lint_with_rule("x = \"Any\"", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn comment_containing_any_ok() {
    let d = lint_with_rule("# x: Any", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn function_named_any_ok() {
    let d = lint_with_rule("def any_handler():\n    pass", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn builtin_any_call_ok() {
    let d = lint_with_rule("result = any(items)", "no-typing-any");
    assert_eq!(d.len(), 0);
}

// -- *args and **kwargs: Any is idiomatic and should not be flagged --

#[test]
fn star_args_any_not_flagged() {
    let d = lint_with_rule("def f(*args: Any):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn double_star_kwargs_any_not_flagged() {
    let d = lint_with_rule("def f(**kwargs: Any):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn star_args_and_kwargs_any_not_flagged() {
    let d = lint_with_rule("def f(*args: Any, **kwargs: Any):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn mixed_params_only_regular_flagged() {
    let d = lint_with_rule("def f(x: Any, *args: Any, **kwargs: Any):\n    pass", "no-typing-any");
    // Only x: Any should be flagged, *args and **kwargs are exempt
    assert_eq!(d.len(), 1);
}

#[test]
fn star_args_complex_type_not_flagged() {
    let d = lint_with_rule("def f(*args: Any, y: int):\n    pass", "no-typing-any");
    assert_eq!(d.len(), 0);
}

#[test]
fn kwargs_with_return_any_flagged() {
    let d = lint_with_rule("def f(**kwargs: Any) -> Any:\n    pass", "no-typing-any");
    // **kwargs: Any is exempt, but -> Any is still flagged
    assert_eq!(d.len(), 1);
}

// -- Combined import + usage --

#[test]
fn import_and_usage_only_usage_flagged() {
    let source = "from typing import Any\n\ndef f(x: Any) -> Any:\n    pass";
    let d = lint_with_rule(source, "no-typing-any");
    // Only 2 usage sites (param + return), import is not flagged
    assert_eq!(d.len(), 2);
}
