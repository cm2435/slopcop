mod helpers;
use helpers::lint_with_rule;

#[test]
fn str_empty_double_quotes() {
    let source = "def f(x: str = \"\"):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 1);
}

#[test]
fn str_empty_single_quotes() {
    let source = "def f(x: str = ''):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
}

#[test]
fn str_non_empty_ok() {
    let source = "def f(x: str = \"hello\"):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn str_none_default_ok() {
    let source = "def f(x: str | None = None):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn optional_str_empty() {
    let source = "def f(x: str | None = \"\"):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
}

#[test]
fn int_default_ok() {
    let source = "def f(x: int = 0):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn untyped_empty_string_ok() {
    let source = "def f(x=\"\"):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn multiple_params_one_bad() {
    let source = "def f(a: int, b: str = \"\", c: str = \"ok\"):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
}

#[test]
fn multiple_params_both_bad() {
    let source = "def f(a: str = \"\", b: str = ''):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 2);
}

#[test]
fn method_parameter() {
    let source = "class C:\n    def m(self, x: str = \"\"):\n        pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
}

#[test]
fn async_function() {
    let source = "async def f(x: str = \"\"):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
}

#[test]
fn in_comment_ok() {
    let d = lint_with_rule("# def f(x: str = \"\"):", "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn in_string_ok() {
    let d = lint_with_rule("x = 'def f(x: str = \"\")'", "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn empty_triple_quoted() {
    let source = "def f(x: str = \"\"\"\"\"\"):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
}

#[test]
fn list_str_type_not_flagged() {
    let source = "def f(x: list[str] = []):\n    pass";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

// -- Class field (model field) detection --

#[test]
fn class_field_str_empty() {
    let source = "class C(BaseModel):\n    name: str = \"\"";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].line, 2);
}

#[test]
fn class_field_str_non_empty_ok() {
    let source = "class C(BaseModel):\n    name: str = \"hello\"";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn class_field_str_no_default_ok() {
    let source = "class C(BaseModel):\n    name: str";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn class_field_int_default_ok() {
    let source = "class C(BaseModel):\n    count: int = 0";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}

#[test]
fn class_field_multiple_one_bad() {
    let source = "class C(BaseModel):\n    name: str = \"\"\n    title: str = \"ok\"\n    desc: str = ''";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 2);
}

#[test]
fn class_field_optional_str_empty() {
    let source = "class C(BaseModel):\n    name: str | None = \"\"";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 1);
}

#[test]
fn module_level_assignment_not_flagged() {
    // str = "" at module level (not in a class) should NOT be flagged
    let source = "name: str = \"\"";
    let d = lint_with_rule(source, "no-str-empty-default");
    assert_eq!(d.len(), 0);
}
