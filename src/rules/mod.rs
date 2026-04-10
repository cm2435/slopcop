pub mod guarded_function_import;
pub mod max_function_params;
pub mod no_assert;
pub mod no_bare_except;
pub mod no_boolean_positional;
pub mod no_broad_except;
pub mod no_dataclass;
pub mod no_future_annotations;
pub mod no_hasattr_getattr;
pub mod no_nested_try;
pub mod no_pass_except;
pub mod no_print;
pub mod no_redundant_none_check;
pub mod no_sentinel_default;
pub mod no_str_empty_default;
pub mod no_todo_comment;
pub mod no_typing_any;

use crate::config::Config;
use crate::diagnostic::Diagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn severity(&self) -> Severity { Severity::Error }
    /// Detailed guidance shown once per rule when violations are grouped.
    /// Should explain *why* the pattern is bad and *how* to fix it properly.
    fn help(&self) -> &'static str { "" }
    fn node_kinds(&self) -> &'static [&'static str];
    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    );
}

/// Build the default rule set. Pass config to apply per-rule settings.
pub fn all_rules_with_config(config: &Config) -> Vec<Box<dyn Rule>> {
    let max = config
        .rules
        .max_function_params
        .as_ref()
        .map(|c| c.max)
        .unwrap_or(max_function_params::DEFAULT_MAX_PARAMS);

    vec![
        Box::new(no_hasattr_getattr::NoHasattrGetattr),
        Box::new(guarded_function_import::GuardedFunctionImport),
        Box::new(no_future_annotations::NoFutureAnnotations),
        Box::new(no_dataclass::NoDataclass),
        Box::new(no_bare_except::NoBareExcept),
        Box::new(no_broad_except::NoBroadExcept),
        Box::new(no_print::NoPrint),
        Box::new(no_todo_comment::NoTodoComment),
        Box::new(no_str_empty_default::NoStrEmptyDefault),
        Box::new(no_typing_any::NoTypingAny),
        Box::new(no_assert::NoAssert),
        Box::new(no_nested_try::NoNestedTry),
        Box::new(no_pass_except::NoPassExcept),
        Box::new(max_function_params::MaxFunctionParams { max }),
        Box::new(no_boolean_positional::NoBooleanPositional),
        Box::new(no_redundant_none_check::NoRedundantNoneCheck),
        Box::new(no_sentinel_default::NoSentinelDefault),
    ]
}

/// Build the default rule set with default config.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    all_rules_with_config(&Config::default())
}

/// Return a map from rule_id → help text for all rules, with user overrides applied.
pub fn help_texts(config: &Config) -> std::collections::HashMap<&'static str, String> {
    let rules = all_rules_with_config(config);

    let mut map: std::collections::HashMap<&'static str, String> = rules
        .iter()
        .filter(|r| !r.help().is_empty())
        .map(|r| (r.name(), r.help().to_string()))
        .collect();

    for (rule_id, text) in &config.help_overrides {
        if let Some(key) = map.keys().copied().find(|k| *k == rule_id.as_str()) {
            map.insert(key, text.clone());
        }
    }

    map
}
