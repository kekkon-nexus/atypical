# atypical-config

[![crates.io](https://img.shields.io/crates/v/atypical-config)](https://crates.io/crates/atypical-config)
[![docs.rs](https://img.shields.io/docsrs/atypical-config)](https://docs.rs/atypical-config)

Discovery and loading of `atypical.toml`. Schema-free: each tool
deserializes its own section.

## Usage

```rust
#[derive(serde::Deserialize)]
struct Section {
    enabled: bool,
}

if let Some(path) = atypical_config::find(std::env::current_dir()?) {
    let section: Option<Section> = atypical_config::load(path, "tool")?;
}
```

`find` walks up from a directory. `load` resolves `extends`, then
deserializes the section, `None` when absent. `resolve` stops at the
merged table; `section` parses a document as written.

Inside an array whose every entry has a `name`, the keys `name`, `drop`
and `before` are merge directives, consumed before your schema sees
them. The rules are on
[`resolve`](https://docs.rs/atypical-config/latest/atypical_config/fn.resolve.html).

## License

MIT OR Apache-2.0
