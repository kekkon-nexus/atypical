// The `[commit]` section of atypical.toml: an owned mirror of
// `Tokens`, defaulting field by field to the standard preset.

use serde::Deserialize;

use crate::{DelimitedBy, EnclosureToken, Sequence, TokenSet, Tokens};

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

impl From<&TokenSet<'_>> for SetConfig {
    fn from(set: &TokenSet<'_>) -> Self {
        match set {
            TokenSet::Any => SetConfig::Any(Any::Any),
            TokenSet::OneOf(v) => {
                SetConfig::OneOf(v.iter().map(ToString::to_string).collect())
            }
        }
    }
}

impl<'i> From<&'i SetConfig> for TokenSet<'i> {
    fn from(set: &'i SetConfig) -> Self {
        match set {
            SetConfig::Any(_) => TokenSet::Any,
            SetConfig::OneOf(v) => {
                TokenSet::OneOf(v.iter().map(String::as_str).collect())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct CommitConfig {
    pub keywords: SetConfig,
    pub modifiers: Vec<String>,
    pub enclosures: Vec<EnclosureConfig>,
    pub separator: char,
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
    fn default() -> Self {
        (&Tokens::preset_standard()).into()
    }
}

impl From<&Tokens<'_>> for CommitConfig {
    fn from(tokens: &Tokens<'_>) -> Self {
        fn owned(v: &[&str]) -> Vec<String> {
            v.iter().map(ToString::to_string).collect()
        }

        Self {
            keywords: (&tokens.keywords).into(),
            modifiers: owned(&tokens.modifiers),
            enclosures: tokens
                .enclosures
                .iter()
                .map(|enclosure| EnclosureConfig {
                    delimiters: enclosure.delimiters(),
                    allowed: match enclosure {
                        EnclosureToken::Flexible(_) => None,
                        EnclosureToken::Strict(_, allowed) => {
                            Some(owned(allowed))
                        }
                    },
                })
                .collect(),
            separator: tokens.separator,
            modifier_sequence: tokens.modifier_sequence,
            default_ignores: true,
        }
    }
}

impl<'i> From<&'i CommitConfig> for Tokens<'i> {
    fn from(config: &'i CommitConfig) -> Self {
        fn borrowed(v: &[String]) -> Vec<&str> {
            v.iter().map(String::as_str).collect()
        }

        Self {
            keywords: (&config.keywords).into(),
            modifiers: borrowed(&config.modifiers),
            enclosures: config
                .enclosures
                .iter()
                .map(|enclosure| match &enclosure.allowed {
                    None => EnclosureToken::Flexible(enclosure.delimiters),
                    Some(allowed) => EnclosureToken::Strict(
                        enclosure.delimiters,
                        borrowed(allowed),
                    ),
                })
                .collect(),
            separator: config.separator,
            modifier_sequence: config.modifier_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mirrors_the_standard_preset() {
        let config = CommitConfig::default();

        assert_eq!(Tokens::from(&config), Tokens::preset_standard());
    }

    #[test]
    fn test_partial_section_keeps_preset_defaults() {
        let config: CommitConfig =
            toml::from_str(r#"keywords = ["feat", "fix"]"#).unwrap();

        assert_eq!(
            config.keywords,
            SetConfig::OneOf(vec!["feat".into(), "fix".into()])
        );
        assert_eq!(config.modifiers, CommitConfig::default().modifiers);
        assert_eq!(config.separator, ':');
    }

    #[test]
    fn test_any_keywords() {
        let config: CommitConfig =
            toml::from_str(r#"keywords = "any""#).unwrap();

        assert_eq!(config.keywords, SetConfig::Any(Any::Any));
        assert_eq!(Tokens::from(&config).keywords, TokenSet::Any);

        assert!(
            toml::from_str::<CommitConfig>(r#"keywords = "some""#).is_err()
        );
        assert!(toml::from_str::<CommitConfig>("keywords = 1").is_err());
    }

    #[test]
    fn test_enclosures_map_to_strict_and_flexible() {
        let config: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[enclosures]]
            delimiters = ["(", ")"]
            allowed = ["core"]

            [[enclosures]]
            delimiters = ["{", "}"]
        "#})
        .unwrap();

        assert_eq!(
            Tokens::from(&config).enclosures,
            vec![
                EnclosureToken::Strict(['(', ')'], vec!["core"]),
                EnclosureToken::Flexible(['{', '}']),
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
