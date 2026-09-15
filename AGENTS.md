# AGENTS.md

Humans: [CONTRIBUTING.md](CONTRIBUTING.md). User-facing behavior is in
the crate READMEs.

## Layout

Rust workspace on nightly, two crates:

- `crates/atypical-commit`: `commit-lint` (`src/main.rs`, with
  `src/range.rs` behind `--from`/`--to`) and a chumsky parser
  (`src/lib.rs`). `src/config.rs` is the `[commit]` schema, lowered into
  `Tokens`; `src/ignore.rs` holds the default ignores.
- `crates/atypical-config`: `find`, `load`, `resolve`, `section`. Owns
  `extends` and named-array merging; knows no section schema.

The grammar is data: `Tokens` is an ordered slot list that `prefix()`
walks from chumsky's context. New syntax extends `Slot`/`CommitConfig`,
never a special-cased parser.

## Commits

The `commit-msg` hook lints them. Read
[CONTRIBUTING.md#commits](CONTRIBUTING.md#commits) before committing.

## Toolchain

- `.cargo/config.toml` passes `-Z` flags: stable fails, don't switch to
  it. x86_64 Linux and macOS link with `clang` and `lld`.
- `bun install` brings tombi, oxfmt, oxlint, v8r and lefthook, and
  installs the hooks. `cargo-nextest` and `hyperfine` come separately.
- Hooks call JS tools through `bunx`: lefthook doesn't put
  `node_modules/.bin` on `PATH`.
- v8r fetches schemas from schemastore.org by filename, so a cold cache
  needs network. `.v8rignore` drops `.vscode/`, which has no catalog
  entry.

## Commands

| Task          | Command                 |
| ------------- | ----------------------- |
| Lint          | `bun run check`         |
| Lint, autofix | `bun run fix`           |
| Test          | `bun run test`          |
| Release build | `bun run build:rust`    |
| Bench         | `bun run bench:latency` |

Tests run under nextest, with `cargo test --doc` for doctests. CI gates
on `bun run check` and `bun run test` under `cargo llvm-cov` with
`--fail-under-regions 90`, so new code needs tests. A `v*` tag runs
`publish.yaml`, which reruns CI first.

## Style

- `merge_derives = false`: in `src/`, std derives share one line and
  `Deserialize`/`Parser` take the next.
- Workspace dependencies with default features turn them off; each
  crate enables what it uses. Crates inherit `[workspace.package]` fields.
- The `std`, `cli` and `color` features only forward dependency
  features; nothing is `cfg`-gated.
- Library errors are enums implementing `Display` and `Error`; `anyhow`
  stays in the binary.

## Contracts

Each is pinned by tests; change a test only on purpose.

- Exit codes: `Exit` and `after_help` in `main.rs` (`tests/cli.rs`).
- The header is the first line neither blank nor a `#` comment; CRLF is
  tolerated (`message_header`).
- No `[commit]` section lints nothing, and a range then never runs git.
  A section without `slots` lowers to `Tokens::default()`. Unknown keys
  are rejected, the removed fixed-layout keys included, with no
  migration code.
- `presets/*.toml` are the only preset definitions; `tests/presets.rs`
  pins accepted headers and every error span and message.
- `extends`, `drop`, `before` and duplicate names behave as documented
  on `atypical_config::resolve` (`tests/load.rs`). `name`, `drop` and
  `before` are reserved in every named array, for any tool.
- Unlowerable slots are `Invalid` (`config.rs`); indistinguishable ones
  are `Ambiguous` (`ambiguity` in `lib.rs`). Bare sets are tried longest
  first; delimited sets keep declaration order, which diagnostics list.

## Tests

- Unit tests live in-file. `tests/presets.rs` holds parser behavior as
  header and error rows built from `CommitConfig`, never `Tokens`
  fields.
- `tests/cli.rs` runs the real binary against fixtures and throwaway
  repositories under `CARGO_TARGET_TMPDIR`; none are committed.

## Gotchas

- tombi owns TOML; oxfmt owns everything else it supports, Markdown
  included, so its jobs exclude `*.toml`. Run `bun run fix` instead of
  hand-styling.
- lefthook reads `.config/lefthook.yaml` only; `.yml` is silently
  ignored.
- The size-tuned release profile in `.cargo/config.toml` is deliberate.
- `benches/` is a hyperfine script, not `cargo bench`. Its commitlint
  lane needs the `benches` workspace installed and is skipped without
  it.
