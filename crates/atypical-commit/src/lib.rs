// Syntax to follow:
// <keyword>[<modifier>][<open_delim><enclosure><close_delim>]...[<modifier>]: <description>

use chumsky::prelude::*;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Sequence {
    Pre,
    Post,
}

trait TokenParser<'i, I, O>
where
    I: chumsky::input::Input<'i, Token = char, Span = SimpleSpan>,
{
    fn parser(&self) -> impl Parser<'i, I, O, Extra<'i>> + 'i;
}

impl<'i> TokenParser<'i, &'i str, &'i str> for &'i str {
    #[inline]
    #[doc(hidden)]
    fn parser(&self) -> impl Parser<'i, &'i str, &'i str, Extra<'i>> + 'i {
        just(*self)
    }
}

pub type KeywordToken<'i> = Keyword<'i>;

pub type ModifierToken<'i> = Modifier<'i>;

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

impl<'i> TokenParser<'i, &'i str, Enclosure<'i>> for EnclosureToken<'i> {
    fn parser(
        &self,
    ) -> impl Parser<'i, &'i str, Enclosure<'i>, Extra<'i>> + 'i {
        match *self {
            EnclosureToken::Flexible([start, end]) => {
                none_of::<'i, _, _, Extra>([start, end])
                    .repeated()
                    .to_slice()
                    .delimited_by(just(start), just(end))
                    .map(move |s| (s, [start, end]))
                    .boxed()
            }
            EnclosureToken::Strict([start, end], ref allowed) => {
                let allowed =
                    allowed.iter().map(|&s| just(s)).collect::<Vec<_>>();

                choice(allowed)
                    .to_slice()
                    .delimited_by(just(start), just(end))
                    .map(move |s| (s, [start, end]))
                    .boxed()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tokens<'i> {
    pub keywords: Vec<KeywordToken<'i>>,
    pub modifiers: Vec<ModifierToken<'i>>,
    pub enclosures: Vec<EnclosureToken<'i>>,
    pub separator: char,

    pub modifier_sequence: Sequence,
}

pub struct Positional {
    pub modifier_sequence: Sequence,
}

impl Tokens<'_> {
    pub fn preset_standard() -> Self {
        Self {
            keywords: vec!["add", "rem", "ref", "fix", "undo", "release"],
            modifiers: vec!["?", "!", "!!"],
            enclosures: vec![
                EnclosureToken::Strict(
                    ['(', ')'],
                    vec!["exe", "lib", "test", "build", "doc", "ci", "cd"],
                ),
                EnclosureToken::Strict(
                    ['[', ']'],
                    vec![
                        "int", "pre", "eff", "rel", "cmp", "mnt", "tmp", "exp",
                        "sec", "upg", "ux", "pol", "sty",
                    ],
                ),
            ],
            separator: ':',
            modifier_sequence: Sequence::Pre,
        }
    }
}

impl Default for Tokens<'_> {
    fn default() -> Self {
        Self::preset_standard()
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
        fn sort(v: &mut Vec<&str>) {
            v.sort_unstable_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
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

pub fn keyword<'i>() -> impl Parser<'i, &'i str, Keyword<'i>, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        let parsers: Vec<_> = i
            .ctx()
            .tokens
            .keywords
            .iter()
            .map(TokenParser::parser)
            .collect();

        i.parse(choice(parsers))
    })
    .labelled("keyword")
}

pub fn modifier<'i>() -> impl Parser<'i, &'i str, Modifier<'i>, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        let parsers = i
            .ctx()
            .tokens
            .modifiers
            .iter()
            .map(TokenParser::parser)
            .collect::<Vec<_>>();

        i.parse(choice(parsers))
    })
    .labelled("modifier")
}

pub fn enclosures<'i>()
-> impl Parser<'i, &'i str, Vec<Enclosure<'i>>, Extra<'i>> {
    use chumsky::input::InputRef;

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

            let parsers = delimiters[index..]
                .iter()
                .map(TokenParser::parser)
                .collect::<Vec<_>>();

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
    .labelled("enclosures")
}

pub fn separator<'i>() -> impl Parser<'i, &'i str, char, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        let ctx = i.ctx();

        i.parse(just(ctx.tokens.separator))
    })
    .labelled("separator")
}

pub fn prefix<'i>() -> impl Parser<'i, &'i str, Prefix<'i>, Extra<'i>> {
    let keyword = keyword();

    let modifier_pre = modifier()
        .contextual()
        .configure(|_, ctx| {
            matches!(ctx.tokens.modifier_sequence, Sequence::Pre)
        })
        .or_not();

    let enclosures = enclosures();

    let modifier_post = modifier()
        .contextual()
        .configure(|_, ctx| {
            matches!(ctx.tokens.modifier_sequence, Sequence::Post)
        })
        .or_not();

    let separator = separator();

    group((keyword, modifier_pre, enclosures, modifier_post, separator)).map(
        |(keyword, modifier_pre, enclosures, modifier_post, _)| Prefix {
            keyword,
            modifier: modifier_pre.or(modifier_post),
            enclosures,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword() {
        fn parser_standard<'i>()
        -> impl Parser<'i, &'i str, Keyword<'i>, Extra<'i>> {
            keyword().with_ctx(Tokens::preset_standard().into())
        }

        assert!(parser_standard().parse("").has_errors());

        assert_eq!(parser_standard().parse("add").into_result(), Ok("add"));
        assert_eq!(parser_standard().parse("rem").into_result(), Ok("rem"));
        assert!(parser_standard().parse("feat").has_errors());
    }

    #[test]
    fn test_modifier() {
        fn parser_standard<'i>()
        -> impl Parser<'i, &'i str, Modifier<'i>, Extra<'i>> {
            modifier().with_ctx(Tokens::preset_standard().into())
        }

        assert!(parser_standard().parse("").has_errors());

        assert_eq!(parser_standard().parse("?").into_result(), Ok("?"));
        assert_eq!(parser_standard().parse("!!").into_result(), Ok("!!"));
        assert!(parser_standard().parse("??").has_errors());
    }

    #[test]
    fn test_enclosures() {
        fn parser_standard<'i>()
        -> impl Parser<'i, &'i str, Vec<Enclosure<'i>>, Extra<'i>> {
            enclosures().with_ctx(Tokens::preset_standard().into())
        }

        assert_eq!(parser_standard().parse("").into_result(), Ok(vec![]));

        assert_eq!(
            parser_standard().parse("(lib)").into_result(),
            Ok(vec![("lib", ['(', ')'])])
        );
        assert_eq!(
            parser_standard().parse("[pre]").into_result(),
            Ok(vec![("pre", ['[', ']'])])
        );
        assert_eq!(
            parser_standard().parse("(exe)[int]").into_result(),
            Ok(vec![("exe", ['(', ')']), ("int", ['[', ']'])])
        );
        assert!(parser_standard().parse("(").has_errors());
        assert!(parser_standard().parse("(unsupported)").has_errors());
        assert!(parser_standard().parse("{unsupported}").has_errors());
        assert!(parser_standard().parse("[pre](lib)").has_errors());
        assert!(parser_standard().parse("(exe)(lib)").has_errors());
    }

    #[test]
    fn test_separator() {
        fn parser_standard<'i>() -> impl Parser<'i, &'i str, char, Extra<'i>> {
            separator().with_ctx(Tokens::preset_standard().into())
        }

        assert!(parser_standard().parse("").has_errors());

        assert_eq!(parser_standard().parse(":").into_result(), Ok(':'));
        assert!(parser_standard().parse(";").has_errors());
    }

    #[test]
    fn test_prefix() {
        fn parser_standard<'i>()
        -> impl Parser<'i, &'i str, Prefix<'i>, Extra<'i>> {
            prefix().with_ctx(Tokens::preset_standard().into())
        }

        assert!(parser_standard().parse("").has_errors());

        assert_eq!(
            parser_standard().parse("add:").into_result(),
            Ok(Prefix {
                keyword: "add",
                modifier: None,
                enclosures: vec![]
            })
        );
        assert_eq!(
            parser_standard().parse("rem?(lib):").into_result(),
            Ok(Prefix {
                keyword: "rem",
                modifier: Some("?"),
                enclosures: vec![("lib", ['(', ')'])]
            })
        );
        assert_eq!(
            parser_standard().parse("ref!![eff]:").into_result(),
            Ok(Prefix {
                keyword: "ref",
                modifier: Some("!!"),
                enclosures: vec![("eff", ['[', ']'])]
            })
        );
        assert!(parser_standard().parse("add").has_errors());
        assert!(parser_standard().parse("feat:").has_errors());
        assert!(parser_standard().parse("add(exe)!:").has_errors());
    }
}
