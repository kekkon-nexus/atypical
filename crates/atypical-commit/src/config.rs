// The `[commit]` section of atypical.toml, lowered into `Tokens`;
// unrestricted for every field left unset.

use serde::Deserialize;

use crate::{Class, DelimitedBy, Sequence, Shape, Slot, Tokens, Values};

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

/// `"any"`, or one specific character.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Deserialize)]
#[serde(untagged, expecting = "`any` or a single character")]
pub enum SeparatorConfig {
    Any(Any),
    Just(char),
}

impl From<SeparatorConfig> for Values {
    fn from(separator: SeparatorConfig) -> Self {
        match separator {
            SeparatorConfig::Any(_) => Values::Any,
            SeparatorConfig::Just(c) => Values::Set(vec![c.to_string()]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct CommitConfig {
    pub keywords: SetConfig,
    pub modifiers: SetConfig,
    pub enclosures: Vec<EnclosureConfig>,
    pub separator: SeparatorConfig,
    pub modifier_sequence: Sequence,
    /// Skip machine-generated headers (merges, reverts, version
    /// bumps...); not part of the grammar, so absent from `Tokens`.
    pub default_ignores: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct EnclosureConfig {
    pub delimiters: DelimitedBy,
    /// Restricts the contents to these values; anything goes when omitted.
    pub allowed: Option<Vec<String>>,
}

impl Default for CommitConfig {
    /// Unrestricted; fields omitted from a `[commit]` section fall
    /// back to this.
    fn default() -> Self {
        let flexible = |delimiters| EnclosureConfig {
            delimiters,
            allowed: None,
        };

        Self {
            keywords: SetConfig::Any(Any::Any),
            modifiers: SetConfig::Any(Any::Any),
            enclosures: vec![flexible(['(', ')']), flexible(['[', ']'])],
            separator: SeparatorConfig::Any(Any::Any),
            // `any` would put the same run in two slots, which is
            // ambiguous, so the default picks a side.
            modifier_sequence: Sequence::Post,
            default_ignores: true,
        }
    }
}

/// Today's fixed layout: keyword, modifier, enclosures, modifier,
/// separator, with `modifier-sequence` deciding which modifier slots
/// exist. Each slot is named after the key that declared it.
impl From<&CommitConfig> for Tokens {
    fn from(config: &CommitConfig) -> Self {
        let slot = |name: String, shape, values, required| Slot {
            name,
            shape,
            values,
            required,
        };
        let (pre, post) = match config.modifier_sequence {
            Sequence::Pre => (true, false),
            Sequence::Post => (false, true),
            Sequence::Any => (true, true),
        };
        let modifier = |name: String| {
            let values = (&config.modifiers).into();

            slot(name, Shape::Bare(Class::Symbols), values, false)
        };
        // `any` spends one key on two slots, so each names the key to
        // edit rather than the one that declared its contents.
        let named = |side| {
            if pre && post {
                format!("modifier-sequence ({side})")
            } else {
                "modifiers".to_owned()
            }
        };

        let keywords = (&config.keywords).into();
        let mut slots = vec![slot(
            "keywords".to_owned(),
            Shape::Bare(Class::Word),
            keywords,
            true,
        )];

        slots.extend(pre.then(|| modifier(named("pre"))));
        slots.extend(config.enclosures.iter().enumerate().map(
            |(index, enclosure)| {
                let values =
                    enclosure.allowed.clone().map_or(Values::Any, Values::Set);

                slot(
                    format!("enclosures[{index}]"),
                    Shape::Delimited(enclosure.delimiters),
                    values,
                    false,
                )
            },
        ));
        slots.extend(post.then(|| modifier(named("post"))));
        slots.push(slot(
            "separator".to_owned(),
            Shape::Bare(Class::Symbol),
            config.separator.into(),
            true,
        ));

        Self { slots }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_unrestricted() {
        let config = CommitConfig::default();

        assert_eq!(config.keywords, SetConfig::Any(Any::Any));
        assert_eq!(config.modifiers, SetConfig::Any(Any::Any));
        assert_eq!(config.separator, SeparatorConfig::Any(Any::Any));
    }

    #[test]
    fn test_partial_section_keeps_preset_defaults() {
        let config: CommitConfig =
            toml::from_str(r#"keywords = ["feat", "fix"]"#).unwrap();

        assert_eq!(
            config.keywords,
            SetConfig::OneOf(vec!["feat".into(), "fix".into()])
        );
        assert_eq!(config.modifiers, SetConfig::Any(Any::Any));
        assert_eq!(config.separator, SeparatorConfig::Any(Any::Any));
        assert_eq!(config.modifier_sequence, Sequence::Post);
    }

    #[test]
    fn test_any_keywords() {
        let config: CommitConfig =
            toml::from_str(r#"keywords = "any""#).unwrap();

        assert_eq!(config.keywords, SetConfig::Any(Any::Any));
        assert!(
            toml::from_str::<CommitConfig>(r#"keywords = "some""#).is_err()
        );
        assert!(toml::from_str::<CommitConfig>("keywords = 1").is_err());
    }

    #[test]
    fn test_any_modifiers() {
        let config: CommitConfig =
            toml::from_str(r#"modifiers = "any""#).unwrap();

        assert_eq!(config.modifiers, SetConfig::Any(Any::Any));
    }

    #[test]
    fn test_any_separator() {
        let config: CommitConfig =
            toml::from_str(r#"separator = "any""#).unwrap();

        assert_eq!(config.separator, SeparatorConfig::Any(Any::Any));

        let config: CommitConfig =
            toml::from_str(r#"separator = ";""#).unwrap();

        assert_eq!(config.separator, SeparatorConfig::Just(';'));

        assert!(toml::from_str::<CommitConfig>(r#"separator = "ab""#).is_err());
    }

    #[test]
    fn test_enclosures_allowed_is_optional() {
        let config: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[enclosures]]
            delimiters = ["(", ")"]
            allowed = ["core"]

            [[enclosures]]
            delimiters = ["{", "}"]
        "#})
        .unwrap();

        assert_eq!(
            config.enclosures,
            vec![
                EnclosureConfig {
                    delimiters: ['(', ')'],
                    allowed: Some(vec!["core".into()]),
                },
                EnclosureConfig {
                    delimiters: ['{', '}'],
                    allowed: None,
                },
            ]
        );
    }

    #[test]
    fn test_default_ignores_is_on_unless_disabled() {
        assert!(CommitConfig::default().default_ignores);

        let config: CommitConfig =
            toml::from_str("default-ignores = false").unwrap();

        assert!(!config.default_ignores);
    }

    #[test]
    fn test_modifier_sequence_names() {
        let config: CommitConfig =
            toml::from_str(r#"modifier-sequence = "post""#).unwrap();

        assert_eq!(config.modifier_sequence, Sequence::Post);

        let config: CommitConfig =
            toml::from_str(r#"modifier-sequence = "any""#).unwrap();

        assert_eq!(config.modifier_sequence, Sequence::Any);

        assert!(
            toml::from_str::<CommitConfig>(r#"modifier-sequence = "sideways""#)
                .is_err()
        );
    }

    #[test]
    fn test_unknown_fields_are_rejected() {
        assert!(
            toml::from_str::<CommitConfig>(r#"keyword = ["typo"]"#).is_err()
        );
    }
}
