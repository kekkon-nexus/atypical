# atypical-commit

[![crates.io](https://img.shields.io/crates/v/atypical-commit)](https://crates.io/crates/atypical-commit)
[![docs.rs](https://img.shields.io/docsrs/atypical-commit)](https://docs.rs/atypical-commit)

Commit message linting: a parser library and the `commit-lint` binary.

## `commit-lint`

Lints the commit message header against
`<keyword>[<modifier>][(<scope>)][<reason>]: <description>`,
e.g. `add(exe)[int]: initial commit linting`.

### Install

```sh
cargo install atypical-commit
```

Or as a prebuilt binary, [from npm](https://www.npmjs.com/package/@atypical/commit):

```sh
npm i -D @atypical/commit
```

### Usage

Pass a commit message file, or `-` to read from stdin:

```sh
commit-lint -- .git/COMMIT_EDITMSG
echo 'add(lib)[int]: something' | commit-lint -
```

Exit codes: `0` valid, `1` failed linting (or unreadable input),
`2` usage error or nothing to lint.

### Configuration

Every part of the syntax comes from the `[commit]` section of the
nearest `atypical.toml`, found from the working directory upward
(or passed with `--config <FILE>`). If no config is found, or keys are
omitted, the standard preset is used (shown here in full):

```toml
[commit]
keywords = ["add", "rem", "ref", "fix", "undo", "release"]
modifiers = ["?", "!", "!!"]
separator = ":"
modifier-sequence = "pre" # before the enclosures, or "post"
default-ignores = true    # skip machine-generated headers

# Scopes; omit `allowed` to accept anything between the delimiters.
[[commit.enclosures]]
delimiters = ["(", ")"]
allowed = ["exe", "lib", "test", "build", "doc", "ci", "cd"]

# Reasons.
[[commit.enclosures]]
delimiters = ["[", "]"]
allowed = [
  "int", "pre", "eff", "rel", "cmp", "mnt", "tmp",
  "exp", "sec", "upg", "ux", "pol", "sty",
]
```

### Default ignores

Headers that git and forges generate on their own are exempt from
linting, mirroring [commitlint's default
ignores](https://commitlint.js.org/reference/configuration.html#defaultignores):

- merges: `Merge pull request ...`, `Merge branch '...'`,
  `Merge tag '...'`, `Merge x into y`,
  `Merge remote-tracking branch '...'`, `Merged x in(to) y`,
  `Merged PR 1: ...`, `Automatic merge ...`, `Auto-merged x into y`
- reverts and reapplies: `Revert "..."`, `Reapply "..."`
- autosquash markers: `fixup! ...`, `squash! ...`, `amend! ...`
- release bumps: a semver version, optionally behind a `chore:`
  prefix and a `[skip ci]`-style marker, e.g. `chore(release): v1.2.3`

Set `default-ignores = false` in the `[commit]` section to lint these
like any other header.

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

MIT OR Apache-2.0
