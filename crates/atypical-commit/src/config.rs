//! The `[commit]` section of `atypical.toml`, lowered into
//! [`Tokens`](crate::Tokens).

use serde::Deserialize;

use crate::{Class, DelimitedBy, Shape, Slot, Tokens, Values};

pub const SECTION: &str = "commit";

/// The literal string `any` in TOML.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Any {
    Any,
}

/// `"any"`, or a closed list of accepted spellings.
#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(untagged, expecting = "`any` or an array of strings")]
pub enum SetConfig {
    Any(Any),
    OneOf(Vec<String>),
}

impl From<&SetConfig> for Values {
    fn from(set: &SetConfig) -> Self {
        match set {
            SetConfig::Any(_) => Values::Any,
            SetConfig::OneOf(v) => Values::Set(v.clone()),
        }
    }
}

/// What a bare slot's contents are made of.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KindConfig {
    Word,
    Symbols,
    Symbol,
}

impl From<KindConfig> for Class {
    fn from(kind: KindConfig) -> Self {
        match kind {
            KindConfig::Word => Class::Word,
            KindConfig::Symbols => Class::Symbols,
            KindConfig::Symbol => Class::Symbol,
        }
    }
}

/// One `[[commit.slots]]` entry: delimited when it has `delimiters`,
/// bare when it has a `kind`, and an alternation when it has `one-of`.
#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct SlotConfig {
    pub name: String,
    pub kind: Option<KindConfig>,
    pub delimiters: Option<DelimitedBy>,
    pub one_of: Option<Vec<SlotConfig>>,
    /// Unset takes anything, but unlike `any` it is also allowed where
    /// values mean nothing, as on a `one-of`.
    pub values: Option<SetConfig>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub gap: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct CommitConfig {
    pub slots: Option<Vec<SlotConfig>>,
    /// Skip machine-generated headers (merges, reverts, version
    /// bumps...); not part of the grammar, so absent from `Tokens`.
    pub default_ignores: bool,
}

/// A `[commit]` section that cannot be lowered into slots.
#[derive(Debug, Clone, PartialEq)]
pub enum Invalid {
    /// A slot with no shape, or more than one.
    Shape(String),
    /// A key that means nothing where it is: `values` on a `one-of`, or
    /// `required`, `gap` or `one-of` on one of its options.
    Key { slot: String, key: &'static str },
    /// A spelling the slot's kind can never match.
    Spelling {
        slot: String,
        spelling: String,
        kind: &'static str,
    },
}

impl core::fmt::Display for Invalid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Invalid::Shape(name) => write!(
                f,
                "slot `{name}` needs exactly one of `kind`, `delimiters` or \
                 `one-of`"
            ),
            Invalid::Key { slot, key } => {
                write!(f, "`{key}` does not apply to slot `{slot}`")
            }
            Invalid::Spelling {
                slot,
                spelling,
                kind,
            } => write!(
                f,
                "slot `{slot}` is {kind}, so it cannot match `{spelling}`"
            ),
        }
    }
}

/// What a slot's kind is called in TOML, for diagnostics. A `one-of`
/// holds no spellings of its own, so it never gets here.
fn kind(shape: &Shape) -> &'static str {
    match shape {
        Shape::Bare(Class::Word) => "a `word`",
        Shape::Bare(Class::Symbols) => "`symbols`",
        Shape::Bare(Class::Symbol) => "a `symbol`",
        Shape::Delimited(_) | Shape::OneOf(_) => "delimited",
    }
}

/// Whether a slot of this shape could ever match this spelling. A bare
/// word slot reads one word; a delimited slot allows a hyphen too, so a
/// shortcode like `t-rex` fits. A `one-of` never gets here, as for
/// [`kind`].
fn fits(shape: &Shape, spelling: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut chars = spelling.chars();

    match shape {
        Shape::Bare(Class::Symbols) => {
            !spelling.is_empty() && spelling.chars().all(crate::is_symbol)
        }
        Shape::Bare(Class::Symbol) => {
            chars.next().is_some_and(crate::is_symbol) && chars.next().is_none()
        }
        Shape::Bare(Class::Word) | Shape::OneOf(_) => {
            !spelling.is_empty() && spelling.chars().all(word)
        }
        Shape::Delimited(_) => {
            !spelling.is_empty()
                && spelling.chars().all(|c| word(c) || c == '-')
        }
    }
}

impl core::error::Error for Invalid {}

impl Default for CommitConfig {
    /// Every key unset, which lowers to the unrestricted grammar.
    fn default() -> Self {
        Self {
            slots: None,
            default_ignores: true,
        }
    }
}

impl TryFrom<&SlotConfig> for Slot {
    type Error = Invalid;

    fn try_from(slot: &SlotConfig) -> Result<Self, Self::Error> {
        let shape = match (slot.delimiters, slot.kind, &slot.one_of) {
            (Some(delimiters), None, None) => Shape::Delimited(delimiters),
            (None, Some(kind), None) => Shape::Bare(kind.into()),
            (None, None, Some(options)) if !options.is_empty() => {
                if slot.values.is_some() {
                    return Err(invalid_key(slot, "values"));
                }

                Shape::OneOf(
                    options.iter().map(option).collect::<Result<_, _>>()?,
                )
            }
            _ => return Err(Invalid::Shape(slot.name.clone())),
        };

        Ok(Self {
            name: slot.name.clone(),
            shape,
            values: slot.values.as_ref().map_or(Values::Any, Values::from),
            required: slot.required,
            gap: slot.gap,
        })
    }
}

fn invalid_key(slot: &SlotConfig, key: &'static str) -> Invalid {
    Invalid::Key {
        slot: slot.name.clone(),
        key,
    }
}

/// An option of a `one-of`, which leaves presence and spacing to the
/// slot holding it.
fn option(option: &SlotConfig) -> Result<Slot, Invalid> {
    let key = [
        (option.required, "required"),
        (option.gap, "gap"),
        (option.one_of.is_some(), "one-of"),
    ]
    .into_iter()
    .find_map(|(set, key)| set.then_some(key));

    match key {
        Some(key) => Err(invalid_key(option, key)),
        None => Slot::try_from(option),
    }
}

impl TryFrom<&CommitConfig> for Tokens {
    type Error = Invalid;

    fn try_from(config: &CommitConfig) -> Result<Self, Self::Error> {
        let slots = self::slots(config)?;

        for slot in slots.iter().flat_map(crate::forms) {
            let Values::Set(spellings) = &slot.values else {
                continue;
            };
            let unmatchable = spellings
                .iter()
                .find(|spelling| !fits(&slot.shape, spelling));

            if let Some(spelling) = unmatchable {
                return Err(Invalid::Spelling {
                    slot: slot.name.clone(),
                    spelling: spelling.clone(),
                    kind: kind(&slot.shape),
                });
            }
        }

        Ok(Self { slots })
    }
}

fn slots(config: &CommitConfig) -> Result<Vec<Slot>, Invalid> {
    let Some(slots) = &config.slots else {
        return Ok(Tokens::default().slots);
    };

    slots.iter().map(Slot::try_from).collect()
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn test_default_is_unrestricted() {
        let config = CommitConfig::default();

        assert!(config.slots.is_none());
        assert_eq!(Tokens::try_from(&config).unwrap(), Tokens::default());
    }

    #[test]
    fn test_a_section_without_slots_is_unrestricted() {
        let config: CommitConfig =
            toml::from_str("default-ignores = false").unwrap();

        assert!(config.slots.is_none());
        assert!(!config.default_ignores);
        assert_eq!(Tokens::try_from(&config).unwrap(), Tokens::default());
    }

    #[test]
    fn test_any_values() {
        let config: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "keywords"
            kind = "word"
            values = "any"
            required = true
        "#})
        .unwrap();

        assert_eq!(
            config.slots.as_ref().unwrap()[0].values,
            Some(SetConfig::Any(Any::Any))
        );
        assert_eq!(
            Tokens::try_from(&config).unwrap().slots[0].values,
            Values::Any
        );

        let invalid = |values: &str| {
            format!("[[slots]]\nname = \"keywords\"\nvalues = {values}\n")
        };

        assert!(toml::from_str::<CommitConfig>(&invalid(r#""some""#)).is_err());
        assert!(toml::from_str::<CommitConfig>(&invalid("1")).is_err());
    }

    #[test]
    fn test_default_ignores_is_on_unless_disabled() {
        assert!(CommitConfig::default().default_ignores);

        let config: CommitConfig =
            toml::from_str("default-ignores = false").unwrap();

        assert!(!config.default_ignores);
    }

    #[test]
    fn test_unknown_fields_are_rejected() {
        // The keys the slot list replaced are unknown like any other
        // typo now.
        for section in [
            r#"keyword = ["typo"]"#,
            r#"keywords = ["add"]"#,
            r#"modifiers = "any""#,
            r#"separator = ":""#,
            r#"modifier-sequence = "post""#,
            "enclosures = []",
        ] {
            assert!(
                toml::from_str::<CommitConfig>(section).is_err(),
                "{section}"
            );
        }
    }

    #[test]
    fn test_a_slot_is_delimited_or_bare() {
        let neither: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "scope"
        "#})
        .unwrap();
        let both: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "scope"
            kind = "word"
            delimiters = ["(", ")"]
        "#})
        .unwrap();

        for config in [neither, both] {
            assert_eq!(
                Tokens::try_from(&config),
                Err(Invalid::Shape("scope".to_owned()))
            );
        }

        assert_eq!(
            Invalid::Shape("scope".to_owned()).to_string(),
            "slot `scope` needs exactly one of `kind`, `delimiters` or `one-of`"
        );
    }

    #[test]
    fn test_a_one_of_lowers_its_options() {
        let config: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "intention"
            required = true
            gap = true

            [[slots.one-of]]
            name = "emoji"
            kind = "symbols"
            values = ["✨"]

            [[slots.one-of]]
            name = "shortcode"
            delimiters = [":", ":"]
        "#})
        .unwrap();
        let slots = Tokens::try_from(&config).unwrap().slots;

        let shapes = crate::forms(&slots[0])
            .iter()
            .map(|option| option.shape.clone())
            .collect::<Vec<_>>();

        assert!(slots[0].required && slots[0].gap);
        assert_eq!(
            shapes,
            [Shape::Bare(Class::Symbols), Shape::Delimited([':', ':'])]
        );
    }

    #[test]
    fn test_a_key_that_means_nothing_is_rejected() {
        let one_of = |outer: &str, inner: &str| {
            format!(
                "[[slots]]\nname = \"intention\"\n{outer}\n\n\
                 [[slots.one-of]]\nname = \"emoji\"\nkind = \"symbols\"\n{inner}\n"
            )
        };

        for (toml, slot, key) in [
            (one_of("values = [\"x\"]", ""), "intention", "values"),
            (one_of("", "required = true"), "emoji", "required"),
            (one_of("", "gap = true"), "emoji", "gap"),
        ] {
            let config: CommitConfig = toml::from_str(&toml).unwrap();

            assert_eq!(
                Tokens::try_from(&config),
                Err(Invalid::Key {
                    slot: slot.to_owned(),
                    key
                }),
                "{toml}"
            );
        }

        assert_eq!(
            Invalid::Key {
                slot: "emoji".to_owned(),
                key: "gap"
            }
            .to_string(),
            "`gap` does not apply to slot `emoji`"
        );
    }

    #[test]
    fn test_a_spelling_must_fit_the_kind() {
        let separator: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "separator"
            kind = "symbol"
            values = ["::"]
            required = true
        "#})
        .unwrap();
        let invalid = Invalid::Spelling {
            slot: "separator".to_owned(),
            spelling: "::".to_owned(),
            kind: "a `symbol`",
        };

        assert_eq!(Tokens::try_from(&separator), Err(invalid.clone()));
        assert_eq!(
            invalid.to_string(),
            "slot `separator` is a `symbol`, so it cannot match `::`"
        );

        let worded: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "modifiers"
            kind = "symbols"
            values = ["ab"]
        "#})
        .unwrap();
        let spaced: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "scope"
            delimiters = ["(", ")"]
            values = ["two words"]
        "#})
        .unwrap();

        let keyword: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "keywords"
            kind = "word"
            values = ["two words"]
            required = true
        "#})
        .unwrap();

        for config in [worded, spaced, keyword] {
            assert_matches!(
                Tokens::try_from(&config),
                Err(Invalid::Spelling { .. })
            );
        }
    }

    #[test]
    fn test_a_slot_defaults_to_anything_and_optional() {
        let config: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "scope"
            delimiters = ["(", ")"]
        "#})
        .unwrap();
        let slots = Tokens::try_from(&config).unwrap().slots;

        assert_eq!(slots[0].values, Values::Any);
        assert!(!slots[0].required);
    }
}
