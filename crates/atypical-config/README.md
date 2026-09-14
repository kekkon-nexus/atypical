# atypical-config

[![crates.io](https://img.shields.io/crates/v/atypical-config)](https://crates.io/crates/atypical-config)
[![docs.rs](https://img.shields.io/docsrs/atypical-config)](https://docs.rs/atypical-config)

Discovery and loading of `atypical.toml`. This crate is schema-free:
each tool owns its own section and deserializes it from here. It is
not key-free, though: inside an array whose entries all carry a `name`,
`name`, `drop` and `before` are merge directives the crate consumes.

## Usage

```rust
#[derive(serde::Deserialize)]
struct CommitConfig {
    keywords: Vec<String>,
}

// The nearest atypical.toml, from `start` upward.
let path = atypical_config::find(std::env::current_dir()?);

// The `[commit]` section of it; `None` when the section is absent.
if let Some(path) = path {
    let config: Option<CommitConfig> =
        atypical_config::load(path, "commit")?;
}
```

`section` does the same on an already-read document.

## License

MIT OR Apache-2.0
