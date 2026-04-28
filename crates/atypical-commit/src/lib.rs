use chumsky::prelude::*;
use flagset::{FlagSet, flags};

flags! {
    /// The position of the modifier.
    ///
    /// It implements [`FlagSet`] and can be used as a bitmask.
    #[doc(alias("Modifier", "Importance", "Position", "Location"))]
    pub enum ModifierPosition: u8 {
        /// Appears directly after the keyword and before enclosures.
        Before,
        /// Appears after enclosures.
        /// This is the Conventional Commits style.
        After,
    }
}

/// The kind of the modifier.
#[doc(alias("Modifier", "Importance", "Kind", "Type"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierKind {
    /// The symbol `?`.
    Question,
    /// The symbol `!`, with the count of exclamation marks.
    /// Also known as the "breaking change" in Conventional Commits.
    Exclamation(usize),
}

/// The modifier of a prefix.
#[doc(alias("Importance"))]
#[derive(Debug, Clone, PartialEq)]
pub struct Modifier {
    // Should be guaranteed to be XOR-compatible.
    pub position: ModifierPosition,
    pub kind: ModifierKind,
}

flags! {
    /// The kind of an enclosure delimiters.
    ///
    /// It implements [`FlagSet`] and can be used as a bitmask.
    #[doc(alias("Scope"))]
    pub enum Delimiter: u8 {
        /// The delimiters `(` and `)`.
        /// Is used for the "scope" in Conventional Commits.
        #[doc(alias("Parenthesis"))]
        Round,
        /// The delimiters `[` and `]`.
        #[doc(alias("Bracket"))]
        Square,
    }
}

/// An enclosure of content within delimiters.
#[doc(alias("Scope"))]
#[derive(Debug, Clone, PartialEq)]
pub struct Enclosure<'i> {
    // Should be guaranteed to be XOR-compatible.
    pub delimiter: Delimiter,
    pub content: &'i str,
}

/// The prefix of a commit message header.
#[doc(alias("Type", "Verb"))]
#[derive(Debug, Clone, PartialEq)]
pub struct Prefix<'i> {
    /// Also known as the "type" in Conventional Commits.
    pub keyword: &'i str,
    /// Also known is the "breaking change" in Conventional Commits.
    pub modifier: Option<Modifier>,
    /// Also known as the "scope" in Conventional Commits.
    pub enclosures: Vec<Enclosure<'i>>,
}

/// The header of a commit message. Contains a pair of (prefix, summary).
pub type Header<'i> = (Prefix<'i>, &'i str);

/// The body of a commit message. Contains the raw text.
pub type Body<'i> = &'i str;

/// The trailers of a commit message. Contains a sequence of (key, text) pairs.
#[doc(alias("Footer"))]
pub type Trailers<'i> = Vec<(&'i str, &'i str)>;

/// The general context of the parser.
#[doc(alias("Config", "Settings"))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraContext {
    pub allowed_modifier_positions: FlagSet<ModifierPosition>,
    pub allowed_enclosure_delimiters: FlagSet<Delimiter>,
}

impl Default for ExtraContext {
    /// By default, all flags are allowed.
    fn default() -> Self {
        Self {
            allowed_modifier_positions: FlagSet::full(),
            allowed_enclosure_delimiters: FlagSet::full(),
        }
    }
}

impl ExtraContext {
    pub fn with_modifier_positions(
        &self,
        positions: impl Into<FlagSet<ModifierPosition>>,
    ) -> Self {
        Self {
            allowed_modifier_positions: positions.into(),
            ..*self
        }
    }

    pub fn with_enclosure_delimiters(
        &self,
        delimiters: impl Into<FlagSet<Delimiter>>,
    ) -> Self {
        Self {
            allowed_enclosure_delimiters: delimiters.into(),
            ..*self
        }
    }
}

/// Implements [`extra::ParserExtra`].
#[doc(alias("Config", "Settings"))]
pub type Extra = extra::Context<ExtraContext>;

/// Parses a [`ModifierKind`].
///
/// # Examples
///
/// ```
/// ```
pub fn modifier<'i>() -> impl Parser<'i, &'i str, ModifierKind, Extra> {
    choice((
        just('?').to(ModifierKind::Question),
        just('!')
            .repeated()
            .at_least(1)
            .to_slice()
            .map(|v: &str| ModifierKind::Exclamation(v.len())),
    ))
}

/// Parses an [`Enclosure`].
///
/// For contextual parsing, use [`fn@enclosure_with_ctx`].
///
/// # Examples
///
/// ```
/// ```
#[doc(alias("scope"))]
pub fn enclosure<'i>() -> impl Parser<'i, &'i str, Enclosure<'i>, Extra> {
    fn dry<'i>(
        start: char,
        end: char,
        delimiter: Delimiter,
    ) -> impl Parser<'i, &'i str, Enclosure<'i>, Extra> {
        none_of::<'i, _, _, Extra>([start, end])
            .repeated()
            .to_slice()
            .delimited_by(just(start), just(end))
            .contextual()
            .configure(move |_, ctx| {
                ctx.allowed_enclosure_delimiters.contains(delimiter)
            })
            .map(move |content| Enclosure { delimiter, content })
    }

    choice((
        dry('(', ')', Delimiter::Round),
        dry('[', ']', Delimiter::Square),
    ))
}

/// Parses an [`Enclosure`] with a given delimiter context.
/// Accepts a parameter of allowed [delimiters](Delimiter).
///
/// For generic parsing, use [`fn@enclosure`].
///
/// # Examples
///
/// ```
/// ```
#[doc(alias("scope"))]
pub fn enclosure_with_ctx<'i>(
    ctx: ExtraContext,
) -> impl Parser<'i, &'i str, Enclosure<'i>, Extra> {
    Parser::<'i, &'i str, Enclosure<'i>, Extra>::with_ctx(enclosure(), ctx)
}

pub fn prefix<'i>() -> impl Parser<'i, &'i str, Prefix<'i>, Extra> {
    let keyword = any()
        .filter(|c: &char| c.is_ascii_alphabetic())
        .repeated()
        .at_least(1)
        .to_slice()
        .labelled("keyword");

    let before_modifier = modifier()
        .contextual()
        .configure(|_, ctx: &ExtraContext| {
            ctx.allowed_modifier_positions
                .contains(ModifierPosition::Before)
        })
        .map(|kind| Modifier {
            position: ModifierPosition::Before,
            kind,
        })
        .or_not()
        .labelled("modifier");

    let enclosures = enclosure().labelled("enclosure").repeated().collect();

    let after_modifier = modifier()
        .contextual()
        .configure(|_, ctx| {
            ctx.allowed_modifier_positions
                .contains(ModifierPosition::After)
        })
        .map(|kind| Modifier {
            position: ModifierPosition::After,
            kind,
        })
        .or_not()
        .labelled("modifier");

    group((keyword, before_modifier, enclosures, after_modifier)).map(
        |(keyword, before_modifier, enclosures, after_modifier)| Prefix {
            keyword,
            modifier: after_modifier.or(before_modifier),
            enclosures,
        },
    )
}

pub fn header<'i>() -> impl Parser<'i, &'i str, Header<'i>, Extra> {
    let summary = any().filter(|c: &char| *c != '\n').repeated().to_slice();

    group((prefix(), just(':').then_ignore(just(' ')), summary))
        .map(|(prefix, _, summary)| (prefix, summary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier() {
        assert_eq!(
            modifier().parse("?").into_result(),
            Ok(ModifierKind::Question)
        );

        assert_eq!(
            modifier().parse("!!").into_result(),
            Ok(ModifierKind::Exclamation(2))
        );

        assert!(modifier().parse("??").has_errors());
    }

    #[test]
    fn test_enclosure() {
        assert_eq!(
            enclosure().parse("(example)").into_result(),
            Ok(Enclosure {
                delimiter: Delimiter::Round,
                content: "example"
            })
        );

        assert_eq!(
            enclosure().parse("[(\t]").into_result(),
            Ok(Enclosure {
                delimiter: Delimiter::Square,
                content: "(\t"
            })
        );
    }

    #[test]
    fn test_enclosure_with_ctx() {
        assert_eq!(
            enclosure_with_ctx(
                ExtraContext::default()
                    .with_enclosure_delimiters(Delimiter::Round)
            )
            .parse("(example)")
            .into_result(),
            Ok(Enclosure {
                delimiter: Delimiter::Round,
                content: "example"
            })
        );

        assert!(
            enclosure_with_ctx(
                ExtraContext::default()
                    .with_enclosure_delimiters(Delimiter::Square)
            )
            .parse("(fail)")
            .has_errors()
        );
    }

    #[test]
    fn test_header_simple() {
        assert_eq!(
            header().parse("fix: resolve issue").into_result(),
            Ok((
                Prefix {
                    keyword: "fix",
                    modifier: None,
                    enclosures: vec![]
                },
                "resolve issue"
            ))
        );
    }

    #[test]
    fn test_header_modifier() {
        assert_eq!(
            header().parse("add!: breaking change").into_result(),
            Ok((
                Prefix {
                    keyword: "add",
                    modifier: Some(Modifier {
                        position: ModifierPosition::Before,
                        kind: ModifierKind::Exclamation(1)
                    }),
                    enclosures: vec![]
                },
                "breaking change"
            ))
        );

        assert_eq!(
            header().parse("fix?: maybe breaking").into_result(),
            Ok((
                Prefix {
                    keyword: "fix",
                    modifier: Some(Modifier {
                        position: ModifierPosition::Before,
                        kind: ModifierKind::Question
                    }),
                    enclosures: vec![]
                },
                "maybe breaking"
            ))
        );

        assert_eq!(
            header().parse("ref!!: mass refactor").into_result(),
            Ok((
                Prefix {
                    keyword: "ref",
                    modifier: Some(Modifier {
                        position: ModifierPosition::Before,
                        kind: ModifierKind::Exclamation(2)
                    }),
                    enclosures: vec![]
                },
                "mass refactor"
            ))
        );
    }

    #[test]
    fn test_header_enclosure() {
        assert_eq!(
            header()
                .parse("add(lib): new library feature")
                .into_result(),
            Ok((
                Prefix {
                    keyword: "add",
                    modifier: None,
                    enclosures: vec![Enclosure {
                        delimiter: Delimiter::Round,
                        content: "lib"
                    }]
                },
                "new library feature"
            ))
        );

        assert_eq!(
            header()
                .parse("fix[eff]: faster algorithm")
                .into_result(),
            Ok((
                Prefix {
                    keyword: "fix",
                    modifier: None,
                    enclosures: vec![Enclosure {
                        delimiter: Delimiter::Square,
                        content: "eff"
                    }]
                },
                "faster algorithm"
            ))
        );

        assert_eq!(
            header()
                .parse("ref(build)[cmp]: build compatibility")
                .into_result(),
            Ok((
                Prefix {
                    keyword: "ref",
                    modifier: None,
                    enclosures: vec![
                        Enclosure {
                            delimiter: Delimiter::Round,
                            content: "build"
                        },
                        Enclosure {
                            delimiter: Delimiter::Square,
                            content: "cmp"
                        }
                    ]
                },
                "build compatibility"
            ))
        );
    }

    #[test]
    fn test_header_modifier_enclosure() {
        assert_eq!(
            header()
                .parse("fix(lib)!: update library API")
                .into_result(),
            Ok((
                Prefix {
                    keyword: "fix",
                    modifier: Some(Modifier {
                        position: ModifierPosition::After,
                        kind: ModifierKind::Exclamation(1)
                    }),
                    enclosures: vec![Enclosure {
                        delimiter: Delimiter::Round,
                        content: "lib"
                    }]
                },
                "update library API"
            ))
        );

        assert_eq!(
            header()
                .parse("fix?(ci)[exp]: try new CI configuration")
                .into_result(),
            Ok((
                Prefix {
                    keyword: "fix",
                    modifier: Some(Modifier {
                        position: ModifierPosition::Before,
                        kind: ModifierKind::Question
                    }),
                    enclosures: vec![Enclosure {
                        delimiter: Delimiter::Round,
                        content: "ci"
                    }, Enclosure {
                        delimiter: Delimiter::Square,
                        content: "exp"
                    }]
                },
                "try new CI configuration"
            ))
        );

        assert_eq!(
            header()
                .parse("rem!!(build): remove current build system")
                .into_result(),
            Ok((
                Prefix {
                    keyword: "rem",
                    modifier: Some(Modifier {
                        position: ModifierPosition::Before,
                        kind: ModifierKind::Exclamation(2)
                    }),
                    enclosures: vec![Enclosure {
                        delimiter: Delimiter::Round,
                        content: "build"
                    }]
                },
                "remove current build system"
            ))
        );
    }
}
