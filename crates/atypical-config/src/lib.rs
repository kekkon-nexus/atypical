//! Discovery and loading of `atypical.toml`. Each tool owns its section
//! schema; a top-level `extends` layers other files beneath, see
//! [`resolve`].
//!
//! Schema-free but not key-free: within a named array, `name`, `drop`
//! and `before` are directives consumed before any schema sees them.

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

pub const FILE_NAME: &str = "atypical.toml";

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Toml(toml::de::Error),
    /// A document names itself, directly or indirectly, in `extends`.
    Cycle(PathBuf),
    /// `extends` is not a path or an array of paths.
    Extends(PathBuf),
    /// A `before` names no entry to sit ahead of, or names its own.
    Before(PathBuf, String),
    /// One array gives the same `name` to two of its entries.
    Duplicate(PathBuf, String),
}

/// An [`Error`] from within a document, before the document is known.
enum Conflict {
    Before(String),
    Duplicate(String),
}

impl Conflict {
    fn at(self, path: &Path) -> Error {
        let path = path.to_path_buf();

        match self {
            Conflict::Before(name) => Error::Before(path, name),
            Conflict::Duplicate(name) => Error::Duplicate(path, name),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => error.fmt(f),
            Error::Toml(error) => error.fmt(f),
            Error::Cycle(path) => {
                write!(f, "cyclic `extends` involving {}", path.display())
            }
            Error::Extends(path) => write!(
                f,
                "`extends` in {} must be a path or an array of paths",
                path.display()
            ),
            Error::Before(path, name) => write!(
                f,
                "`before = \"{name}\"` in {} names no other entry",
                path.display()
            ),
            Error::Duplicate(path, name) => write!(
                f,
                "two entries named \"{name}\" in one array in {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            Error::Toml(error) => Some(error),
            Error::Cycle(_)
            | Error::Extends(_)
            | Error::Before(..)
            | Error::Duplicate(..) => None,
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

/// Deserialize the `[key]` section of a TOML document as written: no
/// `extends`, no directives. A document without the section is
/// `Ok(None)`.
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

/// Parse the file at `path` into a table, resolving its top-level
/// `extends` key (a path or an array of paths, relative to the
/// extending file). Extended documents are applied one by one in
/// declaration order, the extending document last: tables merge
/// key-by-key, arrays whose every entry carries a `name` merge by that
/// name, any other value replaces the one beneath it.
///
/// A named entry matching one beneath it merges into it field by field;
/// an unmatched one appends, keeping base order; `drop = true` removes
/// the entry it names; `before = "other"` places the entry ahead of the
/// one named, moving it if it was already there. A directive of the
/// wrong type is left in place for the section schema to reject.
///
/// A `before` that names no other entry is an error rather than a
/// silent append or no-op: the position asked for is part of the
/// grammar, so anywhere else is a different grammar. Two entries of
/// one array sharing a `name` is an error for the same reason: the
/// second would merge into the first instead of being an entry of its
/// own.
pub fn resolve(path: impl AsRef<Path>) -> Result<toml::Table, Error> {
    resolve_into(path.as_ref(), &mut Vec::new())
}

fn resolve_into(
    path: &Path,
    stack: &mut Vec<PathBuf>,
) -> Result<toml::Table, Error> {
    let path = path.canonicalize().map_err(Error::Io)?;

    if stack.contains(&path) {
        return Err(Error::Cycle(path));
    }

    let document = std::fs::read_to_string(&path).map_err(Error::Io)?;
    let mut table: toml::Table = document.parse().map_err(Error::Toml)?;

    let bases = match table.remove("extends") {
        None => Vec::new(),
        Some(toml::Value::String(base)) => vec![base],
        Some(toml::Value::Array(bases)) => bases
            .into_iter()
            .map(|base| match base {
                toml::Value::String(base) => Ok(base),
                _ => Err(Error::Extends(path.clone())),
            })
            .collect::<Result<_, _>>()?,
        Some(_) => return Err(Error::Extends(path)),
    };

    let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();

    stack.push(path.clone());

    let mut merged = toml::Table::new();

    for base in bases {
        let base = resolve_into(&dir.join(base), stack)?;

        merge(&mut merged, base).map_err(|it| it.at(&path))?;
    }

    stack.pop();
    merge(&mut merged, table).map_err(|it| it.at(&path))?;

    Ok(merged)
}

fn merge(base: &mut toml::Table, layer: toml::Table) -> Result<(), Conflict> {
    for (key, value) in layer {
        match (base.get_mut(&key), value) {
            (Some(base), value) => merge_value(base, value)?,
            (None, value) => {
                base.insert(key, normalised(value)?);
            }
        }
    }

    Ok(())
}

fn merge_value(
    base: &mut toml::Value,
    layer: toml::Value,
) -> Result<(), Conflict> {
    match (base, layer) {
        (toml::Value::Table(base), toml::Value::Table(layer)) => {
            merge(base, layer)
        }
        (toml::Value::Array(base), toml::Value::Array(layer))
            if named(base) && named(&layer) =>
        {
            merge_named(base, layer)
        }
        (base, layer) => {
            *base = normalised(layer)?;

            Ok(())
        }
    }
}

/// What a value looks like with nothing beneath it to merge into: as it
/// was, except that the directives its entries carry are consumed here
/// rather than left to reach a section schema.
fn normalised(layer: toml::Value) -> Result<toml::Value, Conflict> {
    match layer {
        toml::Value::Table(layer) => {
            let mut onto = toml::Table::new();

            merge(&mut onto, layer)?;
            Ok(toml::Value::Table(onto))
        }
        toml::Value::Array(layer) if named(&layer) => {
            let mut onto = Vec::new();

            merge_named(&mut onto, layer)?;
            Ok(toml::Value::Array(onto))
        }
        layer => Ok(layer),
    }
}

fn name_of(entry: &toml::Value) -> Option<&str> {
    match entry {
        toml::Value::Table(fields) => {
            fields.get("name").and_then(toml::Value::as_str)
        }
        _ => None,
    }
}

/// Whether every entry carries a `name` to be matched on, which is what
/// makes an array mergeable entry by entry rather than wholesale.
fn named(array: &[toml::Value]) -> bool {
    !array.is_empty() && array.iter().all(|entry| name_of(entry).is_some())
}

fn position(base: &[toml::Value], name: &str) -> Option<usize> {
    base.iter().position(|entry| name_of(entry) == Some(name))
}

/// Whether this entry says to remove the one it names, consuming the
/// directive so it never reaches a section schema. A `drop` that is not
/// a boolean is left where it is, so the schema reports it rather than
/// it quietly meaning `false`.
fn dropped(entry: &mut toml::Value) -> bool {
    let flag = entry
        .as_table()
        .and_then(|fields| fields.get("drop"))
        .and_then(toml::Value::as_bool);

    if let (Some(flag), Some(fields)) = (flag, entry.as_table_mut()) {
        fields.remove("drop");

        return flag;
    }

    false
}

/// Which entry this one says to sit ahead of, consuming the directive
/// so it never reaches a section schema. Like `drop`, a `before` that
/// is not a string is left where it is for the schema to report.
fn before(entry: &mut toml::Value) -> Option<String> {
    let name = entry
        .as_table()
        .and_then(|fields| fields.get("before"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)?;

    entry.as_table_mut()?.remove("before");

    Some(name)
}

/// The first name this array gives to two of its entries.
fn duplicate(array: &[toml::Value]) -> Option<&str> {
    array.iter().enumerate().find_map(|(at, entry)| {
        let name = name_of(entry)?;

        array[..at]
            .iter()
            .any(|earlier| name_of(earlier) == Some(name))
            .then_some(name)
    })
}

fn merge_named(
    base: &mut Vec<toml::Value>,
    layer: Vec<toml::Value>,
) -> Result<(), Conflict> {
    if let Some(name) = duplicate(&layer) {
        return Err(Conflict::Duplicate(name.to_owned()));
    }

    for mut entry in layer {
        let dropped = dropped(&mut entry);
        let before = before(&mut entry);
        let name = name_of(&entry).unwrap_or_default().to_owned();

        let to = match before {
            None => None,
            Some(before) if before == name => {
                return Err(Conflict::Before(before));
            }
            Some(before) => {
                Some(position(base, &before).ok_or(Conflict::Before(before))?)
            }
        };

        match (position(base, &name), dropped) {
            (Some(at), true) => {
                base.remove(at);
            }
            (Some(at), false) => {
                merge_value(&mut base[at], entry)?;

                if let Some(to) = to {
                    let entry = base.remove(at);

                    base.insert(to - usize::from(to > at), entry);
                }
            }
            (None, true) => {}
            (None, false) => {
                let entry = normalised(entry)?;

                match to {
                    Some(to) => base.insert(to, entry),
                    None => base.push(entry),
                }
            }
        }
    }

    Ok(())
}

/// Read the file at `path`, [`resolve`] its `extends` chain, and
/// deserialize the `[key]` section of the merged document.
pub fn load<T: DeserializeOwned>(
    path: impl AsRef<Path>,
    key: &str,
) -> Result<Option<T>, Error> {
    let mut table = resolve(path)?;

    let Some(value) = table.remove(key) else {
        return Ok(None);
    };

    value.try_into().map(Some).map_err(Error::Toml)
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
        let document = indoc::indoc! {r#"
            [commit]
            name = "value"
        "#};

        assert_eq!(
            section::<Section>(document, "commit").unwrap(),
            Some(Section {
                name: "value".into()
            })
        );
        assert_eq!(section::<Section>(document, "branch").unwrap(), None);
        assert_eq!(section::<Section>("", "commit").unwrap(), None);

        assert!(section::<Section>("not toml", "commit").is_err());
        assert!(
            section::<Section>(
                indoc::indoc! {r#"
                [commit]
                name = 1
            "#},
                "commit"
            )
            .is_err()
        );
    }
}
