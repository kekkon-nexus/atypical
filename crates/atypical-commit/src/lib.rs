// Syntax to follow:
// <keyword>[<modifier>][<open_delim><enclosure><close_delim>]...[<modifier>]: <description>

use chumsky::prelude::*;

pub mod config;
pub mod ignore;

pub type DelimitedBy = [char; 2];

#[doc(alias("Type", "Verb"))]
pub type Keyword<'i> = &'i str;

#[doc(alias("Importance", "BreakingChange"))]
pub type Modifier<'i> = &'i str;

#[doc(alias("Scope"))]
pub type Enclosure<'i> = (&'i str, DelimitedBy);

#[derive(Debug, Clone, PartialEq)]
pub struct Prefix<'i> {
    pub keyword: Keyword<'i>,
    pub modifier: Option<Modifier<'i>>,
    pub enclosures: Vec<Enclosure<'i>>,
}

#[doc(alias("Subject"))]
pub type Description<'i> = &'i str;

#[derive(Debug, Clone, PartialEq)]
pub struct Header<'i> {
    pub prefix: Prefix<'i>,
    pub description: Description<'i>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sequence {
    Pre,
    Post,
    /// Either position.
    Any,
}

/// A restrictable set of accepted spellings: anything, or a closed
/// list.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenSet<'i> {
    Any,
    OneOf(Vec<&'i str>),
}

pub type KeywordToken<'i> = TokenSet<'i>;

pub type ModifierToken<'i> = TokenSet<'i>;

/// The separator: one specific character, or any single symbol.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeparatorToken {
    Any,
    Just(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnclosureToken<'i> {
    Flexible(DelimitedBy),
    Strict(DelimitedBy, Vec<&'i str>),
}

impl<'i> EnclosureToken<'i> {
    #[inline]
    pub fn delimiters(&self) -> DelimitedBy {
        match self {
            EnclosureToken::Flexible(delimiters) => *delimiters,
            EnclosureToken::Strict(delimiters, _) => *delimiters,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tokens<'i> {
    pub keywords: KeywordToken<'i>,
    pub modifiers: ModifierToken<'i>,
    pub enclosures: Vec<EnclosureToken<'i>>,
    pub separator: SeparatorToken,

    pub modifier_sequence: Sequence,
}

pub struct Positional {
    pub modifier_sequence: Sequence,
}

impl Default for Tokens<'_> {
    /// Unrestricted: any keyword, any modifier on either side of
    /// free-form `(...)`/`[...]` enclosures, and any single-symbol
    /// separator. Only the header shape itself is enforced.
    fn default() -> Self {
        Self {
            keywords: TokenSet::Any,
            modifiers: TokenSet::Any,
            enclosures: vec![
                EnclosureToken::Flexible(['(', ')']),
                EnclosureToken::Flexible(['[', ']']),
            ],
            separator: SeparatorToken::Any,
            modifier_sequence: Sequence::Any,
        }
    }
}

pub type ExtraError<'i> = Rich<'i, char>;

pub type ExtraState<'i> = ();

#[doc(alias("Config", "Settings"))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraContext<'i> {
    pub tokens: Tokens<'i>,
}

impl<'i> ExtraContext<'i> {
    pub fn new(tokens: &Tokens<'i>) -> Self {
        fn sort(set: &mut TokenSet<'_>) {
            if let TokenSet::OneOf(v) = set {
                v.sort_unstable_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
            }
        }

        let mut tokens = tokens.clone();

        sort(&mut tokens.keywords);
        sort(&mut tokens.modifiers);

        Self { tokens }
    }
}

impl<'i> Default for ExtraContext<'i> {
    fn default() -> Self {
        Self::new(&Tokens::default())
    }
}

impl<'i> From<Tokens<'i>> for ExtraContext<'i> {
    fn from(val: Tokens<'i>) -> Self {
        ExtraContext::new(&val)
    }
}

#[doc(alias("Config", "Settings"))]
pub type Extra<'i> =
    extra::Full<ExtraError<'i>, ExtraState<'i>, ExtraContext<'i>>;

fn ident<'i>(
    i: &mut chumsky::input::InputRef<'i, '_, &'i str, Extra<'i>>,
) -> (&'i str, SimpleSpan) {
    let before = i.cursor();

    while i
        .peek()
        .is_some_and(|c: char| c.is_alphanumeric() || c == '_')
    {
        i.next();
    }

    (i.slice_since(&before..), i.span_since(&before))
}

/// A visible char that can't belong to a keyword or a description.
fn is_symbol(c: char) -> bool {
    !c.is_alphanumeric() && c != '_' && !c.is_whitespace()
}

fn expected_one_of(found: &str, kind: &str, expected: &[&str]) -> String {
    let expected = expected.join(", ");

    if found.is_empty() {
        format!("expected {kind}, one of: {expected}")
    } else {
        format!("unknown {kind} `{found}`, expected one of: {expected}")
    }
}

pub fn keyword<'i>() -> impl Parser<'i, &'i str, Keyword<'i>, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        let (s, span) = ident(i);

        match &i.ctx().tokens.keywords {
            TokenSet::Any if !s.is_empty() => Ok(s),
            TokenSet::Any => Err(Rich::custom(span, "expected a keyword")),
            TokenSet::OneOf(keywords) if keywords.contains(&s) => Ok(s),
            TokenSet::OneOf(keywords) => {
                let message = expected_one_of(s, "keyword", keywords);

                Err(Rich::custom(span, message))
            }
        }
    })
}

pub fn modifier<'i>() -> impl Parser<'i, &'i str, Modifier<'i>, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        if let TokenSet::OneOf(modifiers) = &i.ctx().tokens.modifiers {
            let parsers = modifiers
                .iter()
                .map(|&token| just(token))
                .collect::<Vec<_>>();

            return i.parse(choice(parsers));
        }

        // Any: the longest run of symbols that leaves the separator
        // and the enclosure openers untouched.
        let tokens = &i.ctx().tokens;
        let separator = tokens.separator;
        let openers = tokens
            .enclosures
            .iter()
            .map(|enclosure| enclosure.delimiters()[0])
            .collect::<Vec<_>>();

        let before = i.cursor();

        while let Some(c) = i.peek() {
            if !is_symbol(c) || openers.contains(&c) {
                break;
            }

            match separator {
                SeparatorToken::Just(s) if c == s => break,
                SeparatorToken::Just(_) => {
                    i.next();
                }
                // With an unrestricted separator, the last symbol
                // of the run is the separator, not the modifier.
                SeparatorToken::Any => {
                    let checkpoint = i.save();

                    i.next();

                    if !i.peek().is_some_and(is_symbol) {
                        i.rewind(checkpoint);
                        break;
                    }
                }
            }
        }

        let s = i.slice_since(&before..);

        if s.is_empty() {
            let span = i.span_since(&before);

            return Err(Rich::custom(span, "expected a modifier"));
        }

        Ok(s)
    })
}

pub fn enclosures<'i>()
-> impl Parser<'i, &'i str, Vec<Enclosure<'i>>, Extra<'i>> {
    use chumsky::input::InputRef;

    fn parser<'i>(
        token: &EnclosureToken<'i>,
    ) -> impl Parser<'i, &'i str, Enclosure<'i>, Extra<'i>> {
        match *token {
            EnclosureToken::Flexible([start, end]) => {
                none_of::<'i, _, _, Extra>([start, end])
                    .repeated()
                    .to_slice()
                    .delimited_by(just(start), just(end))
                    .map(move |s| (s, [start, end]))
                    .boxed()
            }
            EnclosureToken::Strict([start, end], ref allowed) => {
                let allowed = allowed.clone();

                custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
                    let (s, span) = ident(i);

                    if allowed.contains(&s) {
                        return Ok(s);
                    }

                    let message = expected_one_of(s, "enclosure", &allowed);

                    Err(Rich::custom(span, message))
                })
                .delimited_by(just(start), just(end))
                .map(move |s| (s, [start, end]))
                .boxed()
            }
        }
    }

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        let ctx = i.ctx();
        let delimiters = ctx.tokens.enclosures.clone();
        let mut index = 0;
        let mut results = Vec::new();

        loop {
            if index >= delimiters.len() {
                break;
            }

            let next = i.peek();
            let is_open = delimiters[index..]
                .iter()
                .any(|enclosure| Some(enclosure.delimiters()[0]) == next);

            if !is_open {
                break;
            }

            let parsers =
                delimiters[index..].iter().map(parser).collect::<Vec<_>>();

            let (content, delimited_by) = i.parse(choice(parsers))?;
            let position = delimiters
                .iter()
                .position(|enclosure| enclosure.delimiters() == delimited_by)
                .unwrap();
            index += position + 1;
            results.push((content, delimited_by));
        }

        Ok(results)
    })
}

pub fn separator<'i>() -> impl Parser<'i, &'i str, char, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        match i.ctx().tokens.separator {
            SeparatorToken::Just(separator) => i.parse(just(separator)),
            SeparatorToken::Any => {
                let before = i.cursor();

                match i.peek() {
                    Some(c) if is_symbol(c) => {
                        i.next();

                        Ok(c)
                    }
                    _ => Err(Rich::custom(
                        i.span_since(&before),
                        "expected a separator",
                    )),
                }
            }
        }
    })
}

pub fn modifier_when<'i>(
    sequence: Sequence,
) -> impl Parser<'i, &'i str, Option<Modifier<'i>>, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        let position = i.ctx().tokens.modifier_sequence;

        if position != sequence && position != Sequence::Any {
            return Ok(None);
        }

        i.parse(modifier().or_not())
    })
}

pub fn description<'i>() -> impl Parser<'i, &'i str, Description<'i>, Extra<'i>>
{
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        let before = i.cursor();

        while i.peek().is_some_and(|c: char| c != '\n') {
            i.next();
        }

        let s = i.slice_since(&before..);
        let span = i.span_since(&before);

        let Some(rest) = s.strip_prefix(' ') else {
            let message = if s.trim().is_empty() {
                "expected a description after the separator"
            } else {
                "expected a space before the description"
            };

            return Err(Rich::custom(span, message));
        };

        if rest.trim().is_empty() {
            return Err(Rich::custom(
                span,
                "expected a description after the separator",
            ));
        }

        Ok(rest.trim_end())
    })
}

pub fn prefix<'i>() -> impl Parser<'i, &'i str, Prefix<'i>, Extra<'i>> {
    let keyword = keyword();

    let modifier_pre = modifier_when(Sequence::Pre);

    let enclosures = enclosures();

    let modifier_post = modifier_when(Sequence::Post);

    let separator = separator();

    group((keyword, modifier_pre, enclosures, modifier_post, separator)).map(
        |(keyword, modifier_pre, enclosures, modifier_post, _)| Prefix {
            keyword,
            modifier: modifier_pre.or(modifier_post),
            enclosures,
        },
    )
}

pub fn header<'i>() -> impl Parser<'i, &'i str, Header<'i>, Extra<'i>> {
    group((prefix(), description())).map(|(prefix, description)| Header {
        prefix,
        description,
    })
}
