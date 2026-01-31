use bitflag_attr::bitflag;
use chumsky::prelude::*;

/// The position of the modifier.
///
/// It is an enum flag and can be used as a bitmask.
/// [`Self::default`] allows all positions.
///
/// See [`struct@Modifier`].
#[doc(alias("modifier", "importance", "position", "location"))]
#[bitflag(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ModifierPosition {
    /// Appears directly after the keyword and before enclosures.
    Before = 1 << 0,
    /// Appears after enclosures.
    /// This is the Conventional Commits style.
    After = 1 << 1,
}

impl Default for ModifierPosition {
    /// Allows all positions.
    fn default() -> Self {
        Self::all()
    }
}

/// The kind of the modifier.
///
/// See [`struct@Modifier`].
#[doc(alias("modifier", "importance", "kind", "type"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierKind {
    /// The symbol `?`.
    Question,
    /// The symbol `!`, with the count of exclamation marks.
    /// Also known as the "breaking change" in Conventional Commits.
    Exclamation(usize),
}

/// The modifier of a prefix.
#[doc(alias("modifier", "importance"))]
#[derive(Debug, Clone, PartialEq)]
pub struct Modifier {
    // Should be guaranteed to be XOR-compatible.
    pub position: ModifierPosition,
    pub kind: ModifierKind,
}

/// The kind of an enclosure delimiters.
///
/// It is an enum flag and can be used as a bitmask.
/// [`Self::default`] allows all delimiters.
///
/// See [`struct@Enclosure`].
#[doc(alias("scope"))]
#[bitflag(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Delimiter {
    /// The delimiters `(` and `)`.
    /// Is used for the "scope" in Conventional Commits.
    #[doc(alias("parenthesis"))]
    Round = 1 << 0,
    /// The delimiters `[` and `]`.
    #[doc(alias("bracket"))]
    Square = 1 << 1,
}

impl Default for Delimiter {
    /// Allows all delimiters.
    fn default() -> Self {
        Self::all()
    }
}

/// An enclosure of content within delimiters.
#[doc(alias("scope"))]
#[derive(Debug, Clone, PartialEq)]
pub struct Enclosure<'i> {
    // Should be guaranteed to be XOR-compatible.
    pub delimiter: Delimiter,
    pub content: &'i str,
}

/// The prefix of a commit message header.
#[doc(alias("type", "verb"))]
#[derive(Debug, Clone, PartialEq)]
pub struct Prefix<'i> {
    /// Also known as the "type" in Conventional Commits.
    pub keyword: &'i str,
    /// Also known is the "breaking change" in Conventional Commits.
    pub modifier: Option<ModifierKind>,
    /// Also known as the "scope" in Conventional Commits.
    pub enclosures: Vec<Enclosure<'i>>,
}

/// The header of a commit message. Contains a pair of (prefix, summary).
pub type Header<'i> = (Prefix<'i>, &'i str);

/// The body of a commit message. Contains the raw text.
pub type Body<'i> = &'i str;

/// The trailers of a commit message. Contains a sequence of (key, text) pairs.
#[doc(alias("footer"))]
pub type Trailers<'i> = Vec<(&'i str, &'i str)>;

/// The general context of the parser.
// The default should allow all flags.
#[doc(alias("config", "settings"))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtraContext {
    pub allowed_modifier_position: ModifierPosition,
    pub allowed_enclosure_delimiter: Delimiter,
}

/// The parser extras.
///
/// See [extra::ParserExtra].
#[doc(alias("config", "settings"))]
pub type Extra = extra::Context<ExtraContext>;

/// Parses a [`ModifierKind`].
///
/// # Examples
///
/// ```
/// assert_eq!(
///     modifier().parse("?").into_result(),
///     Ok(ModifierKind::Question)
/// );
///
/// assert_eq!(
///     modifier().parse("!!").into_result(),
///     Ok(ModifierKind::Exclamation(2))
/// );
/// ```
pub fn modifier<'i>() -> impl Parser<'i, &'i str, ModifierKind> {
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
/// assert_eq!(
///     enclosure().parse("(example)").into_result(),
///     Ok(Enclosure {
///         delimiter: Delimiter::Round,
///         content: "example",
///    })
/// );
///
/// assert_eq!(
///     enclosure().parse("[(\t]").into_result(),
///     Ok(Enclosure {
///         delimiter: Delimiter::Square,
///         content: "(\t",
///     })
/// );
pub fn enclosure<'i>() -> impl Parser<'i, &'i str, Enclosure<'i>, extra::Context<Delimiter>> {
    fn dry<'i>(
        start: char,
        end: char,
        delimiter: Delimiter,
    ) -> impl Parser<'i, &'i str, Enclosure<'i>, extra::Context<Delimiter>> {
        none_of(format!("{}{}", start, end))
            .repeated()
            .to_slice()
            .delimited_by(just(start), just(end))
            .contextual()
            .configure(move |_, ctx: &Delimiter| ctx.contains(delimiter))
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
/// assert_eq!(
///     enclosure_with_ctx(Delimiter::Round)
///         .parse("(example)")
///         .into_result(),
///     Ok(Enclosure {
///         delimiter: Delimiter::Round,
///         content: "example",
///     })
/// );
///
/// assert!(
///     enclosure_with_ctx(Delimiter::Square)
///         .parse("(fail)")
///         .has_errors()
/// );
/// ```
pub fn enclosure_with_ctx<'i>(
    delimiter: Delimiter,
) -> impl Parser<'i, &'i str, Enclosure<'i>, extra::Context<Delimiter>> {
    enclosure().with_ctx(delimiter)
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
                content: "example",
            })
        );

        assert_eq!(
            enclosure().parse("[(\t]").into_result(),
            Ok(Enclosure {
                delimiter: Delimiter::Square,
                content: "(\t",
            })
        );
    }

    #[test]
    fn test_enclosure_with_ctx() {
        assert_eq!(
            enclosure_with_ctx(Delimiter::Round)
                .parse("(example)")
                .into_result(),
            Ok(Enclosure {
                delimiter: Delimiter::Round,
                content: "example",
            })
        );

        assert!(
            enclosure_with_ctx(Delimiter::Square)
                .parse("(fail)")
                .has_errors()
        );
    }
}
