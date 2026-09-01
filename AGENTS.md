# AGENTS.md

Guidance for AI agents working in this repository. Humans:
see [CONTRIBUTING.md](CONTRIBUTING.md).

## What this is

**Atypical** — a toolkit for enforcing your own conventions, configured
from a single `atypical.toml`. Rust workspace (edition 2024, **nightly**
toolchain, resolver 3) with two crates:

- `crates/atypical-commit` — commit message linting: the `commit-lint`
  binary (`src/main.rs`) plus a chumsky-based parser library
  (`src/lib.rs`), rendering diagnostics with `ariadne`. Without a
  `[commit]` section there is nothing to enforce and every message
  passes. The section schema lives in `src/config.rs`; fields left
  unset are unrestricted (`Tokens::default()`): any keyword, any
  modifiers in either position, any single-symbol separator,
  free-form `(...)`/`[...]` enclosures.
- `crates/atypical-config` — discovery (`find`, walking ancestors for
  `atypical.toml`) and loading (`section`/`load`/`resolve`) of
  `atypical.toml`. Schema-free: each tool owns its own section schema
  and deserializes it from here.

The design principle is **grammar-as-data**: the entire commit syntax
(keywords, modifiers, enclosures, separator, ordering) lives in one
`Tokens` struct, populated from a preset or the `[commit]`
section of `atypical.toml`. Parsers read it at runtime via chumsky's
context (`ExtraContext`), so nothing about the grammar is hardcoded
into parser structure. Preserve this: new syntax features should
extend `Tokens`/`CommitConfig`, not add special-cased parsers.

## Commit messages (you will be linted)

Headers follow [Standard Commits](https://github.com/standard-commits/standard-commits):

```txt
<keyword>[<modifier>][(<scope>)][<reason>]: <description>
```

The standard preset (also this repo's convention, pinned by the root
`atypical.toml`):

- keywords: `add`, `rem`, `ref`, `fix`, `undo`, `release`
- modifiers: `?`, `!`, `!!` (placed before the enclosures)
- scope, in `(...)`: `exe`, `lib`, `test`, `build`, `doc`, `ci`, `cd`
- reason, in `[...]`: `int`, `pre`, `eff`, `rel`, `cmp`, `mnt`, `tmp`,
  `exp`, `sec`, `upg`, `ux`, `pol`, `sty`
- scope must come before reason; both are optional; a space is
  required after `:` and the description must be non-empty

Examples from history: `add(exe)[int]: initial commit linting`,
`fix(ci): generate report when testing`, `ref(doc)[rel]: use workspace
keywords and per crate's categories`. Releases use the plain
`release: vX.Y.Z` form.

The `commit-msg` hook lints with **this repo's own `commit-lint`**
(`cargo run -p atypical-commit -- <msg-file>`), so an invalid header is
rejected locally. Conventional Commits style (`feat:`, `chore:` ...)
will be rejected.

## Toolchain & environment

- Rust **nightly**, pinned by `rust-toolchain.toml` (components:
  clippy, llvm-tools, rustfmt). `.cargo/config.toml` passes nightly
  `-Z` rustflags and expects `clang` + `lld` on Linux/macOS; builds
  fail on stable, or at link time if those are missing.
- Linters come from npm devDependencies (`bun install`): `tombi`
  (TOML), `oxfmt` (YAML/JSON/TS), `v8r` (JSON Schema), `lefthook`.
  `cargo-nextest` and `hyperfine` are not on npm and must be installed
  separately (`cargo install cargo-nextest --locked`).
- [lefthook](https://lefthook.dev/) (`.config/lefthook.yaml`) runs the
  linters and installs the git hooks via its own postinstall. JS tools
  are invoked through `bunx` because lefthook does not put
  `node_modules/.bin` on `PATH`.
- `v8r` resolves schemas from the schemastore.org catalog by filename,
  so no schema is vendored here. It needs network on cache miss (HTTP
  responses are cached for 600s), and `.v8rignore` drops `.vscode/`,
  whose files have no catalog entry and would otherwise fail the run.
  It also runs in CI (`schema` job) to cover unstaged files.

## Commands

| Task | Command |
| --- | --- |
| Lint (check only) | `bun run check` (= `lefthook run check`) |
| Lint + autofix | `bun run fix` (= `lefthook run fix`) |
| Test | `bun run test:rust` (= `cargo nextest run --workspace`) |
| Build release binary | `bun run build:rust` (= `cargo build --release -p atypical-commit`) |
| Latency benchmarks | `bun run bench:latency` (receipts in `benches/results.md`) |

Tests use **cargo-nextest**, not `cargo test`. Without lefthook, the
raw CI equivalents:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo nextest run --workspace
```

## CI gates (all must pass)

From `.github/workflows/ci.yaml`:

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features --locked
   -- -D warnings`
3. Coverage: `cargo llvm-cov nextest ... --fail-under-regions 90` —
   **region coverage must stay at 90% or above**, so new code needs
   tests.

Tags matching `v*` additionally publish to crates.io, cut a GitHub
release, and build `commit-lint` binaries for a six-target matrix
(x86_64/aarch64 across Linux, macOS, and Windows).

## Code style

`rustfmt.toml` is enforced; the non-defaults matter:

- `max_width = 80` (markdown and TOML wrap at ~80 columns to match)
- `merge_derives = false` — keep **separate `#[derive(...)]` lines**
  grouped as the surrounding code does (e.g. `#[derive(Debug, Clone,
  PartialEq)]` on one line, `#[derive(serde::Deserialize)]` on the
  next).
- `group_imports = "StdExternalCrate"`, `imports_granularity = "Module"`
- `use_field_init_shorthand = true`

`.editorconfig`: LF, UTF-8, final newline; 2-space indent everywhere
except `.rs` (4-space).

Conventions visible in the code:

- Dependencies are declared in `[workspace.dependencies]` with
  `default-features = false`; each crate re-enables exactly the
  features it needs. Version and metadata come from
  `[workspace.package]` — new crates should use
  `version.workspace = true` etc.
- `atypical-commit` gates functionality behind features: `std`, `cli`,
  `color` (all default). Keep new code compiling across feature
  combinations (CI runs `--all-features`; the library avoids baking in
  `std`-only conveniences).
- Errors: library crates define their own `Error` enums with `Display` +
  `std::error::Error` impls; `anyhow` is used only at the binary
  boundary.
- Comments are sparse and explain intent/constraints, not mechanics.

## Behavior contracts (do not break)

- `commit-lint` exit codes: `0` valid, `1` failed linting or
  unreadable input, `2` usage error / nothing to lint (clap also
  uses 2). The integration tests assert these.
- Header extraction mimics git: leading blank lines and `#` comment
  lines are skipped; the first remaining line is the header
  (`message_header` in `main.rs`). CRLF is tolerated.
- The preset files in `presets/` (`standard.toml`, `conventional.toml`)
  are meant to be targeted by `extends`; `tests/presets.rs` in
  `atypical-commit` pins `standard.toml` to `Tokens::preset_standard()`
  and the headers each preset accepts — keep file and code in sync.
- A top-level `extends` key (a path or an array of paths, relative to
  the extending file) is resolved by `atypical-config` before section
  lookup: extended documents apply one by one in declaration order,
  the extending file last; tables merge key-by-key, any other value
  replaces the one beneath it. Cycles and non-path values are errors
  (`Error::Cycle` / `Error::Extends`).
- Config semantics: no `[commit]` section means nothing is linted
  (exit 0 for any message); a declared section defaults *field by
  field* to unrestricted (`#[serde(default)]` on `CommitConfig`);
  unknown keys are rejected (`deny_unknown_fields`); an enclosure
  without `allowed` is flexible (anything between the delimiters);
  `keywords`, `modifiers`, `separator`, and `modifier-sequence`
  accept the literal string `"any"`.
- Enclosure order is positional: each `[[commit.enclosures]]` entry
  may appear at most once, in declaration order.
- Machine-generated headers — merges, reverts, `fixup!`/`squash!`/
  `amend!`, semver release bumps — exit 0 without linting
  (`src/ignore.rs`, mirroring commitlint's default ignores) unless
  `default-ignores = false` is set in `[commit]`.
- `ExtraContext::new` sorts keywords/modifiers longest-first so that
  e.g. `!!` wins over `!`. Any new token class with overlapping
  prefixes needs the same treatment.

## Testing conventions

- Unit tests live in-file under `#[cfg(test)] mod tests`; parser
  tests bind the preset via
  `.with_ctx(Tokens::preset_standard().into())`.
- Integration tests live in each crate's `tests/` (`cli.rs`,
  `load.rs`): `cli.rs` drives the real binary through
  `env!("CARGO_BIN_EXE_commit-lint")` and writes fixtures to
  `env!("CARGO_TARGET_TMPDIR")` — no fixture files are committed.
- nextest is configured (`.config/nextest.toml`) to emit
  `target/nextest/default/junit.xml` for CI's Codecov upload.

## Gotchas

- The toolchain is pinned to **nightly**; don't "fix" builds by
  switching to stable.
- TOML is formatted by **tombi** and YAML/JSON by **oxfmt** via
  lefthook — run `bun run fix` after editing config/workflow files
  rather than hand-styling them.
- oxfmt also formats TOML and Markdown, and disagrees with tombi and
  with the committed Markdown, so its globs are scoped to
  `yml,yaml,json,ts` on purpose. Don't widen them to `.`.
- lefthook only finds a `.config/` config named `lefthook.yaml`;
  `.config/lefthook.yml` is silently ignored.
- Tool configs are gathered under `.config/` (lefthook, nextest,
  tombi) rather than the repo root.
- `benches/` is not `cargo bench`: it is a POSIX-sh hyperfine harness
  (`latency.sh`) comparing against a vendored commitlint.
  `benches/node_modules` exists only after
  `npm --prefix benches install`; it's optional (the commitlint lane
  is skipped without it).
- The release profile in `.cargo/config.toml` is size-tuned
  (`codegen-units = 1`, `lto = "fat"`, `opt-level = "z"`,
  `strip = "symbols"`, `panic = "abort"`). These are deliberate,
  benchmarked choices — do not "fix" them.
