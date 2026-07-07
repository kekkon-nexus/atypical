# atypical-commit

Commit message linting: the `commit-lint` binary, prebuilt.

Installing this package downloads the binary for your platform from the
matching [GitHub release](https://github.com/kekkon-nexus/atypical/releases)
and verifies its sha256.

## Install

```sh
npm i -D atypical-commit
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
