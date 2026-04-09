# slopcop

A fast, opinionated Python linter written in Rust that catches anti-patterns commonly produced by language models.

Built on [tree-sitter](https://tree-sitter.github.io/) for AST-correct analysis — no false positives on strings or comments.

## Install

```bash
pip install slopcop
# or
uv tool install slopcop
```

Or from source:

```bash
cargo install --git https://github.com/cm2435/slopcop
```

## Usage

```bash
# Lint directories (walks recursively for .py files)
slopcop src/ tests/

# Lint specific files
slopcop path/to/file.py

# Quiet mode (exit code only, for CI)
slopcop --quiet src/

# JSON output
slopcop --format json src/

# Only fail on errors, treat warnings as non-blocking
slopcop --warn-only src/
```

Exit codes: `0` = clean, `1` = violations found, `2` = fatal error.

## Rules

All rules are **enabled by default**. Disable per-project via `pyproject.toml`.

| Rule | What it catches |
|------|----------------|
| `no-hasattr-getattr` | `hasattr()` and `getattr()` calls — use explicit attribute checks or protocols |
| `guarded-function-import` | Function-scope `import` without a comment on the line above explaining why |
| `no-future-annotations` | `from __future__ import annotations` — unnecessary on 3.13+ and breaks runtime inspection |
| `no-dataclass` | `@dataclass` usage and `dataclasses` imports — use Pydantic or project-standard models |
| `no-bare-except` | `except:` without a type — catches everything including KeyboardInterrupt |
| `no-broad-except` | `except Exception:` / `except BaseException:` — too broad, catch specific types |
| `no-pass-except` | `except` blocks containing only `pass` — silently swallows exceptions |
| `no-nested-try` | Nested `try` blocks — extract the inner try into a separate function |
| `no-print` | `print()` calls — use structured logging |
| `no-todo-comment` | `TODO`, `FIXME`, `HACK`, `XXX` comments — resolve or track in an issue |
| `no-assert` | `assert` in production code — use `if not ...: raise ValueError(...)` instead |
| `no-typing-any` | `Any` in type annotations — use specific types or protocols |
| `no-str-empty-default` | `str = ""` defaults on params and model fields — use `str \| None = None` or make required |
| `no-boolean-positional` | Bare `True`/`False` as positional arguments — use keyword arguments for clarity |
| `no-redundant-none-check` | `x is None` when `x` is typed as non-optional |
| `max-function-params` | Functions with more than 8 parameters (configurable) — group into a model |

## Configuration

Add `[tool.slopcop]` to your `pyproject.toml`:

```toml
[tool.slopcop]
exclude = [
    "no-dataclass",   # this project uses dataclasses
    "no-print",       # CLI app, print is fine
]
```

### Per-file ignores

Disable rules for specific file patterns:

```toml
[tool.slopcop.per-file-ignores]
"tests/**" = ["no-print"]
"**/cli/**" = ["no-print"]
```

### Rule-specific config

```toml
[tool.slopcop.rules.max-function-params]
max = 10
```

slopcop walks upward from the target path to find the nearest `pyproject.toml`.

## Inline suppression

```python
x = getattr(obj, name)                     # slopcop: ignore
x = getattr(obj, name)                     # slopcop: ignore[no-hasattr-getattr]
x = getattr(obj, name)                     # slopcop: ignore[no-hasattr-getattr, no-print]
```

## Adding to CI

```yaml
# GitHub Actions
- run: pip install slopcop
- run: slopcop src/ tests/
```

## Development

```bash
cargo test          # run the test suite
cargo run -- src/   # run locally
```

## License

MIT
