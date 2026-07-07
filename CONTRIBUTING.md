# Contributing

## Setup

This project uses [mise](https://mise.jdx.dev/) for the toolchain and
[hk](https://hk.jdx.dev/) for linting and git hooks. Install mise, then:

```sh
mise install
```

The git hooks are installed automatically.

## Workflow

```sh
hk check
hk fix
mise run test:rust
```

The `pre-commit` hook runs `hk fix` on staged files.

`mise run bench:latency` writes per-invocation latency receipts to
`benches/results.md`; `npm --prefix benches install` first to include
the commitlint head-to-head.

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
