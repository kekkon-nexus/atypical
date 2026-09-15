# atypical-commit

[![crates.io](https://img.shields.io/crates/v/atypical-commit)](https://crates.io/crates/atypical-commit)
[![docs.rs](https://img.shields.io/docsrs/atypical-commit)](https://docs.rs/atypical-commit)

Commit message linting: the `commit-lint` binary and its parser library.

## Install

```sh
cargo install atypical-commit
```

Prebuilt, [from npm](https://www.npmjs.com/package/@atypical/commit):

```sh
npm i -D @atypical/commit
```

## Usage

```sh
commit-lint -- .git/COMMIT_EDITMSG
echo 'add(lib)[int]: something' | commit-lint -
commit-lint --from origin/main
```

`--from` and `--to` lint every commit in `from..to` instead of an input.
`to` defaults to `HEAD`; without `from`, the whole history reachable
from `to` is linted.

Only the header is linted: the first line that is neither blank nor a
`#` comment.

| Exit | Meaning                                                  |
| ---- | -------------------------------------------------------- |
| `0`  | Valid, or no `[commit]` section to lint against          |
| `1`  | Failed linting, an empty message in a range, or an error |
| `2`  | Usage error, or no commit message to lint                |

As a `commit-msg` hook, in `.husky/commit-msg` or an executable
`.git/hooks/commit-msg` starting with `#!/bin/sh`:

```sh
commit-lint -- "$1"
```

## Configuration

The `[commit]` section of the nearest `atypical.toml` from the working
directory upward, or of `--config <FILE>`. Without one, nothing is
linted.

The grammar is `[[commit.slots]]`, one entry per part of the header, in
header order:

| Field        | Value                                              |
| ------------ | -------------------------------------------------- |
| `name`       | Any string; errors and merges refer to it          |
| `kind`       | `"word"`, `"symbols"`, or `"symbol"`               |
| `delimiters` | A pair of characters, eg `["(", ")"]`              |
| `values`     | `"any"` (default), or a list of accepted spellings |
| `required`   | `false` (default), or `true`                       |

A slot has `kind` or `delimiters`, never both. A `word` is a run of
alphanumerics and `_`, `symbols` a run of other visible characters,
`symbol` exactly one. A delimited slot holds a word from `values`, or
anything but its delimiters when unrestricted.

```toml
[[commit.slots]]
name = "keyword"
kind = "word"
values = ["feat", "fix"]
required = true

[[commit.slots]]
name = "scope"
delimiters = ["(", ")"]

[[commit.slots]]
name = "separator"
kind = "symbol"
values = [":"]
required = true
```

A section without `slots` still enforces the shape: a word, optional
`(...)` and `[...]`, optional symbols, a separator symbol, then a space
and a description.

Slots that cannot be told apart are rejected before any header is read:
a word after a word, more symbols after unrestricted `symbols`,
neighbouring spellings sharing a prefix, a closed set in front of an
unrestricted slot of the same alphabet, or two slots with the same
delimiters. An optional slot does not separate its neighbours.

`default-ignores` sits beside the slots; see [Default
ignores](#default-ignores).

### Presets

[`presets/`](https://github.com/kekkon-nexus/atypical/tree/main/presets)
holds `standard.toml`
([Standard Commits](https://github.com/standard-commits/standard-commits))
and `conventional.toml`
([Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)).
They cannot be referenced remotely yet: copy one in, or vendor the
repository.

`extends` takes a path or an array of paths, relative to the extending
file, applied in order with the extending file last. Tables merge by
key; `[[commit.slots]]` merges by `name`:

```toml
extends = "conventional.toml"

[[commit.slots]]
name = "keywords"
values = ["feat", "fix", "docs"]

[[commit.slots]]
name = "ticket"
delimiters = ["[", "]"]
before = "separator"
```

- A matched `name` merges field by field; an unmatched one appends.
- `drop = true` removes the entry it names.
- `before = "<name>"` places the entry ahead of the one named, moving it
  if already present. Naming no other entry is an error.
- Two entries sharing a `name` in one file is an error.

### From the fixed-layout keys

These are removed and now rejected as unknown keys.

| Was                 | Now                                                |
| ------------------- | -------------------------------------------------- |
| `keywords`          | A `kind = "word"` slot's `values`                  |
| `modifiers`         | A `kind = "symbols"` slot's `values`               |
| `modifier-sequence` | Where the modifier slot sits in the list           |
| `separator`         | A `kind = "symbol"` slot's `values`                |
| `enclosures[]`      | One slot per enclosure, `delimiters` plus `values` |

### Default ignores

Unless `default-ignores = false`, headers that git and forges generate
pass unlinted, as in [commitlint](https://commitlint.js.org/reference/configuration.html#defaultignores):

- merges: `Merge pull request ...`, `Merge branch '...'`,
  `Merge tag '...'`, `Merge x into y`,
  `Merge remote-tracking branch '...'`, `Merged x in(to) y`,
  `Merged PR 1: ...`, `Automatic merge ...`, `Auto-merged x into y`
- reverts and reapplies: `Revert ...`, `Reapply ...`
- autosquash markers: `fixup! ...`, `squash! ...`, `amend! ...`
- release bumps: a semver version, optionally behind `chore:` or
  `chore(<scope>):` and a `[skip ci]`-style marker

## License

MIT OR Apache-2.0
