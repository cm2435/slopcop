use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use clap::Parser;
use ignore::WalkBuilder;
use rayon::prelude::*;

use slopcop::config;
use slopcop::diagnostic::Diagnostic;
use slopcop::engine;
use slopcop::rules::{self, Severity};

#[derive(Parser)]
#[command(name = "slopcop", version, about = "Fast Python linter for LLM-generated anti-patterns")]
struct Cli {
    /// Files or directories to lint (walks recursively for .py files)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Rule IDs to run. If provided, only these rules are enabled.
    #[arg(long, value_delimiter = ',')]
    select: Vec<String>,

    /// Rule IDs to skip after selection/config has been resolved.
    #[arg(long, value_delimiter = ',')]
    ignore: Vec<String>,

    /// Suppress output; only set exit code
    #[arg(short, long)]
    quiet: bool,

    /// Only fail (exit 1) on errors, not warnings
    #[arg(long)]
    warn_only: bool,

    /// Output format
    #[arg(long, default_value = "text")]
    format: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn main() {
    let cli = Cli::parse();

    let code = match run(&cli) {
        Ok(has_violations) => {
            if has_violations { 1 } else { 0 }
        }
        Err(e) => {
            eprintln!("slopcop: {e:#}");
            2
        }
    };

    process::exit(code);
}

fn run(cli: &Cli) -> Result<bool> {
    let mut config = config::discover_config(
        &std::env::current_dir().unwrap_or_else(|_| {
            cli.paths
                .first()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("."))
        }),
    );

    apply_cli_rule_filters(&mut config, &cli.select, &cli.ignore)?;

    let files = collect_python_files(&cli.paths)?;

    if files.is_empty() {
        return Ok(false);
    }

    let error_count = AtomicUsize::new(0);
    let warning_count = AtomicUsize::new(0);

    let all_diagnostics: Vec<Vec<Diagnostic>> = files
        .par_iter()
        .filter_map(|path| {
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let path_str = path.to_string_lossy();
            let diagnostics =
                engine::lint_source_with_config(&source, &path_str, &config);

            if diagnostics.is_empty() {
                None
            } else {
                for d in &diagnostics {
                    match d.severity {
                        Severity::Error => { error_count.fetch_add(1, Ordering::Relaxed); }
                        Severity::Warning => { warning_count.fetch_add(1, Ordering::Relaxed); }
                    }
                }
                Some(diagnostics)
            }
        })
        .collect();

    if !cli.quiet {
        let mut flat: Vec<&Diagnostic> = all_diagnostics.iter().flat_map(|v| v.iter()).collect();
        flat.sort_by(|a, b| {
            a.path
                .cmp(&b.path)
                .then(a.line.cmp(&b.line))
                .then(a.col.cmp(&b.col))
        });

        match cli.format {
            OutputFormat::Text => {
                print_grouped_text(&flat, &config);
            }
            OutputFormat::Json => {
                print_grouped_json(&flat, &config);
            }
        }

        let errors = error_count.load(Ordering::Relaxed);
        let warnings = warning_count.load(Ordering::Relaxed);
        let total = errors + warnings;
        if total > 0 {
            eprintln!(
                "\nFound {total} violation{} ({errors} error{}, {warnings} warning{}) across {} file{}.",
                if total == 1 { "" } else { "s" },
                if errors == 1 { "" } else { "s" },
                if warnings == 1 { "" } else { "s" },
                all_diagnostics.len(),
                if all_diagnostics.len() == 1 { "" } else { "s" },
            );
        }
    }

    if cli.warn_only {
        Ok(error_count.load(Ordering::Relaxed) > 0)
    } else {
        let total = error_count.load(Ordering::Relaxed) + warning_count.load(Ordering::Relaxed);
        Ok(total > 0)
    }
}

/// Group diagnostics by rule_id, print help text once per group, then list locations.
fn print_grouped_text(diagnostics: &[&Diagnostic], config: &config::Config) {
    let help_map = rules::help_texts(config);

    // BTreeMap for deterministic ordering by rule_id
    let mut groups: BTreeMap<&str, (Severity, Vec<&Diagnostic>)> = BTreeMap::new();
    for d in diagnostics {
        groups
            .entry(d.rule_id)
            .or_insert_with(|| (d.severity, Vec::new()))
            .1
            .push(d);
    }

    for (rule_id, (severity, diags)) in &groups {
        let sev = match severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let count = diags.len();
        eprintln!(
            "{sev}[{rule_id}] ({count} violation{})",
            if count == 1 { "" } else { "s" }
        );

        if let Some(help) = help_map.get(rule_id) {
            for line in textwrap(help.as_str(), 76) {
                eprintln!("  {line}");
            }
            eprintln!();
        }

        for d in diags {
            eprintln!("  {}:{}:{}", d.path, d.line, d.col);
        }
        eprintln!();
    }
}

/// Group diagnostics by rule_id in JSON output, with help text per group.
fn print_grouped_json(diagnostics: &[&Diagnostic], config: &config::Config) {
    let help_map = rules::help_texts(config);

    let mut groups: BTreeMap<&str, Vec<&Diagnostic>> = BTreeMap::new();
    for d in diagnostics {
        groups.entry(d.rule_id).or_default().push(d);
    }

    let json_groups: Vec<serde_json::Value> = groups
        .into_iter()
        .map(|(rule_id, diags)| {
            let locations: Vec<serde_json::Value> = diags
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "path": d.path,
                        "line": d.line,
                        "col": d.col,
                        "message": d.message,
                    })
                })
                .collect();
            serde_json::json!({
                "rule_id": rule_id,
                "severity": match diags[0].severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                },
                "help": help_map.get(rule_id).map(|s| s.as_str()).unwrap_or(""),
                "count": diags.len(),
                "violations": locations,
            })
        })
        .collect();

    eprintln!("{}", serde_json::to_string_pretty(&json_groups).unwrap());
}

/// Simple word-wrap that breaks on whitespace boundaries.
fn textwrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn collect_python_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_file() {
            files.push(path.clone());
            continue;
        }

        if !path.exists() {
            anyhow::bail!("path does not exist: {}", path.display());
        }

        let walker = WalkBuilder::new(path)
            .hidden(true)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry.context("walking directory")?;
            let p = entry.path();
            if p.is_file() && p.extension().is_some_and(|ext| ext == "py") {
                files.push(p.to_path_buf());
            }
        }
    }

    Ok(files)
}

fn apply_cli_rule_filters(
    config: &mut config::Config,
    select: &[String],
    ignore: &[String],
) -> Result<()> {
    let known_rule_names = rules::all_rule_names();
    let known_rule_set: BTreeSet<&str> = known_rule_names.iter().copied().collect();

    let invalid_rule_ids = select
        .iter()
        .chain(ignore.iter())
        .filter(|rule_id| !known_rule_set.contains(rule_id.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>();

    if !invalid_rule_ids.is_empty() {
        let invalid_rule_ids = invalid_rule_ids.into_iter().collect::<Vec<_>>().join(", ");
        anyhow::bail!("unknown rule id(s): {invalid_rule_ids}");
    }

    if !select.is_empty() {
        let selected_rule_ids: BTreeSet<&str> = select.iter().map(String::as_str).collect();
        config.exclude.extend(
            known_rule_names
                .into_iter()
                .filter(|rule_id| !selected_rule_ids.contains(rule_id))
                .map(str::to_owned),
        );
    }

    config.exclude.extend(ignore.iter().cloned());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_cli_rule_filters;
    use slopcop::{Config, lint_source_with_config};

    #[test]
    fn select_runs_only_selected_rules() {
        let mut config = Config::default();
        let source = "print(\"hi\")\nassert value";

        apply_cli_rule_filters(&mut config, &["no-print".to_string()], &[]).unwrap();

        let diagnostics = lint_source_with_config(source, "src/example.py", &config);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.rule_id == "no-print"));
        assert!(!diagnostics.iter().any(|diagnostic| diagnostic.rule_id == "no-assert"));
    }

    #[test]
    fn ignore_skips_rule() {
        let mut config = Config::default();
        let source = "print(\"hi\")";

        apply_cli_rule_filters(&mut config, &[], &["no-print".to_string()]).unwrap();

        let diagnostics = lint_source_with_config(source, "src/example.py", &config);

        assert!(!diagnostics.iter().any(|diagnostic| diagnostic.rule_id == "no-print"));
    }

    #[test]
    fn unknown_rule_ids_error() {
        let mut config = Config::default();

        let error = apply_cli_rule_filters(
            &mut config,
            &["no-print".to_string()],
            &["not-a-rule".to_string()],
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "unknown rule id(s): not-a-rule");
    }
}
