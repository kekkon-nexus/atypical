// Discovery and loading of `atypical.toml`: each tool owns its own
// section schema and deserializes it from here.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

pub const FILE_NAME: &str = "atypical.toml";

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => error.fmt(f),
            Error::Toml(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            Error::Toml(error) => Some(error),
        }
    }
}

/// The nearest [`FILE_NAME`] in `start` or any of its ancestors.
pub fn find(start: impl AsRef<Path>) -> Option<PathBuf> {
    start
        .as_ref()
        .ancestors()
        .map(|dir| dir.join(FILE_NAME))
        .find(|path| path.is_file())
}

/// Deserialize the `[key]` section of a TOML document.
/// A document without the section is `Ok(None)`.
pub fn section<T: DeserializeOwned>(
    document: &str,
    key: &str,
) -> Result<Option<T>, toml::de::Error> {
    let table: toml::Table = document.parse()?;

    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    value.clone().try_into().map(Some)
}

/// Read the file at `path` and deserialize its `[key]` section.
pub fn load<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    key: &str,
) -> Result<Option<T>, Error> {
    let document = std::fs::read_to_string(path).map_err(Error::Io)?;

    section(&document, key).map_err(Error::Toml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Section {
        name: String,
    }

    #[test]
    fn test_section() {
        let document = "[commit]\nname = \"value\"\n";

        assert_eq!(
            section::<Section>(document, "commit").unwrap(),
            Some(Section {
                name: "value".into()
            })
        );
        assert_eq!(section::<Section>(document, "branch").unwrap(), None);
        assert_eq!(section::<Section>("", "commit").unwrap(), None);

        assert!(section::<Section>("not toml", "commit").is_err());
        assert!(section::<Section>("[commit]\nname = 1\n", "commit").is_err());
    }
}
