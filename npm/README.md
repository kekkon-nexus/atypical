# @atypical/commit

Commit message linting: the `commit-lint` binary, prebuilt.

The binary for your platform comes in as an optional dependency
(`@atypical/commit-<platform>-<arch>`); nothing is downloaded at
install time and there are no install scripts.

## Install

```sh
npm i -D @atypical/commit
```

## Usage

With [husky](https://typicode.github.io/husky/), in `.husky/commit-msg`:

```sh
commit-lint -- "$1"
```

Exit codes: `0` valid, `1` failed linting (or unreadable input),
`2` usage error or nothing to lint.

Syntax, configuration (`atypical.toml`), and the standard preset are
documented in the
[crate README](https://github.com/kekkon-nexus/atypical/tree/main/crates/atypical-commit#readme).

## License

MIT OR Apache-2.0
