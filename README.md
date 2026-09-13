# Atypical

> (Non)standard enforcing DX

[![CI](https://github.com/kekkon-nexus/atypical/actions/workflows/ci.yaml/badge.svg)](https://github.com/kekkon-nexus/atypical/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/github/kekkon-nexus/atypical/graph/badge.svg?token=C2ZID0WFZZ)](https://codecov.io/github/kekkon-nexus/atypical)
[![crates.io](https://img.shields.io/crates/v/atypical-commit)](https://crates.io/crates/atypical-commit)
[![docs.rs](https://img.shields.io/docsrs/atypical-commit)](https://docs.rs/atypical-commit)

A toolkit for enforcing your own conventions.

## Crates

- [`atypical-commit`](crates/atypical-commit) — commit message linting.
  Ships the `commit-lint` binary and a parser library.
- [`atypical-config`](crates/atypical-config) — finds and loads
  `atypical.toml`. Each tool reads its own section.

## Configuration

By default, `commit-lint` doesn't lint anything without an `atypical.toml`
or without a `[commit]` section. Unset fields stay unrestricted.

> [!TIP]
> This project bootstraps its own commit linter! Check our
> [`atypical.toml`](atypical.toml).

Available configuration in `[commit]`:

| Key               | Explanation                            | Values                    |
| ----------------- | -------------------------------------- | ------------------------- |
| `slots[]`         | The grammar, as `[[commit.slots]]`     | Tables, in header order   |
| `default-ignores` | Skips merge, revert, and fixup commits | `true` (default), `false` |

The whole grammar is the slot list: one entry per part of the header,
in the order they appear. Each entry takes:

| Field        | Explanation                         | Values                                  |
| ------------ | ----------------------------------- | --------------------------------------- |
| `name`       | Labels the slot in errors           | Any string                              |
| `kind`       | What an undelimited slot is made of | `"word"`, `"symbols"`, `"symbol"`       |
| `delimiters` | What a delimited slot sits between  | Pair of strings, eg `["(", ")"]`        |
| `values`     | Accepted spellings                  | `"any"` (default), or a list of strings |
| `required`   | Whether a header may leave it out   | `false` (default), `true`               |

A slot is `kind` or `delimiters`, never both. Order is position, so
where a slot sits in the list is where it sits in the header:

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

### Coming from the fixed-layout keys

They are gone; each is a slot now.

| Was                 | Now                                                |
| ------------------- | -------------------------------------------------- |
| `keywords`          | A `kind = "word"` slot's `values`                  |
| `modifiers`         | A `kind = "symbols"` slot's `values`               |
| `modifier-sequence` | Where the modifier slot sits in the list           |
| `separator`         | A `kind = "symbol"` slot's `values`                |
| `enclosures[]`      | One slot per enclosure, `delimiters` plus `values` |

## Presets

Ready-made `[commit]` sections live in [`presets/`](presets):

- [`standard.toml`](presets/standard.toml) —
  [Standard Commits](https://github.com/standard-commits/standard-commits).
- [`conventional.toml`](presets/conventional.toml) —
  [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

> [!NOTE]
> Currently, there isn't a way to use this preset in your project
> automatically. You may copy one into your project, or vendor this
> repository.

To use:

```toml
extends = "conventional.toml"

# Narrows the preset's keyword slot; every other slot is left alone.
[[commit.slots]]
name = "keywords"
values = ["feat", "fix", "docs"]
```

`extends` also takes an array of paths. They apply in order and can be
overridden by setting custom configuration locally. Slots are matched
by `name`, so adjusting one does not mean restating the rest; an
unmatched name adds a slot at the end, and `drop = true` removes the
one it names.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup and
workflow. This project follows the
[Contributor Covenant](CODE_OF_CONDUCT.md) code of conduct.

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
