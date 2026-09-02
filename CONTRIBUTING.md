# Contributing

## Setup

Linters and git hooks come from npm via
[lefthook](https://lefthook.dev/); the Rust toolchain comes from
`rust-toolchain.toml`. Install [bun](https://bun.sh/) and
[rustup](https://rustup.rs/), then:

```sh
bun install
cargo install cargo-nextest --locked
```

The git hooks are installed automatically by lefthook's postinstall.

## Workflow

```sh
bun run check
bun run fix
bun run test:rust
```

The `pre-commit` hook runs the same linters on staged files.

`bun run bench:latency` writes per-invocation latency receipts to
`benches/results.md`; it needs
[hyperfine](https://github.com/sharkdp/hyperfine) on `PATH`.

## Commits

Headers follow the
[Standard Commits](https://github.com/standard-commits/standard-commits)
convention:

```txt
<keyword>[<modifier>][(<scope>)][<reason>]: <description>
```

e.g. `add(exe)[int]: initial commit linting`

The `commit-msg` hook lints them with this repo's own `atypical-commit`.

## CI

Pull requests must pass `cargo fmt --check`, `clippy -D warnings`,
and keep region coverage at 90% or above.
