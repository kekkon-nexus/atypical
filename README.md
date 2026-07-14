# Atypical

> (Non)-standard enforcing DX

[![CI](https://github.com/kekkon-nexus/atypical/actions/workflows/ci.yaml/badge.svg)](https://github.com/kekkon-nexus/atypical/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/github/kekkon-nexus/atypical/graph/badge.svg?token=C2ZID0WFZZ)](https://codecov.io/github/kekkon-nexus/atypical)
[![crates.io](https://img.shields.io/crates/v/atypical-commit)](https://crates.io/crates/atypical-commit)
[![docs.rs](https://img.shields.io/docsrs/atypical-commit)](https://docs.rs/atypical-commit)

A toolkit for enforcing your own conventions, configured from a single
`atypical.toml`.

## Crates

- [`atypical-commit`](crates/atypical-commit) — commit message linting,
  as the `commit-lint` binary and a parser library.
- [`atypical-config`](crates/atypical-config) — discovery and loading
  of `atypical.toml`; each tool owns its own section.

## Presets

Without configuration, `commit-lint` is as lax as possible: any
keyword, any modifiers on either side of free-form `(...)`/`[...]`
enclosures, any single-symbol separator — only the header shape
itself is enforced. Presets tighten it.

Ready-made `[commit]` sections live in [`presets/`](presets):

- [`standard.toml`](presets/standard.toml) —
  [Standard Commits](https://github.com/standard-commits/standard-commits).
- [`conventional.toml`](presets/conventional.toml) —
  [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

Copy one next to your project (or vendor this repository) and point
the top-level `extends` key of `atypical.toml` at it; anything you
set locally overrides the preset key by key:

```toml
extends = "conventional.toml"

[commit]
keywords = ["feat", "fix", "docs"]
```

`extends` also takes an array of paths, applied one by one in order,
with the extending file last.

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
