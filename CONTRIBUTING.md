# Contributing

## Setup

Install [bun](https://bun.sh/) and [rustup](https://rustup.rs/), then:

```sh
bun install
cargo install cargo-nextest --locked
```

`bun install` also installs the git hooks. On x86_64 Linux and macOS,
linking needs `clang` and `lld`.

## Workflow

```sh
bun run check
bun run fix
bun run test
```

`pre-commit` runs the fixers on staged files and restages what they
change. `commit-msg` lints the message with this repository's own
`commit-lint`.

`bun run bench:latency` needs
[hyperfine](https://github.com/sharkdp/hyperfine) and writes
`benches/results.md`.

## Commits

Headers follow [Standard Commits](https://github.com/standard-commits/standard-commits):

Each commit MUST have a `<verb>` and a `<summary>` but all the other fields are present on a case-by-case basis.

Syntax Specification:

```bnf
<verb><importance?>(<scope?>)[<reason?>]: <summary>

<body?>

<footer?>
```

| 🔊 verb               | ⚠️ importance             | 🔖 scope                      | 💡 reason                |
| --------------------- | ------------------------- | ----------------------------- | ------------------------ |
| `add` (_add_)         | `?` (_possibly breaking_) | `exe` (_executable_)          | `int` (_introduction_)   |
| `rem` (_remove_)      | `!` (_breaking_)          | `lib` (_backend library_)     | `pre` (_preliminary_)    |
| `ref` (_refactor_)    | `!!`(_critical_)          | `test` (_testing_)            | `eff` (_efficiency_)     |
| `fix` (_fix_)         |                           | `build` (_building_)          | `rel` (_reliability_)    |
| `undo` (_undo_)       |                           | `doc` (_documentation_)       | `cmp` (_compatibility_)  |
| `release` (_release_) |                           | `ci` (continuous integration) | `mnt` (_maintenance_)    |
|                       |                           | `cd` (continuous delivery)    | `tmp` (_temporary_)      |
|                       |                           |                               | `exp` (_experiment_)     |
|                       |                           |                               | `sec` (_security_)       |
|                       |                           |                               | `upg` (_upgrade_)        |
|                       |                           |                               | `ux` (_user experience_) |
|                       |                           |                               | `pol` (_policy_)         |
|                       |                           |                               | `sty` (_styling_)        |

| 📝 summary                                                  | ℹ️ body                                                   | ⚙️ footer                                        |
| ----------------------------------------------------------- | --------------------------------------------------------- | ------------------------------------------------ |
| Starts with a _lowercase letter_                            | Starts with an _uppercase letter_                         | Each tag on a new line, format: `<key>: <value>` |
| _Concise_ and _descriptive_ of what the change does         | Expands on _why_ and _how_, not what (already in summary) | MUST be separated from body by a blank line      |
| MUST _not repeat_ info from the structured fragment         | Organized in _short_, _clear_ paragraphs                  | `Breaking:` ─ describe breaking changes          |
| ≤ _50 UTF-8 characters_ (_excluding_ the structured prefix) | Written in _imperative mood_                              | `Fixes: #N` ─ closes referenced issues           |
| SHOULD use a subset of Markdown                             | SHOULD use a subset of Markdown                           | `Co-authored-by:` ─ attributes co-authorship     |

Example:

```txt
add!(lib/type-check)[rel]: enforce type checking in function calls

Previously, the semantic analyzer allowed mismatched parameter types
in function calls, leading to runtime errors. This fix implements
strict type validation during the semantic analysis phase.

Breaking: The `validateCall` function now returns `TypeMismatchError`
  instead of returning a boolean, requiring updates in error handling.
Fixes: #247
Co-authored-by: Foo Bar <foo.bar@compiler.dev>
```

## CI

Pull requests must pass `bun run check` and keep region coverage at 90%
or above.

## Releasing

```sh
scripts/release.sh <major|minor|patch|X.Y.Z>
```

It needs `cargo set-version` (cargo-edit) and `jq`. Commit as
`release: v<version>` and push a `v<version>` tag; `publish.yaml` does
the rest.
