mod helpers;
use helpers::count_rule;
use slopcop::{lint_source_with_config, Config};

#[test]
fn default_all_active() {
    let source = "hasattr(obj, \"x\")";
    let d = lint_source_with_config(source, "<test>", &Config::default());
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn exclude_rule() {
    let source = "hasattr(obj, \"x\")";
    let config = Config { exclude: vec!["no-hasattr-getattr".to_string()], ..Config::default() };
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
}

#[test]
fn exclude_unrelated_rule() {
    let source = "hasattr(obj, \"x\")";
    let config = Config { exclude: vec!["no-print".to_string()], ..Config::default() };
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-hasattr-getattr") >= 1);
}

#[test]
fn exclude_print() {
    let source = "print(\"hi\")";
    let config = Config { exclude: vec!["no-print".to_string()], ..Config::default() };
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "no-print"), 0);
}

#[test]
fn empty_exclude() {
    let source = "print(\"hi\")";
    let config = Config { exclude: vec![], ..Config::default() };
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-print") >= 1);
}

// -- Per-file ignores --

#[test]
fn per_file_ignore_matching_glob() {
    let source = "assert x > 0";
    let mut config = Config::default();
    config.per_file_ignores.insert("tests/**".to_string(), vec!["no-assert".to_string()]);
    let d = lint_source_with_config(source, "tests/test_foo.py", &config);
    assert_eq!(count_rule(&d, "no-assert"), 0);
}

#[test]
fn per_file_ignore_non_matching_glob() {
    let source = "assert x > 0";
    let mut config = Config::default();
    config.per_file_ignores.insert("tests/**".to_string(), vec!["no-assert".to_string()]);
    let d = lint_source_with_config(source, "src/foo.py", &config);
    assert!(count_rule(&d, "no-assert") >= 1);
}

#[test]
fn per_file_ignore_union_with_global() {
    let source = "hasattr(obj, \"x\")\nprint(\"hi\")";
    let mut config = Config::default();
    config.exclude = vec!["no-hasattr-getattr".to_string()];
    config.per_file_ignores.insert("scripts/**".to_string(), vec!["no-print".to_string()]);
    let d = lint_source_with_config(source, "scripts/run.py", &config);
    assert_eq!(count_rule(&d, "no-hasattr-getattr"), 0);
    assert_eq!(count_rule(&d, "no-print"), 0);
}

#[test]
fn per_file_ignore_double_star_middle() {
    let source = "print(\"hi\")";
    let mut config = Config::default();
    config.per_file_ignores.insert("**/cli/**".to_string(), vec!["no-print".to_string()]);
    let d = lint_source_with_config(source, "src/cli/main.py", &config);
    assert_eq!(count_rule(&d, "no-print"), 0);
}

// -- Per-rule config --

#[test]
fn max_function_params_custom() {
    let source = "def f(a, b, c, d, e, f, g, h, i, j):\n    pass";
    let mut config = Config::default();
    config.rules.max_function_params = Some(slopcop::config::MaxFunctionParamsConfig { max: 10 });
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "max-function-params"), 0);
}

#[test]
fn max_function_params_custom_exceeded() {
    let source = "def f(a, b, c, d, e, f, g, h, i, j, k):\n    pass";
    let mut config = Config::default();
    config.rules.max_function_params = Some(slopcop::config::MaxFunctionParamsConfig { max: 10 });
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "max-function-params"), 1);
}

// -- Python version --

#[test]
fn python_version_313_keeps_future_annotations() {
    let source = "from __future__ import annotations";
    let mut config = Config::default();
    config.min_python_version = Some((3, 13));
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-future-annotations") >= 1);
}

#[test]
fn python_version_310_disables_future_annotations() {
    let source = "from __future__ import annotations";
    let mut config = Config::default();
    config.min_python_version = Some((3, 10));
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "no-future-annotations"), 0);
}

#[test]
fn python_version_none_keeps_future_annotations() {
    let source = "from __future__ import annotations";
    let config = Config::default();
    let d = lint_source_with_config(source, "<test>", &config);
    assert!(count_rule(&d, "no-future-annotations") >= 1);
}

// -- Severity --

#[test]
fn error_severity_on_bare_except() {
    let source = "try:\n    pass\nexcept:\n    pass";
    let d = lint_source_with_config(source, "<test>", &Config::default());
    let bare = d.iter().find(|d| d.rule_id == "no-bare-except").unwrap();
    assert_eq!(bare.severity, slopcop::Severity::Error);
}

#[test]
fn warning_severity_on_print() {
    let source = "print(\"hi\")";
    let d = lint_source_with_config(source, "<test>", &Config::default());
    let print_d = d.iter().find(|d| d.rule_id == "no-print").unwrap();
    assert_eq!(print_d.severity, slopcop::Severity::Warning);
}

#[test]
fn display_format_includes_severity() {
    let source = "print(\"hi\")";
    let d = lint_source_with_config(source, "<test>", &Config::default());
    let print_d = d.iter().find(|d| d.rule_id == "no-print").unwrap();
    let formatted = format!("{print_d}");
    assert!(formatted.contains("warning[no-print]"), "got: {formatted}");
}

// -- Help overrides --

#[test]
fn help_override_replaces_builtin() {
    let config = Config {
        help_overrides: [("no-print".to_string(), "Use structlog.".to_string())]
            .into_iter()
            .collect(),
        ..Config::default()
    };
    let map = slopcop::rules::help_texts(&config);
    assert_eq!(map.get("no-print").map(|s| s.as_str()), Some("Use structlog."));
}

#[test]
fn help_override_preserves_unoverridden() {
    let config = Config {
        help_overrides: [("no-print".to_string(), "custom".to_string())]
            .into_iter()
            .collect(),
        ..Config::default()
    };
    let map = slopcop::rules::help_texts(&config);
    let bare_help = map.get("no-bare-except").map(|s| s.as_str()).unwrap_or("");
    assert!(bare_help.contains("KeyboardInterrupt"), "built-in help should be untouched");
}

#[test]
fn help_override_coexists_with_max() {
    let mut config = Config::default();
    config.rules.max_function_params = Some(slopcop::config::MaxFunctionParamsConfig { max: 5 });
    config.help_overrides.insert("max-function-params".to_string(), "Use a model.".to_string());

    let source = "def f(a, b, c, d, e, f):\n    pass";
    let d = lint_source_with_config(source, "<test>", &config);
    assert_eq!(count_rule(&d, "max-function-params"), 1);

    let map = slopcop::rules::help_texts(&config);
    assert_eq!(map.get("max-function-params").map(|s| s.as_str()), Some("Use a model."));
}

#[test]
fn display_format_error_severity() {
    let source = "try:\n    pass\nexcept:\n    pass";
    let d = lint_source_with_config(source, "<test>", &Config::default());
    let bare = d.iter().find(|d| d.rule_id == "no-bare-except").unwrap();
    let formatted = format!("{bare}");
    assert!(formatted.contains("error[no-bare-except]"), "got: {formatted}");
}
