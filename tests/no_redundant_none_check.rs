mod helpers;
use helpers::lint_with_rule;

// -- Should flag: non-optional param checked against None --

#[test]
fn is_none_on_str_param() {
    let source = "def f(x: str):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn is_not_none_on_str_param() {
    let source = "def f(x: str):\n    if x is not None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 1);
}

#[test]
fn is_none_on_int_param() {
    let source = "def f(count: int):\n    if count is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 1);
}

#[test]
fn is_none_on_uuid_param() {
    let source = "def f(task_id: uuid.UUID):\n    if task_id is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 1);
}

#[test]
fn is_none_on_typed_default_param() {
    let source = "def f(x: str = \"hello\"):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 1);
}

// -- Should NOT flag: optional / nullable params --

#[test]
fn optional_union_none() {
    let source = "def f(x: str | None):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn optional_none_first() {
    let source = "def f(x: None | str):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn optional_typing() {
    let source = "def f(x: Optional[str]):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn none_default_value() {
    let source = "def f(x: str | None = None):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

// -- Should NOT flag: untyped or non-param --

#[test]
fn untyped_param() {
    let source = "def f(x):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn local_variable() {
    let source = "def f():\n    x = get_value()\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn not_in_function() {
    let source = "if x is None:\n    pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

// -- Should NOT flag: Any type --

#[test]
fn any_type() {
    let source = "def f(x: Any):\n    if x is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

// -- Edge cases --

#[test]
fn comparison_not_with_none() {
    let source = "def f(x: str):\n    if x is True:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn equality_none_not_flagged() {
    // `== None` is a different smell (use `is None`), not our concern
    let source = "def f(x: str):\n    if x == None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_comment_ok() {
    let d = lint_with_rule("# if x is None:", "no-redundant-none-check");
    assert_eq!(d.len(), 0);
}

#[test]
fn method_param() {
    let source = "class C:\n    def m(self, x: str):\n        if x is None:\n            pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 1);
}

#[test]
fn multiple_params_one_flagged() {
    let source = "def f(a: str, b: str | None):\n    if a is None:\n        pass\n    if b is None:\n        pass";
    let d = lint_with_rule(source, "no-redundant-none-check");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}
