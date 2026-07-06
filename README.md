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
