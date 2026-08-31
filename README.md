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

| Key                 | Explanation                                      | Values                        | Eg                         |
| ------------------- | ------------------------------------------------ | ----------------------------- | -------------------------- |
| `keywords`          | Allowed keywords                                 | List of strings               | `feat`, `wip`, `create`    |
| `modifiers`         | Allowed modifier symbols                         | List of strings               | `!`, `*`, `+`              |
| `modifier-sequence` | Modifier position, before or after enclosures    | `"pre"`, `"post"`, `"any"`    | `feat!(api)`, `feat(api)!` |
| `separator`         | Symbol between header and subject                | Single-symbol string, `"any"` | `:`, `-`, `/`              |
| `default-ignores`   | Skips merge, revert, and fixup commits           | `true` (default), `false`     | —                          |
| `enclosures[]`      | Enclosures, as `[[commit.enclosures]]`           | Table: `delimiters` + optional `allowed` | `delimiters = ["(", ")"]` |

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

[commit]
keywords = ["feat", "fix", "docs"]
```

`extends` also takes an array of paths. They apply in order and can be
overridden by setting custom configuration locally.

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
