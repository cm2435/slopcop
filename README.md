# pyguard

A fast, opinionated Python linter written in Rust that catches anti-patterns commonly produced by language models.

Built on [tree-sitter](https://tree-sitter.github.io/) for AST-correct analysis — no false positives on strings or comments.

## Install

```bash
pip install pyguard
# or
uv tool install pyguard
# or
cargo install pyguard
```

## Usage

```bash
# Lint directories (walks recursively for .py files)
pyguard src/ tests/

# Lint specific files
pyguard path/to/file.py

# Quiet mode (exit code only, for CI)
pyguard --quiet src/

# JSON output
pyguard --format json src/
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
| `no-bare-except` | `except:`, `except Exception:`, `except BaseException:` — catch specific types |
| `no-print` | `print()` calls — use structured logging |
| `no-todo-comment` | `TODO`, `FIXME`, `HACK`, `XXX` comments — resolve or track in an issue |

## Configuration

Add `[tool.pyguard]` to your `pyproject.toml`:

```toml
[tool.pyguard]
exclude = [
    "no-dataclass",   # this project uses dataclasses
    "no-print",       # CLI app, print is fine
]
```

pyguard walks upward from the target path to find the nearest `pyproject.toml`.

## Inline suppression

```python
x = getattr(obj, name)                     # pyguard: ignore
x = getattr(obj, name)                     # pyguard: ignore[no-hasattr-getattr]
x = getattr(obj, name)                     # pyguard: ignore[no-hasattr-getattr, no-print]
```

## Adding to CI

```yaml
# GitHub Actions
- run: pip install pyguard
- run: pyguard src/ tests/
```

## Development

```bash
cargo test          # run all 89 tests
cargo run -- src/   # run locally
```

## License

MIT
