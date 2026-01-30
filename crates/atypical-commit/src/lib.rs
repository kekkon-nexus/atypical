use bitflag_attr::bitflag;
use chumsky::prelude::*;

/// The modifier of a prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// The symbol `?`.
    Question,
    /// The symbol `!`, with the number of exclamation marks.
    /// Also known as the "breaking change" in Conventional Commits.
    Exclamation(usize),
}

/// The kind of the enclosure delimiters.
///
/// See [Enclosure].
#[bitflag(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Delimiter {
    /// The delimiters `(` and `)`.
    /// Is used for the "scope" in Conventional Commits.
    Round = 1 << 0,
    /// The delimiters `[` and `]`.
    Square = 1 << 1,
}

impl Default for Delimiter {
    fn default() -> Self {
        Self::all()
    }
}

/// An enclosure of content within delimiters.
#[derive(Debug, Clone, PartialEq)]
pub struct Enclosure<'i> {
    pub delimiter: Delimiter,
    pub content: &'i str,
}

/// The prefix of a commit message header.
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
pub type Trailers<'i> = Vec<(&'i str, &'i str)>;

/// The location of the modifier symbol.
///
/// See [Modifier].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModifierLocation {
    /// Appears directly after the keyword and before enclosures.
    Before,
    /// Appears after enclosures.
    /// This is the Conventional Commits style.
    After,
    /// Appears either before or after enclosures.
    /// It may only appear once.
    #[default]
    Either,
}

/// The context of the parser.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtraContext {
    pub modifier_location: ModifierLocation,
}

/// The parser extras.
///
/// See [extra::ParserExtra].
pub type Extra = extra::Context<ExtraContext>;

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

pub fn enclosure_with_ctx<'i>(
    delimiter: Delimiter,
) -> impl Parser<'i, &'i str, Enclosure<'i>, extra::Context<Delimiter>> {
    enclosure().with_ctx(delimiter)
}

#[cfg(test)]
mod tests {
    use super::*;

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
