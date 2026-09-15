# Atypical

> (Non)standard enforcing DX

[![CI](https://github.com/kekkon-nexus/atypical/actions/workflows/ci.yaml/badge.svg)](https://github.com/kekkon-nexus/atypical/actions/workflows/ci.yaml)
[![codecov](https://codecov.io/github/kekkon-nexus/atypical/graph/badge.svg?token=C2ZID0WFZZ)](https://codecov.io/github/kekkon-nexus/atypical)
[![crates.io](https://img.shields.io/crates/v/atypical-commit)](https://crates.io/crates/atypical-commit)
[![docs.rs](https://img.shields.io/docsrs/atypical-commit)](https://docs.rs/atypical-commit)

A toolkit for enforcing your own conventions, configured from one
`atypical.toml` in which each tool reads its own section.

## Crates

- [`atypical-commit`](crates/atypical-commit): commit message linting.
  Usage, configuration, and presets are in its README.
- [`atypical-config`](crates/atypical-config): finds and loads
  `atypical.toml`.

This repository lints its own commits; see [`atypical.toml`](atypical.toml).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). This project follows the
[Contributor Covenant](CODE_OF_CONDUCT.md) code of conduct.

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.
