// The `[commit]` section of atypical.toml, lowered into `Tokens`:
// either a slot list, or the keys describing the fixed layout.

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

fn anything() -> SetConfig {
    SetConfig::Any(Any::Any)
}

/// One `[[commit.slots]]` entry: delimited when it has `delimiters`,
/// bare when it has a `kind`.
#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct SlotConfig {
    pub name: String,
    pub kind: Option<KindConfig>,
    pub delimiters: Option<DelimitedBy>,
    #[serde(default = "anything")]
    pub values: SetConfig,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", default)]
pub struct CommitConfig {
    pub slots: Option<Vec<SlotConfig>>,
    pub keywords: Option<SetConfig>,
    pub modifiers: Option<SetConfig>,
    pub enclosures: Option<Vec<EnclosureConfig>>,
    pub separator: Option<SeparatorConfig>,
    pub modifier_sequence: Option<Sequence>,
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

/// A `[commit]` section that cannot be lowered into slots.
#[derive(Debug, Clone, PartialEq)]
pub enum Invalid {
    /// A slot list beside a key the list already says.
    Mixed(&'static str),
    /// A slot that is neither delimited nor bare, or both at once.
    Shape(String),
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
            Invalid::Mixed(key) => write!(
                f,
                "`slots` and `{key}` describe the same thing; keep one"
            ),
            Invalid::Shape(name) => write!(
                f,
                "slot `{name}` needs either `kind` or `delimiters`, not both"
            ),
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

/// What a slot's kind is called in TOML, for diagnostics.
fn kind(shape: Shape) -> &'static str {
    match shape {
        Shape::Delimited(_) => "delimited",
        Shape::Bare(Class::Word) => "a `word`",
        Shape::Bare(Class::Symbols) => "`symbols`",
        Shape::Bare(Class::Symbol) => "a `symbol`",
    }
}

/// Whether a slot of this shape could ever match this spelling. A
/// delimited slot reads its contents as one word, as a bare word slot
/// does.
fn fits(shape: Shape, spelling: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut chars = spelling.chars();

    match shape {
        Shape::Delimited(_) | Shape::Bare(Class::Word) => {
            !spelling.is_empty() && spelling.chars().all(word)
        }
        Shape::Bare(Class::Symbols) => {
            !spelling.is_empty() && spelling.chars().all(crate::is_symbol)
        }
        Shape::Bare(Class::Symbol) => {
            chars.next().is_some_and(crate::is_symbol) && chars.next().is_none()
        }
    }
}

impl core::error::Error for Invalid {}

impl Default for CommitConfig {
    /// Every key unset, which lowers to the unrestricted grammar.
    fn default() -> Self {
        Self {
            slots: None,
            keywords: None,
            modifiers: None,
            enclosures: None,
            separator: None,
            modifier_sequence: None,
            default_ignores: true,
        }
    }
}

/// The fixed layout: keyword, modifier, enclosures, modifier,
/// separator, with `modifier-sequence` deciding which modifier slots
/// exist. Each slot is named after the key that declared it.
pub(crate) fn fixed(config: &CommitConfig) -> Vec<Slot> {
    let slot = |name: String, shape, values, required| Slot {
        name,
        shape,
        values,
        required,
    };
    let values = |set: Option<&SetConfig>| set.map_or(Values::Any, Into::into);
    // `any` would put the same run in two slots, which is ambiguous,
    // so an unset sequence picks a side.
    let sequence = config.modifier_sequence.unwrap_or(Sequence::Post);
    let (pre, post) = match sequence {
        Sequence::Pre => (true, false),
        Sequence::Post => (false, true),
        Sequence::Any => (true, true),
    };
    let modifier = |name: String| {
        let modifiers = values(config.modifiers.as_ref());

        slot(name, Shape::Bare(Class::Symbols), modifiers, false)
    };
    // `any` spends one key on two slots, so each names the key to edit
    // rather than the one that declared its contents.
    let named = |side| {
        if pre && post {
            format!("modifier-sequence ({side})")
        } else {
            "modifiers".to_owned()
        }
    };
    let flexible = |delimiters| EnclosureConfig {
        delimiters,
        allowed: None,
    };
    let enclosures = config
        .enclosures
        .clone()
        .unwrap_or_else(|| vec![flexible(['(', ')']), flexible(['[', ']'])]);

    let keywords = values(config.keywords.as_ref());
    let mut slots = vec![slot(
        "keywords".to_owned(),
        Shape::Bare(Class::Word),
        keywords,
        true,
    )];

    slots.extend(pre.then(|| modifier(named("pre"))));
    slots.extend(enclosures.iter().enumerate().map(|(index, enclosure)| {
        let allowed =
            enclosure.allowed.clone().map_or(Values::Any, Values::Set);

        slot(
            format!("enclosures[{index}]"),
            Shape::Delimited(enclosure.delimiters),
            allowed,
            false,
        )
    }));
    slots.extend(post.then(|| modifier(named("post"))));
    slots.push(slot(
        "separator".to_owned(),
        Shape::Bare(Class::Symbol),
        config.separator.map_or(Values::Any, Into::into),
        true,
    ));

    slots
}

impl TryFrom<&SlotConfig> for Slot {
    type Error = Invalid;

    fn try_from(slot: &SlotConfig) -> Result<Self, Self::Error> {
        let shape = match (slot.delimiters, slot.kind) {
            (Some(delimiters), None) => Shape::Delimited(delimiters),
            (None, Some(kind)) => Shape::Bare(kind.into()),
            _ => return Err(Invalid::Shape(slot.name.clone())),
        };

        Ok(Self {
            name: slot.name.clone(),
            shape,
            values: (&slot.values).into(),
            required: slot.required,
        })
    }
}

impl TryFrom<&CommitConfig> for Tokens {
    type Error = Invalid;

    fn try_from(config: &CommitConfig) -> Result<Self, Self::Error> {
        let slots = self::slots(config)?;

        for slot in &slots {
            let Values::Set(spellings) = &slot.values else {
                continue;
            };
            let unmatchable = spellings
                .iter()
                .find(|spelling| !fits(slot.shape, spelling));

            if let Some(spelling) = unmatchable {
                return Err(Invalid::Spelling {
                    slot: slot.name.clone(),
                    spelling: spelling.clone(),
                    kind: kind(slot.shape),
                });
            }
        }

        Ok(Self { slots })
    }
}

fn slots(config: &CommitConfig) -> Result<Vec<Slot>, Invalid> {
    let Some(slots) = &config.slots else {
        return Ok(fixed(config));
    };

    let replaced = [
        ("keywords", config.keywords.is_some()),
        ("modifiers", config.modifiers.is_some()),
        ("enclosures", config.enclosures.is_some()),
        ("separator", config.separator.is_some()),
        ("modifier-sequence", config.modifier_sequence.is_some()),
    ];

    for (key, present) in replaced {
        if present {
            return Err(Invalid::Mixed(key));
        }
    }

    slots.iter().map(Slot::try_from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_unrestricted() {
        let config = CommitConfig::default();

        assert!(config.keywords.is_none());
        assert!(config.slots.is_none());
        assert_eq!(Tokens::try_from(&config).unwrap(), Tokens::default());
    }

    #[test]
    fn test_partial_section_keeps_preset_defaults() {
        let config: CommitConfig =
            toml::from_str(r#"keywords = ["feat", "fix"]"#).unwrap();

        assert_eq!(
            config.keywords,
            Some(SetConfig::OneOf(vec!["feat".into(), "fix".into()]))
        );
        assert!(config.modifiers.is_none());
        assert!(config.separator.is_none());
        assert!(config.modifier_sequence.is_none());
    }

    #[test]
    fn test_any_keywords() {
        let config: CommitConfig =
            toml::from_str(r#"keywords = "any""#).unwrap();

        assert_eq!(config.keywords, Some(SetConfig::Any(Any::Any)));
        assert!(
            toml::from_str::<CommitConfig>(r#"keywords = "some""#).is_err()
        );
        assert!(toml::from_str::<CommitConfig>("keywords = 1").is_err());
    }

    #[test]
    fn test_any_separator() {
        let config: CommitConfig =
            toml::from_str(r#"separator = "any""#).unwrap();

        assert_eq!(config.separator, Some(SeparatorConfig::Any(Any::Any)));

        let slots = Tokens::try_from(&config).unwrap().slots;

        assert_eq!(slots.last().unwrap().values, Values::Any);

        let config: CommitConfig =
            toml::from_str(r#"separator = ";""#).unwrap();

        assert_eq!(config.separator, Some(SeparatorConfig::Just(';')));

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
            Some(vec![
                EnclosureConfig {
                    delimiters: ['(', ')'],
                    allowed: Some(vec!["core".into()]),
                },
                EnclosureConfig {
                    delimiters: ['{', '}'],
                    allowed: None,
                },
            ])
        );
    }

    #[test]
    fn test_modifier_sequence_places_the_slot() {
        let names = |section: &str| {
            let config: CommitConfig = toml::from_str(section).unwrap();

            Tokens::try_from(&config)
                .unwrap()
                .slots
                .into_iter()
                .map(|slot| slot.name)
                .collect::<Vec<_>>()
        };

        assert_eq!(
            names(r#"modifier-sequence = "pre""#),
            [
                "keywords",
                "modifiers",
                "enclosures[0]",
                "enclosures[1]",
                "separator"
            ]
        );
        assert_eq!(
            names(r#"modifier-sequence = "post""#),
            [
                "keywords",
                "enclosures[0]",
                "enclosures[1]",
                "modifiers",
                "separator"
            ]
        );
        // Both sides, so each names the key to edit rather than the one
        // that declared its contents. Rejected later as ambiguous.
        assert_eq!(
            names(r#"modifier-sequence = "any""#),
            [
                "keywords",
                "modifier-sequence (pre)",
                "enclosures[0]",
                "enclosures[1]",
                "modifier-sequence (post)",
                "separator"
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

        assert_eq!(config.modifier_sequence, Some(Sequence::Post));

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

    #[test]
    fn test_slots_spell_out_the_fixed_layout() {
        let keys: CommitConfig = toml::from_str(indoc::indoc! {r#"
            keywords = ["add"]
            modifiers = ["!"]
            modifier-sequence = "post"
            separator = ":"

            [[enclosures]]
            delimiters = ["(", ")"]
            allowed = ["lib"]
        "#})
        .unwrap();
        let slots: CommitConfig = toml::from_str(indoc::indoc! {r#"
            [[slots]]
            name = "keywords"
            kind = "word"
            values = ["add"]
            required = true

            [[slots]]
            name = "enclosures[0]"
            delimiters = ["(", ")"]
            values = ["lib"]

            [[slots]]
            name = "modifiers"
            kind = "symbols"
            values = ["!"]

            [[slots]]
            name = "separator"
            kind = "symbol"
            values = [":"]
            required = true
        "#})
        .unwrap();

        assert_eq!(
            Tokens::try_from(&keys).unwrap(),
            Tokens::try_from(&slots).unwrap()
        );
    }

    #[test]
    fn test_slots_and_the_keys_they_replace_do_not_mix() {
        let config: CommitConfig = toml::from_str(indoc::indoc! {r#"
            keywords = ["add"]

            [[slots]]
            name = "keywords"
            kind = "word"
            required = true
        "#})
        .unwrap();

        assert_eq!(Tokens::try_from(&config), Err(Invalid::Mixed("keywords")));
        assert_eq!(
            Invalid::Mixed("keywords").to_string(),
            "`slots` and `keywords` describe the same thing; keep one"
        );
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
            "slot `scope` needs either `kind` or `delimiters`, not both"
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
            assert!(matches!(
                Tokens::try_from(&config),
                Err(Invalid::Spelling { .. })
            ));
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
