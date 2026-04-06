# pyguard -- Technical Proposal

> A standalone, reusable Rust CLI linter for Python codebases.
> Detects anti-patterns commonly produced by language models.
> Configurable per-project via `[tool.pyguard]` in `pyproject.toml`.

## 1. Motivation

Language models generating Python code routinely produce patterns that are
technically valid but architecturally harmful:

- **`hasattr()` / `getattr()`** -- duck-typing escape hatches that hide type
  errors and make static analysis useless.
- **Unguarded function-scope imports** -- deferred imports that accumulate
  silently. Each one should be a conscious decision with a documented reason,
  not a default habit.
- **`from __future__ import annotations`** -- changes runtime semantics of type
  hints in ways that break Pydantic, FastAPI, and other frameworks that inspect
  annotations at runtime. On Python 3.13+ (our target) it is never necessary.
- **`@dataclass`** -- in projects that standardize on Pydantic/SQLModel, a
  stray dataclass is the LM defaulting to stdlib instead of using the project's
  conventions.
- **Bare `except:`** -- swallows all exceptions including KeyboardInterrupt and
  SystemExit. LMs produce these constantly.
- **`print()`** -- in library/application code that uses structured logging,
  `print()` is always wrong. LMs scatter these liberally.

These are not caught by ruff, ty, or mypy because they are style/intent
violations, not correctness violations. A dedicated tool keeps them out of the
codebase at CI time and gives the LM a clear signal to self-correct.

### Why not ruff?

Ruff covers some overlapping ground (`F403` for star imports, `B006` for
mutable defaults, `PGH003` for untyped `type: ignore`), but pyguard targets
patterns ruff deliberately does not lint -- convention/intent violations that
are project-specific and opinionated. The `[tool.pyguard]` configuration
makes this practical: enable the opinionated rules where they fit, disable
them where they don't.

---

## 2. Architecture overview

```
                ┌─────────────────────────────────┐
                │           CLI (clap)             │
                │  args: paths, --format, --quiet  │
                └───────────┬─────────────────────┘
                            │
                ┌───────────▼─────────────────────┐
                │  Config discovery (config.rs)     │
                │  walk up dirs → pyproject.toml    │
                │  read [tool.pyguard]              │
                └───────────┬─────────────────────┘
                            │  Config { exclude }
                ┌───────────▼─────────────────────┐
                │  Build rule registry              │
                │  all_rules() minus excluded       │
                └───────────┬─────────────────────┘
                            │
                ┌───────────▼─────────────────────┐
                │     File discovery (ignore)      │
                │  respects .gitignore, parallel   │
                └───────────┬─────────────────────┘
                            │  Vec<PathBuf>
                ┌───────────▼─────────────────────┐
                │    Parallel lint (rayon)          │
                │  par_iter over files              │
                └───────────┬─────────────────────┘
                            │  per file:
                ┌───────────▼─────────────────────┐
                │  tree-sitter-python parse         │
                │  → concrete syntax tree           │
                └───────────┬─────────────────────┘
                            │
                ┌───────────▼─────────────────────┐
                │     Single-pass CST walk          │
                │  cursor-based depth-first         │
                │                                   │
                │  At each node:                    │
                │    kind → lookup in dispatch map  │
                │    dispatch only to matching rules│
                └───────────┬─────────────────────┘
                            │  Vec<Diagnostic>
                ┌───────────▼─────────────────────┐
                │  Inline suppression filter        │
                │  remove # pyguard: ignore lines   │
                └───────────┬─────────────────────┘
                            │  Vec<Diagnostic>
                ┌───────────▼─────────────────────┐
                │     Aggregate + render            │
                │  sort by file:line:col            │
                │  exit 0 or 1                      │
                └───────────────────────────────────┘
```

### Design principles (borrowed from oxlint)

**Rule trait pattern.** Every rule is a struct implementing a `Rule` trait.
Rules declare which CST node kinds they care about, so the engine only
dispatches relevant nodes. Adding a new rule means: one new file, implement
the trait, one line in the registry. Zero changes to the engine.

```rust
pub trait Rule: Send + Sync {
    /// Unique rule identifier, e.g. "no-hasattr-getattr"
    fn name(&self) -> &'static str;

    /// CST node kinds this rule wants to inspect.
    /// The engine skips dispatch for nodes not in this list.
    fn node_kinds(&self) -> &'static [&'static str];

    /// Inspect a single node. Push to `diagnostics` if violated.
    fn check(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        ancestors: &[tree_sitter::Node],
        diagnostics: &mut Vec<Diagnostic>,
    );
}
```

**Single-pass dispatch.** The engine builds a `HashMap<&str, Vec<&dyn Rule>>`
from the registry at startup. During the tree walk, it looks up
`node.kind()` and only calls `check()` on rules that registered for that
kind. Cost of adding rules: O(1) per rule at startup, zero overhead per node
for non-matching rules.

**Ancestor stack.** The engine maintains a `Vec<Node>` ancestor stack during
traversal, pushed/popped as the cursor descends/ascends. This is passed to
`check()` so rules like `guarded-function-import` can inspect scope without
re-walking the tree.

**Inline suppression.** After the engine collects all diagnostics for a file,
it runs a post-filter pass. For each diagnostic, it checks whether the source
line at `diagnostic.line` contains a `# pyguard: ignore` comment. If so, the
diagnostic is dropped. This keeps suppression logic out of individual rules
entirely.

---

## 3. Configuration

### 3.1 Config file: `[tool.pyguard]` in `pyproject.toml`

pyguard discovers configuration by walking upward from the first target path
until it finds a `pyproject.toml` containing a `[tool.pyguard]` table. If
none is found, all rules are enabled (sensible default).

```toml
[tool.pyguard]
# Disable specific rules (all others remain active).
# Model: exclude-list. New rules are active by default when you upgrade.
exclude = [
    "no-dataclass",     # this project legitimately uses dataclasses
    "no-print",         # CLI app, print is fine
]
```

**Design decision: exclude-list, not select-list.** All rules are on by
default. When pyguard adds new rules in future versions, they take effect
immediately everywhere. Projects opt out of rules that don't fit. This is
the right default for a tool whose purpose is catching LM mistakes -- you
want maximum coverage unless you've consciously decided otherwise.

Config schema (deserialised to a Rust struct):

```rust
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// Rule IDs to disable. If empty (or absent), all rules active.
    #[serde(default)]
    pub exclude: Vec<String>,
}
```

### 3.2 Inline suppression: `# pyguard: ignore`

Any line can suppress diagnostics with a trailing comment:

```python
x = getattr(obj, name)                          # pyguard: ignore
x = getattr(obj, name)                          # pyguard: ignore[no-hasattr-getattr]
x = getattr(obj, name)                          # pyguard: ignore[no-hasattr-getattr, no-print]
```

Three forms:
- **Blanket:** `# pyguard: ignore` -- suppresses all rules on this line.
- **Targeted single:** `# pyguard: ignore[rule-id]` -- suppresses one rule.
- **Targeted multi:** `# pyguard: ignore[rule-a, rule-b]` -- suppresses listed rules.

**Implementation:** Post-filter in `engine.rs`. After collecting all
diagnostics for a file, iterate them and check the source line for the
suppression pattern. This is a single regex match per diagnostic, not per
node, so the cost is negligible. Rule implementations never see or handle
suppression -- it's fully orthogonal.

```rust
fn filter_suppressed(diagnostics: Vec<Diagnostic>, source: &str) -> Vec<Diagnostic> {
    let lines: Vec<&str> = source.lines().collect();
    diagnostics.into_iter().filter(|d| {
        let line = lines.get(d.line - 1).unwrap_or(&"");
        !is_suppressed(line, d.rule_id)
    }).collect()
}
```

---

## 4. Crate structure

```
pyguard/
├── Cargo.toml
├── TECHNICAL_PROPOSAL.md      ← this file
├── src/
│   ├── main.rs                # CLI: clap arg parsing, file discovery, parallel dispatch
│   ├── lib.rs                 # Public API: lint_source(), lint_file(), rule_registry()
│   ├── config.rs              # pyproject.toml discovery + [tool.pyguard] deserialization
│   ├── diagnostic.rs          # Diagnostic struct + Display impl
│   ├── suppression.rs         # Inline # pyguard: ignore parsing + filtering
│   ├── engine.rs              # tree-sitter parse, cursor walk, rule dispatch, suppression
│   └── rules/
│       ├── mod.rs             # Rule trait definition + all_rules() registry fn
│       ├── no_hasattr_getattr.rs
│       ├── guarded_function_import.rs
│       ├── no_future_annotations.rs
│       ├── no_dataclass.rs
│       ├── no_bare_except.rs
│       └── no_print.rs
└── tests/
    └── integration.rs         # Full-pipeline tests: source string → diagnostics
```

### Key API surface

```rust
// lib.rs -- the only public interface tests and future consumers need

/// Lint with all rules (default config).
pub fn lint_source(source: &str, path: &str) -> Vec<Diagnostic>;

/// Lint with specific rules (for testing individual rules).
pub fn lint_source_with_rules(source: &str, path: &str, rules: &[&dyn Rule]) -> Vec<Diagnostic>;

/// Lint with config (exclude list applied).
pub fn lint_source_with_config(source: &str, path: &str, config: &Config) -> Vec<Diagnostic>;
```

Tests call `lint_source()` or `lint_source_with_rules()` directly with inline
Python strings. No filesystem needed. This is what makes TDD ergonomic.

---

## 5. Dependencies

### Runtime

| Crate                  | Purpose                                          |
| ---------------------- | ------------------------------------------------ |
| `tree-sitter`          | CST parsing runtime                              |
| `tree-sitter-python`   | Python grammar for tree-sitter                   |
| `clap` (derive)        | CLI argument parsing                             |
| `ignore`               | File discovery with .gitignore respect, parallel  |
| `rayon`                | Parallel file linting                            |
| `anyhow`               | Error propagation                                |
| `toml`                 | Parse `pyproject.toml` for `[tool.pyguard]`      |
| `serde` + `serde_derive` | Deserialize config struct                      |
| `regex`                | Parse `# pyguard: ignore[...]` comments          |

### Dev-only

| Crate       | Purpose                                           |
| ----------- | ------------------------------------------------- |
| `insta`     | Snapshot testing for diagnostic output (optional)  |

We deliberately avoid `walkdir` in favor of `ignore` (same author, superset
functionality, built-in .gitignore filtering and parallel walk).

The `regex` crate is used only for inline suppression parsing. It could be
replaced with manual string matching if we want zero-regex, but the compile
cost is paid once at startup and the runtime cost per line is negligible.

---

## 6. Algorithm design

### 6.1 Rule 1: `no-hasattr-getattr`

**Registered node kinds:** `["call"]`

**Algorithm:**

1. Engine dispatches every `call` node to this rule.
2. Get the `function` child of the `call` node.
3. **Case A -- bare identifier:** If `function` is an `identifier` node with
   text `hasattr` or `getattr`, emit diagnostic.
4. **Case B -- attribute access:** If `function` is an `attribute` node (e.g.
   `builtins.getattr()`), get the `attribute` named child (the rightmost
   identifier). If its text is `hasattr` or `getattr`, emit diagnostic.
5. No other cases match. String literals, comments, and decorators never
   appear as `call → function` children in the CST.

**Diagnostic message:**
```
{path}:{line}:{col}  [no-hasattr-getattr] Avoid `{fn_name}()`; use explicit attribute checks or protocols
```

**Complexity:** O(1) per dispatched `call` node (constant work: one child
lookup, one string comparison).

---

### 6.2 Rule 2: `guarded-function-import`

**Registered node kinds:** `["import_statement", "import_from_statement"]`

**Algorithm:**

1. Engine dispatches every import node to this rule.
2. Walk the `ancestors` slice (provided by the engine) to check if any
   ancestor has kind `function_definition`.
   - If no function ancestor → module-scope import → **skip** (no violation).
3. Get the import node's start row `R` (0-indexed from tree-sitter).
4. Determine if row `R - 1` contains a comment:
   - Scan the **previous sibling** nodes of the import within the same block.
   - If the immediately preceding sibling is a `comment` node whose
     **end row** is `R - 1`, the import is guarded → **skip**.
   - Otherwise → **emit diagnostic**.
5. This means:
   - A blank line between comment and import → NOT guarded (comment's end
     row would be `R - 2` or less).
   - An inline comment on the import line itself → NOT guarded (it is a
     child/trailing of the import node, not a preceding sibling).
   - A comment two lines above → NOT guarded.

**Why "previous sibling" instead of source-line scanning:**
Using the CST's sibling relationship is more robust than byte-offset line
scanning. It correctly handles cases where the comment is on the same
indentation level and in the same block, which is exactly the pattern we
want to enforce.

**Diagnostic message:**
```
{path}:{line}:{col}  [guarded-function-import] Function-scope `import` requires a `# reason` comment on the line immediately above
```

**Complexity:** O(d) per import node where d = nesting depth (ancestor walk).
Typical d < 10.

---

### 6.3 Rule 3: `no-future-annotations`

**Registered node kinds:** `["import_from_statement"]`

**Algorithm:**

1. Engine dispatches every `import_from_statement` node.
2. Get the `module_name` child. If its text is not `__future__`, skip.
3. Iterate children looking for nodes with kind `dotted_name` or
   `aliased_import`:
   - For `dotted_name`: check if text is `annotations`.
   - For `aliased_import`: get the `name` child and check if its text is
     `annotations`.
4. If any imported name matches → emit diagnostic.

**Diagnostic message:**
```
{path}:{line}:{col}  [no-future-annotations] Do not use `from __future__ import annotations`; unnecessary on Python 3.13+ and breaks runtime annotation inspection
```

**Complexity:** O(k) per dispatched node where k = number of imported names
(typically 1-3).

---

### 6.4 Rule 4: `no-dataclass`

**Registered node kinds:** `["decorator", "import_from_statement", "import_statement"]`

**Goal:** Flag usage of `@dataclass` / `@dataclasses.dataclass` decorators and
imports of the `dataclasses` module. In projects standardized on Pydantic or
SQLModel, a `dataclass` is almost always the LM defaulting to stdlib.

**Algorithm:**

1. **Decorator form:** When the engine dispatches a `decorator` node:
   a. If the child is an `identifier` with text `dataclass`, emit diagnostic.
   b. If the child is a `call` whose `function` is an `identifier` with text
      `dataclass`, emit diagnostic (handles `@dataclass(frozen=True)`).
   c. If the child is an `attribute` with text ending in `dataclass` (e.g.
      `dataclasses.dataclass`), emit diagnostic.
   d. If the child is a `call` whose `function` is an `attribute` ending in
      `dataclass`, emit diagnostic.

2. **Import form:** When the engine dispatches an `import_statement` or
   `import_from_statement`:
   a. `import dataclasses` → emit diagnostic.
   b. `from dataclasses import dataclass` → emit diagnostic.
   c. `from dataclasses import field` → emit diagnostic (importing anything
      from `dataclasses` module signals usage).

**Diagnostic message:**
```
{path}:{line}:{col}  [no-dataclass] Avoid `dataclass`; use Pydantic `BaseModel` or project-standard model base
```

**Complexity:** O(1) per dispatched node.

This rule is opinionated by design. Projects that legitimately use dataclasses
should add `exclude = ["no-dataclass"]` to `[tool.pyguard]`.

---

### 6.5 Rule 5: `no-bare-except`

**Registered node kinds:** `["except_clause"]`

**Goal:** Flag `except:` (no type) and `except Exception:` (too broad). Both
silently swallow errors that should propagate.

**Algorithm:**

1. Engine dispatches every `except_clause` node.
2. Look for children that specify the exception type:
   - If the `except_clause` has no expression child after the `except` keyword
     → bare except → emit diagnostic.
   - If the expression child is an `identifier` with text `Exception`
     → overly broad → emit diagnostic.
   - If the expression child is an `identifier` with text `BaseException`
     → overly broad → emit diagnostic.
3. All other forms (`except ValueError:`, `except (KeyError, TypeError):`,
   etc.) are fine -- they demonstrate the developer thought about which
   exceptions to catch.

**Diagnostic messages:**
```
{path}:{line}:{col}  [no-bare-except] Bare `except:` catches all exceptions including KeyboardInterrupt; specify an exception type
{path}:{line}:{col}  [no-bare-except] `except Exception` is too broad; catch specific exception types
```

**Complexity:** O(1) per dispatched node.

---

### 6.6 Rule 6: `no-print`

**Registered node kinds:** `["call"]`

**Goal:** Flag calls to `print()` in library/application code. In projects
using structured logging (structlog, logging, logfire, etc.), a `print()` is
always wrong.

**Algorithm:**

1. Engine dispatches every `call` node (shared with Rule 1).
2. Get the `function` child.
3. If it is an `identifier` with text `print`, emit diagnostic.
4. If it is an `attribute` with the rightmost identifier `print` (e.g.
   `builtins.print()`), emit diagnostic.

**Diagnostic message:**
```
{path}:{line}:{col}  [no-print] Avoid `print()`; use structured logging (structlog, logging, etc.)
```

**Complexity:** O(1) per dispatched `call` node. Note: this rule shares the
`call` node kind with Rule 1 (`no-hasattr-getattr`). The dispatch map maps
`"call"` to both rules, so both are checked in the same pass.

This rule is opinionated -- CLI tools, scripts, and notebooks legitimately
use `print()`. Projects should exclude it via `[tool.pyguard]` if needed.

---

## 7. Engine: single-pass tree walk

Pseudocode for the core loop in `engine.rs`:

```
fn lint(source, rules) -> Vec<Diagnostic>:
    tree = tree_sitter_python.parse(source)
    cursor = tree.walk()
    ancestors = []
    diagnostics = []
    dispatch_map = build_dispatch_map(rules)   // kind → [rule, ...]

    loop:
        node = cursor.node()
        if let Some(matching_rules) = dispatch_map.get(node.kind()):
            for rule in matching_rules:
                rule.check(node, source, ancestors, &mut diagnostics)

        if cursor.goto_first_child():
            ancestors.push(node)
            continue

        while !cursor.goto_next_sibling():
            if !cursor.goto_parent():
                return diagnostics       // done: back at root
            ancestors.pop()

    diagnostics
```

This visits every node exactly once. The `dispatch_map` lookup is O(1)
(hash map on interned `&str` kinds). Most nodes match zero rules and cost
only the hash lookup.

---

## 8. TDD test matrix

All tests call `lint_source(code, "<test>")` and assert on the returned
`Vec<Diagnostic>`. Tests are written **before** any rule implementation.
They will initially all fail (red), and we implement until green.

### 8.1 Rule 1: `no-hasattr-getattr`

```
CASE                                           VIOLATIONS  NOTES
─────────────────────────────────────────────  ──────────  ──────────────────────────
hasattr(obj, "x")                              1           bare call
getattr(obj, "x")                              1           bare call
getattr(obj, "x", None)                        1           3-arg form
builtins.hasattr(obj, "x")                     1           attribute-form call
builtins.getattr(obj, "x")                     1           attribute-form call
# hasattr(obj, "x")                            0           inside comment
x = "hasattr(obj, 'x')"                        0           inside string
'''hasattr(obj, 'x')'''                         0           inside multiline string
obj.hasattr()                                  1           method with suspect name
result = getattr(o, k) if c else d             1           expression context
h = hasattr\ngetattr(o, "x")\nhasattr(o, "y") 3           multiple in one file
def f():\n    hasattr(self, "x")               1           inside function
lambda: hasattr(x, "y")                        1           inside lambda
```

### 8.2 Rule 2: `guarded-function-import`

```
CASE                                                             VIOLATIONS  NOTES
─────────────────────────────────────────────────────────────────  ──────────  ────────────────────
import os                                                         0           module scope
from os import path                                               0           module scope
def f():\n    import os                                           1           function scope, no comment
def f():\n    from os import path                                 1           from-import variant
def f():\n    # Deferred: circular\n    import os                 0           properly guarded
def f():\n    \n    # comment\n    import os                      1           blank line gap
def f():\n    import os  # reason                                 1           inline != preceding line
def f():\n    try:\n        import fast\n    except:\n        p   1           inside try, no comment
def f():\n    try:\n        # optional\n        import fast\n..   0           try, guarded
def f():\n    if cond:\n        import x                          1           inside if
def f():\n    with ctx:\n        import x                         1           inside with
class C:\n    def m(self):\n        import x                      1           class method
def f():\n    def g():\n        import x                          1           nested function
class C:\n    import x                                            1           class body
def f():\n    # a\n    import a\n    import b                     1           second import unguarded
def f():\n    # a\n    import a\n    # b\n    import b            0           both guarded
async def f():\n    import x                                      1           async function
```

### 8.3 Rule 3: `no-future-annotations`

```
CASE                                             VIOLATIONS  NOTES
───────────────────────────────────────────────  ──────────  ─────────────────────
from __future__ import annotations               1           basic case
from __future__ import division                   0           different future import
from __future__ import annotations, division      1           multi-import
from __future__ import (annotations)              1           parenthesized
from __future__ import (\n    annotations,\n)     1           multi-line parenthesized
import __future__                                 0           plain import, no effect
# from __future__ import annotations              0           in comment
"from __future__ import annotations"              0           in string
from __future__ import annotations as ann         1           aliased
def f():\n    from __future__ import annotations  1           (future imports here are
                                                               a SyntaxError, but tree-
                                                               sitter still parses it --
                                                               we flag it anyway)
```

### 8.4 Rule 4: `no-dataclass`

```
CASE                                                  VIOLATIONS  NOTES
────────────────────────────────────────────────────  ──────────  ──────────────────────────
@dataclass\nclass Foo:\n    x: int                    1           decorator usage
@dataclass(frozen=True)\nclass Foo:\n    x: int       1           decorator with args
@dataclasses.dataclass\nclass Foo:\n    x: int        1           qualified decorator
@dataclasses.dataclass(eq=False)\nclass Foo: pass     1           qualified with args
from dataclasses import dataclass                     1           import
from dataclasses import dataclass, field              1           multi-import
import dataclasses                                    1           module import
from pydantic import BaseModel                        0           not dataclasses
@some_other_decorator\nclass Foo: pass                0           unrelated decorator
"@dataclass"                                          0           in string
# @dataclass                                          0           in comment
```

### 8.5 Rule 5: `no-bare-except`

```
CASE                                                  VIOLATIONS  NOTES
────────────────────────────────────────────────────  ──────────  ──────────────────────────
try:\n    pass\nexcept:\n    pass                      1           bare except
try:\n    pass\nexcept Exception:\n    pass             1           except Exception
try:\n    pass\nexcept BaseException:\n    pass         1           except BaseException
try:\n    pass\nexcept ValueError:\n    pass            0           specific type
try:\n    pass\nexcept (KeyError, TypeError):\n  pass   0           tuple of specific types
try:\n    pass\nexcept Exception as e:\n    pass        1           broad with alias
try:\n    pass\nexcept ValueError:\n    pass\n
  except:\n    pass                                    1           second clause bare
try:\n    pass\nexcept OSError:\n    pass\n
  except IOError:\n    pass                            0           both specific
# except:                                              0           in comment
```

### 8.6 Rule 6: `no-print`

```
CASE                                           VIOLATIONS  NOTES
─────────────────────────────────────────────  ──────────  ──────────────────────────
print("hello")                                 1           bare call
print(x, y, sep=",")                           1           with args
builtins.print("hello")                        1           qualified call
# print("hello")                               0           in comment
x = "print('hello')"                           0           in string
logging.info("hello")                          0           not print
def f():\n    print("debug")                   1           inside function
print("a")\nprint("b")\nprint("c")            3           multiple
pp = pprint.pprint\npp(obj)                    0           different function
```

### 8.7 Configuration tests

```
CASE                                                          EXPECTED
──────────────────────────────────────────────────────────── ───────────────────────────────
Config { exclude: [] } + hasattr(obj, "x")                   1 violation
Config { exclude: ["no-hasattr-getattr"] } + hasattr(o, "x") 0 violations (rule excluded)
Config { exclude: ["no-print"] } + print("hi")               0 violations
Config { exclude: ["no-print"] } + hasattr(o, "x")           1 violation (different rule)
No pyproject.toml found                                       all rules active (default)
pyproject.toml without [tool.pyguard]                         all rules active (default)
pyproject.toml with empty exclude = []                        all rules active
```

### 8.8 Inline suppression tests

```
CASE                                                          EXPECTED
──────────────────────────────────────────────────────────── ───────────────────────────────
hasattr(o, "x")  # pyguard: ignore                          0 violations (blanket)
hasattr(o, "x")  # pyguard: ignore[no-hasattr-getattr]      0 violations (targeted)
hasattr(o, "x")  # pyguard: ignore[no-print]                1 violation (wrong rule)
hasattr(o, "x")  # pyguard: ignore[no-hasattr-getattr, z]   0 violations (multi-target)
print("hi")  # pyguard: ignore                              0 violations
hasattr(o, "x")  # some other comment                       1 violation (not suppression)
# pyguard: ignore\nhasattr(o, "x")                          1 violation (ignore is on
                                                              previous line, not same line)
def f():\n    import os  # pyguard: ignore                   0 violations (suppresses
                                                              guarded-function-import --
                                                              the inline ignore is parsed
                                                              as suppression, not as the
                                                              required guard comment)
```

### 8.9 Edge cases / robustness

```
CASE                                      EXPECTED BEHAVIOR
────────────────────────────────────────  ─────────────────────────────────────
""  (empty string)                        0 diagnostics, no panic
"# just comments\n# more"                0 diagnostics
"def f(\n    broken syntax"               0 or more diagnostics, NO PANIC
"\x00\x01\x02" (binary content)           0 diagnostics, no panic
100-level nested functions with imports   correct count, no stack overflow
"if True:\n    import os"                 0 violations (module-level if-import
                                           is not inside a function)
```

### 8.10 Fuzz target (optional)

A `cargo-fuzz` target in `fuzz/fuzz_targets/lint_source.rs` that feeds
arbitrary bytes to `lint_source()` and asserts no panics. This is a safety
net for tree-sitter edge cases.

---

## 9. Diagnostic struct

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: String,
    pub line: usize,        // 1-indexed for display
    pub col: usize,         // 0-indexed
    pub rule_id: &'static str,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}  [{}] {}",
            self.path, self.line, self.col, self.rule_id, self.message)
    }
}
```

Output format mirrors ruff/clippy conventions so it integrates naturally
with editor problem matchers and CI log parsers.

---

## 10. CLI interface

```
USAGE:
    pyguard [OPTIONS] <PATHS>...

ARGS:
    <PATHS>...    Files or directories to lint (directories are walked recursively for .py files)

OPTIONS:
    -q, --quiet       Suppress output; only set exit code
    --format <FMT>    Output format: "text" (default), "json"
    -h, --help        Print help
    -V, --version     Print version
```

Exit codes:
- `0` -- no violations
- `1` -- one or more violations found
- `2` -- fatal error (e.g. all paths invalid)

---

## 11. Performance budget

For a codebase the size of arcane_extension (~50 Python files, ~15k lines):

| Phase             | Expected time |
| ----------------- | ------------- |
| File discovery    | < 5ms         |
| Parse all files   | < 50ms        |
| Rule dispatch     | < 10ms        |
| **Total**         | **< 100ms**   |

The `rayon` parallelism and `ignore`-based walk are designed for much larger
codebases (100k+ files). At our scale they add negligible overhead but
future-proof the tool.

---

## 12. Build order

| Phase | What                                                                      | Tests           |
| ----- | ------------------------------------------------------------------------- | --------------- |
| 1     | Scaffold: `Cargo.toml`, all module stubs, `Diagnostic`, `Config`, empty `lint_source()` | compiles        |
| 2     | Write ALL tests from section 8 (red phase -- rules, config, suppression)  | all fail        |
| 3     | Implement `config.rs` (pyproject.toml discovery + deserialization)         | config tests    |
| 4     | Implement `suppression.rs` (inline ignore parsing)                        | suppression tests |
| 5     | Implement `engine.rs` (parse + walk + dispatch + suppression filter)      | infra works     |
| 6     | Implement Rule 1 (`no-hasattr-getattr`)                                   | R1 tests pass   |
| 7     | Implement Rule 2 (`guarded-function-import`)                              | R2 tests pass   |
| 8     | Implement Rule 3 (`no-future-annotations`)                                | R3 tests pass   |
| 9     | Implement Rule 4 (`no-dataclass`)                                         | R4 tests pass   |
| 10    | Implement Rule 5 (`no-bare-except`)                                       | R5 tests pass   |
| 11    | Implement Rule 6 (`no-print`)                                             | R6 tests pass   |
| 12    | Implement CLI (`main.rs`)                                                 | manual smoke    |
| 13    | Wire into `arcane_extension` CI                                           | CI green        |

---

## 13. CI integration (downstream consumer)

Since `pyguard` is a standalone crate (not inside `arcane_extension`), the
CI workflow installs it by path. Added to `arcane_extension/.github/workflows/ci-fast.yml`:

```yaml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable

- name: Cache pyguard binary
  uses: actions/cache@v4
  with:
    path: ~/.cargo/bin/pyguard
    key: pyguard-${{ hashFiles('pyguard/Cargo.lock') }}

- name: Install pyguard
  run: cargo install --path ../pyguard

- name: Custom lint (pyguard)
  run: pyguard h_arcane arcane_builtins arcane_cli
```

And to `arcane_extension/package.json` for local dev:

```json
"check:be:custom-lint": "cargo run --manifest-path ../pyguard/Cargo.toml --release -- h_arcane arcane_builtins arcane_cli",
"check:be": "pnpm run check:be:lint && pnpm run check:be:type && pnpm run check:be:custom-lint"
```

---

## 14. Rule summary

All rules, their default state, and whether they're opinionated:

| Rule ID                    | Default | Opinionated | Target pattern                          |
| -------------------------- | ------- | ----------- | --------------------------------------- |
| `no-hasattr-getattr`       | ON      | No          | `hasattr()` / `getattr()` calls         |
| `guarded-function-import`  | ON      | No          | Uncommented function-scope imports       |
| `no-future-annotations`    | ON      | No          | `from __future__ import annotations`    |
| `no-dataclass`             | ON      | **Yes**     | `@dataclass` and dataclasses imports    |
| `no-bare-except`           | ON      | No          | `except:` / `except Exception:`         |
| `no-print`                 | ON      | **Yes**     | `print()` calls                         |

"Opinionated" rules are the ones most likely to need `exclude` in specific
projects. They are still ON by default because the default consumer is an
LM-generated codebase where these patterns are almost always wrong.

---

## 15. Future extensibility

Adding a new rule requires exactly three steps:

1. Create `src/rules/new_rule_name.rs` implementing the `Rule` trait.
2. Add `pub mod new_rule_name;` to `src/rules/mod.rs`.
3. Add `Box::new(new_rule_name::NewRuleName)` to the `all_rules()` vec.

No changes to the engine, CLI, config, suppression, or any other rule.
The dispatch map rebuilds automatically. The config exclude-list and inline
suppression work for all rules by rule ID -- no per-rule wiring needed.

Candidate future rules:
- `no-star-import` -- `from module import *` (ruff has `F403` but we may
  want stricter behavior or different messaging)
- `no-mutable-default-arg` -- `def f(x=[]):`
- `no-type-ignore-without-code` -- `# type: ignore` without specific error code
- `no-nested-ternary` -- `a if b else (c if d else e)`
- `no-string-type-annotation` -- `def f(x: "SomeType")` instead of proper forward ref
