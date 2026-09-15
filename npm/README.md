# @atypical/commit

Commit message linting: the `commit-lint` binary, prebuilt.

The binary comes in as an optional dependency,
`@atypical/commit-<platform>-<arch>[-musl]`, with no install scripts.
`COMMIT_LINT_BINARY` points the wrapper at another binary; without one
to run, it exits 127.

## Install

```sh
npm i -D @atypical/commit
```

Usage, exit codes, configuration, and presets are in the
[crate README](https://github.com/kekkon-nexus/atypical/tree/main/crates/atypical-commit#readme).

## License

MIT OR Apache-2.0
