use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub exclude: Vec<String>,
    pub per_file_ignores: HashMap<String, Vec<String>>,
    pub rules: RuleConfigs,
    pub min_python_version: Option<(u32, u32)>,
    /// Per-rule help text overrides from `[tool.slopcop.rules.<id>].help`.
    pub help_overrides: HashMap<String, String>,
}

impl Config {
    /// Return the set of rule IDs excluded for a given file path.
    /// Combines global `exclude` with matching `per-file-ignores` patterns.
    pub fn excludes_for_path(&self, path: &str) -> HashSet<String> {
        let mut excluded: HashSet<String> = self.exclude.iter().cloned().collect();
        for (pattern, rules) in &self.per_file_ignores {
            if glob_matches(pattern, path) {
                excluded.extend(rules.iter().cloned());
            }
        }
        excluded
    }
}

#[derive(Debug, Default, Clone)]
pub struct RuleConfigs {
    pub max_function_params: Option<MaxFunctionParamsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaxFunctionParamsConfig {
    pub max: usize,
}

/// Raw serde-friendly mirror of Config for initial deserialization.
/// The `rules` table is kept as raw TOML values so we can extract both
/// known typed fields (e.g. `max-function-params.max`) and arbitrary
/// `help` strings from any rule table.
#[derive(Deserialize)]
struct RawConfig {
    #[serde(default)]
    exclude: Vec<String>,

    #[serde(default, rename = "per-file-ignores")]
    per_file_ignores: HashMap<String, Vec<String>>,

    #[serde(default)]
    rules: HashMap<String, toml::Value>,
}

impl RawConfig {
    fn into_config(self) -> Config {
        let mut rule_configs = RuleConfigs::default();
        let mut help_overrides = HashMap::new();

        for (rule_id, value) in &self.rules {
            if let Some(table) = value.as_table() {
                if rule_id == "max-function-params" {
                    if let Some(max_val) = table.get("max").and_then(|v| v.as_integer()) {
                        rule_configs.max_function_params =
                            Some(MaxFunctionParamsConfig { max: max_val as usize });
                    }
                }
                if let Some(help) = table.get("help").and_then(|v| v.as_str()) {
                    help_overrides.insert(rule_id.clone(), help.to_string());
                }
            }
        }

        Config {
            exclude: self.exclude,
            per_file_ignores: self.per_file_ignores,
            rules: rule_configs,
            min_python_version: None,
            help_overrides,
        }
    }
}

#[derive(Deserialize)]
struct PyprojectToml {
    project: Option<ProjectTable>,
    tool: Option<ToolTable>,
}

#[derive(Deserialize)]
struct ProjectTable {
    #[serde(rename = "requires-python")]
    requires_python: Option<String>,
}

#[derive(Deserialize)]
struct ToolTable {
    slopcop: Option<RawConfig>,
}

/// Walk upward from `start_dir` looking for a pyproject.toml with [tool.slopcop].
/// Returns Config::default() if nothing is found.
pub fn discover_config(start_dir: &Path) -> Config {
    let mut dir = if start_dir.is_file() {
        start_dir.parent().unwrap_or(start_dir)
    } else {
        start_dir
    };

    loop {
        let candidate = dir.join("pyproject.toml");
        if candidate.is_file() {
            if let Ok(config) = try_load_config(&candidate) {
                return config;
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Config::default(),
        }
    }
}

fn try_load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let pyproject: PyprojectToml = toml::from_str(&content)?;

    let mut config = pyproject
        .tool
        .and_then(|t| t.slopcop)
        .map(|raw| raw.into_config())
        .unwrap_or_default();

    if let Some(ref project) = pyproject.project {
        if let Some(ref spec) = project.requires_python {
            config.min_python_version = parse_min_python_version(spec);
        }
    }

    Ok(config)
}

/// Parse a PEP 440 version specifier to extract the minimum version.
/// Handles: ">=3.10", ">=3.13,<4", "==3.12", ">=3.10.1"
fn parse_min_python_version(spec: &str) -> Option<(u32, u32)> {
    for part in spec.split(',') {
        let trimmed = part.trim();
        let version_str = if let Some(v) = trimmed.strip_prefix(">=") {
            Some(v.trim())
        } else if let Some(v) = trimmed.strip_prefix("==") {
            Some(v.trim())
        } else {
            None
        };

        if let Some(v) = version_str {
            let parts: Vec<&str> = v.split('.').collect();
            if parts.len() >= 2 {
                if let (Ok(major), Ok(minor)) = (parts[0].parse(), parts[1].parse()) {
                    return Some((major, minor));
                }
            }
        }
    }
    None
}

/// Simple glob matching for per-file-ignores patterns.
/// Supports `*` (any non-/ chars), `**` (any path segment), and `?` (single char).
fn glob_matches(pattern: &str, path: &str) -> bool {
    // Normalize separators
    let pattern = pattern.replace('\\', "/");
    let path = path.replace('\\', "/");
    glob_match_recursive(&pattern, &path)
}

fn glob_match_recursive(pattern: &str, path: &str) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }

    if let Some(rest) = pattern.strip_prefix("**/") {
        // ** matches zero or more path segments
        if glob_match_recursive(rest, path) {
            return true;
        }
        for (i, c) in path.char_indices() {
            if c == '/' {
                if glob_match_recursive(rest, &path[i + 1..]) {
                    return true;
                }
            }
        }
        return false;
    }

    if pattern == "**" {
        return true;
    }

    if let Some(rest) = pattern.strip_prefix('*') {
        // * matches any non-/ characters
        for i in 0..=path.len() {
            if i > 0 && path.as_bytes()[i - 1] == b'/' {
                break;
            }
            if glob_match_recursive(rest, &path[i..]) {
                return true;
            }
        }
        return false;
    }

    if let Some(rest) = pattern.strip_prefix('?') {
        if let Some(c) = path.chars().next() {
            if c != '/' {
                return glob_match_recursive(rest, &path[c.len_utf8()..]);
            }
        }
        return false;
    }

    // Literal character match
    if let (Some(pc), Some(tc)) = (pattern.chars().next(), path.chars().next()) {
        if pc == tc {
            return glob_match_recursive(&pattern[pc.len_utf8()..], &path[tc.len_utf8()..]);
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_double_star_prefix() {
        assert!(glob_matches("**/test_*.py", "tests/test_foo.py"));
        assert!(glob_matches("**/test_*.py", "a/b/test_bar.py"));
        assert!(!glob_matches("**/test_*.py", "foo.py"));
    }

    #[test]
    fn test_glob_directory_pattern() {
        assert!(glob_matches("tests/**", "tests/test_foo.py"));
        assert!(glob_matches("tests/**", "tests/sub/deep.py"));
        assert!(!glob_matches("tests/**", "src/foo.py"));
    }

    #[test]
    fn test_glob_middle_double_star() {
        assert!(glob_matches("**/cli/**", "src/cli/main.py"));
        assert!(glob_matches("**/cli/**", "cli/commands/run.py"));
        assert!(!glob_matches("**/cli/**", "src/api/main.py"));
    }

    #[test]
    fn test_glob_star_extension() {
        assert!(glob_matches("*.py", "foo.py"));
        assert!(!glob_matches("*.py", "dir/foo.py"));
    }

    #[test]
    fn test_parse_min_python_version() {
        assert_eq!(parse_min_python_version(">=3.10"), Some((3, 10)));
        assert_eq!(parse_min_python_version(">=3.13"), Some((3, 13)));
        assert_eq!(parse_min_python_version(">=3.13,<4"), Some((3, 13)));
        assert_eq!(parse_min_python_version("==3.12"), Some((3, 12)));
        assert_eq!(parse_min_python_version(">=3.10.1"), Some((3, 10)));
        assert_eq!(parse_min_python_version("~=3.8"), None); // ~= not supported
    }

    #[test]
    fn test_help_override_parsing() {
        let toml_str = r#"
[tool.slopcop.rules.no-print]
help = "Use structlog in this project."

[tool.slopcop.rules.max-function-params]
max = 10
help = "Group into a Pydantic model."
"#;
        let pyproject: PyprojectToml = toml::from_str(toml_str).unwrap();
        let config = pyproject.tool.unwrap().slopcop.unwrap().into_config();

        assert_eq!(
            config.help_overrides.get("no-print").map(|s| s.as_str()),
            Some("Use structlog in this project.")
        );
        assert_eq!(
            config.help_overrides.get("max-function-params").map(|s| s.as_str()),
            Some("Group into a Pydantic model.")
        );
        assert_eq!(config.rules.max_function_params.as_ref().unwrap().max, 10);
    }

    #[test]
    fn test_help_override_empty_rules() {
        let toml_str = "[tool.slopcop]\n";
        let pyproject: PyprojectToml = toml::from_str(toml_str).unwrap();
        let config = pyproject.tool.unwrap().slopcop.unwrap().into_config();
        assert!(config.help_overrides.is_empty());
    }
}
