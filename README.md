# Atypical

> (Non)-standard enforcing DX

## `commit-lint`

Lints the commit message header against
`<keyword>[<modifier>][(<scope>)][<reason>]: <description>`,
e.g. `add(exe)[int]: initial commit linting`.

### Install

```sh
cargo install --path crates/atypical-commit
```

### Usage

Pass a commit message file, or `-` to read from stdin:

```sh
commit-lint -- .git/COMMIT_EDITMSG
echo 'add(lib)[int]: something' | commit-lint -
```

Exit codes: `0` valid, `1` failed linting (or unreadable input),
`2` usage error or nothing to lint.

### As a `commit-msg` hook

With [husky](https://typicode.github.io/husky/), in `.husky/commit-msg`:

```sh
commit-lint -- "$1"
```

Or as a plain git hook, in `.git/hooks/commit-msg` (mark it executable):

```sh
#!/bin/sh
commit-lint -- "$1"
```

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
