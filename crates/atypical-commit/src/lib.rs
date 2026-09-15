//! Commit header parser. The grammar is data: [`Tokens`] lists the slots
//! in header order and [`prefix`] walks them through the parser context.
//! A [`config::CommitConfig`] lowers into [`Tokens`] with `try_from`.
//!
//! ```
//! use atypical_commit::{Extra, ExtraContext, Header, Tokens, header};
//! use chumsky::Parser;
//!
//! let context = ExtraContext::new(&Tokens::default()).unwrap();
//! let parser = header().with_ctx(context);
//! let parsed = Parser::<'_, _, Header, Extra>::parse(&parser, "add: x");
//!
//! assert_eq!(parsed.into_result().unwrap().prefix.keyword, "add");
//! ```

use chumsky::prelude::*;

pub mod config;
pub mod ignore;

/// Opening and closing delimiter, in that order.
pub type DelimitedBy = [char; 2];

#[doc(alias("Type", "Verb"))]
pub type Keyword<'i> = &'i str;

#[doc(alias("Importance", "BreakingChange"))]
pub type Modifier<'i> = &'i str;

#[doc(alias("Scope"))]
pub type Enclosure<'i> = (&'i str, DelimitedBy);

/// What the slots matched: `keyword` is the last word slot, `modifiers`
/// the first symbols slot present, `enclosures` every delimited slot
/// present. Separators are not kept.
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

/// What a bare slot's contents are made of.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Class {
    /// A run of alphanumerics and `_`.
    Word,
    /// A run of symbols.
    Symbols,
    /// Exactly one symbol.
    Symbol,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Delimited(DelimitedBy),
    Bare(Class),
}

/// A restrictable set of accepted spellings: anything, or a closed
/// list.
#[derive(Debug, Clone, PartialEq)]
pub enum Values {
    Any,
    Set(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    /// Points at the config that declared it, for errors the user can
    /// act on; diagnostics use the shape's noun instead.
    pub name: String,
    pub shape: Shape,
    pub values: Values,
    pub required: bool,
}

/// The header grammar: its slots, in header order.
#[derive(Debug, Clone, PartialEq)]
pub struct Tokens {
    pub slots: Vec<Slot>,
}

impl Default for Tokens {
    /// Unrestricted: any keyword, any modifier after free-form
    /// `(...)`/`[...]` enclosures, and any single-symbol separator.
    /// Only the header shape itself is enforced. Nothing declared these
    /// slots, so each is named for what it is.
    fn default() -> Self {
        let slot = |name: &str, shape, required| Slot {
            name: name.to_owned(),
            shape,
            values: Values::Any,
            required,
        };

        Self {
            slots: vec![
                slot("keywords", Shape::Bare(Class::Word), true),
                slot("(...)", Shape::Delimited(['(', ')']), false),
                slot("[...]", Shape::Delimited(['[', ']']), false),
                slot("modifiers", Shape::Bare(Class::Symbols), false),
                slot("separator", Shape::Bare(Class::Symbol), true),
            ],
        }
    }
}

pub type ExtraError<'i> = Rich<'i, char>;

pub type ExtraState<'i> = ();

#[doc(alias("Config", "Settings"))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraContext {
    pub tokens: Tokens,
}

/// A pair of neighbouring slots that cannot be told apart.
#[derive(Debug, Clone, PartialEq)]
pub enum Ambiguous {
    /// One takes the whole run the other needs.
    Run { first: String, second: String },
    /// Their spellings share a prefix.
    Prefix {
        first: String,
        second: String,
        spelling: String,
    },
    /// One holds spellings the other would take anyway.
    Shadowed { first: String, second: String },
    /// Two delimited slots carry the same pair of delimiters.
    Delimiters { first: String, second: String },
}

impl core::fmt::Display for Ambiguous {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Ambiguous::Run { first, second } => write!(
                f,
                "`{first}` takes the whole run, leaving nothing for `{second}`"
            ),
            Ambiguous::Prefix {
                first,
                second,
                spelling,
            } => write!(
                f,
                "`{first}` and `{second}` both start with `{spelling}`"
            ),
            Ambiguous::Shadowed { first, second } => write!(
                f,
                "`{second}` takes anything, `{first}`'s spellings included"
            ),
            Ambiguous::Delimiters { first, second } => {
                write!(f, "`{first}` and `{second}` carry the same delimiters")
            }
        }
    }
}

impl core::error::Error for Ambiguous {}

/// A bare slot is only locatable because its alphabet is disjoint from
/// its neighbour's, so the pairs are checked before any input is seen.
fn ambiguity(slots: &[Slot]) -> Option<Ambiguous> {
    for (index, first) in slots.iter().enumerate() {
        // An optional slot can be absent, which makes the slot behind
        // it a neighbour too, up to the first required one.
        for second in &slots[index + 1..] {
            if let Some(ambiguous) = ambiguous_pair(first, second) {
                return Some(ambiguous);
            }

            if second.required {
                break;
            }
        }

        // Delimiters are matched wherever they sit, so a repeat is
        // unreachable however far away it is. A shared opener with its
        // own closer still tells them apart.
        if !matches!(first.shape, Shape::Delimited(_)) {
            continue;
        }

        let repeat = slots[index + 1..]
            .iter()
            .find(|second| second.shape == first.shape);

        if let Some(second) = repeat {
            return Some(Ambiguous::Delimiters {
                first: first.name.clone(),
                second: second.name.clone(),
            });
        }
    }

    None
}

fn ambiguous_pair(first: &Slot, second: &Slot) -> Option<Ambiguous> {
    let (Shape::Bare(class), Shape::Bare(next)) = (first.shape, second.shape)
    else {
        return None;
    };

    let eats_run = matches!(
        (class, &first.values),
        (Class::Word, _) | (Class::Symbols, Values::Any)
    );

    if eats_run && class == next {
        return Some(Ambiguous::Run {
            first: first.name.clone(),
            second: second.name.clone(),
        });
    }

    // Symbols and symbol share an alphabet, so a closed set in front of
    // one that takes anything has no spelling of its own.
    let alike = matches!(
        (class, next),
        (Class::Word, Class::Word)
            | (
                Class::Symbols | Class::Symbol,
                Class::Symbols | Class::Symbol
            )
    );

    if alike
        && matches!(first.values, Values::Set(_))
        && matches!(second.values, Values::Any)
    {
        return Some(Ambiguous::Shadowed {
            first: first.name.clone(),
            second: second.name.clone(),
        });
    }

    let (Values::Set(spellings), Values::Set(next_spellings)) =
        (&first.values, &second.values)
    else {
        return None;
    };

    for spelling in spellings {
        for next_spelling in next_spellings {
            let shared = if spelling.starts_with(next_spelling.as_str()) {
                next_spelling
            } else if next_spelling.starts_with(spelling.as_str()) {
                spelling
            } else {
                continue;
            };

            return Some(Ambiguous::Prefix {
                first: first.name.clone(),
                second: second.name.clone(),
                spelling: shared.clone(),
            });
        }
    }

    None
}

impl ExtraContext {
    /// Rejects slots that cannot be told apart, and orders every bare set
    /// longest first.
    pub fn new(tokens: &Tokens) -> Result<Self, Ambiguous> {
        if let Some(ambiguous) = ambiguity(&tokens.slots) {
            return Err(ambiguous);
        }

        let mut tokens = tokens.clone();

        // Bare slots match by prefix, so `!!` must be tried before `!`.
        for slot in &mut tokens.slots {
            if let (Shape::Bare(_), Values::Set(set)) =
                (slot.shape, &mut slot.values)
            {
                set.sort_unstable_by(|a, b| {
                    b.len().cmp(&a.len()).then(a.cmp(b))
                });
            }
        }

        Ok(Self { tokens })
    }
}

impl Default for ExtraContext {
    /// The unrestricted grammar, which chumsky builds per `parse` call
    /// and `with_ctx` replaces. It skips `new`, whose validation can
    /// fail, and has nothing to validate or sort: every slot takes
    /// anything.
    fn default() -> Self {
        Self {
            tokens: Tokens::default(),
        }
    }
}

#[doc(alias("Config", "Settings"))]
pub type Extra<'i> = extra::Full<ExtraError<'i>, ExtraState<'i>, ExtraContext>;

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

/// A visible char outside the word alphabet.
pub(crate) fn is_symbol(c: char) -> bool {
    !c.is_alphanumeric() && c != '_' && !c.is_whitespace()
}

/// The word diagnostics use for a slot of this shape.
fn noun(shape: Shape) -> &'static str {
    match shape {
        Shape::Bare(Class::Word) => "keyword",
        Shape::Bare(Class::Symbols) => "modifier",
        Shape::Bare(Class::Symbol) => "separator",
        Shape::Delimited(_) => "enclosure",
    }
}

fn expected_one_of(found: &str, kind: &str, expected: &[String]) -> String {
    let expected = expected.join(", ");

    if found.is_empty() {
        format!("expected {kind}, one of: {expected}")
    } else {
        format!("unknown {kind} `{found}`, expected one of: {expected}")
    }
}

fn one_of<'i>(
    set: &[String],
) -> impl Parser<'i, &'i str, &'i str, Extra<'i>> + use<'i> {
    let parsers = set
        .iter()
        .map(|token| just(token.clone()))
        .collect::<Vec<_>>();

    choice(parsers).to_slice()
}

fn word<'i>(
    slot: &Slot,
) -> impl Parser<'i, &'i str, &'i str, Extra<'i>> + use<'i> {
    use chumsky::input::InputRef;

    let noun = noun(slot.shape);
    let Slot { values, .. } = slot.clone();

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        let (s, span) = ident(i);

        match &values {
            Values::Any if !s.is_empty() => Ok(s),
            Values::Any => {
                Err(Rich::custom(span, format!("expected a {noun}")))
            }
            Values::Set(set) if set.iter().any(|value| value == s) => Ok(s),
            Values::Set(set) => {
                let message = expected_one_of(s, noun, set);

                Err(Rich::custom(span, message))
            }
        }
    })
}

/// A run of symbols that leaves the separator and every enclosure
/// opener to the slots around it.
fn symbols<'i>(
    slot: &Slot,
    separator: Option<Values>,
    openers: Vec<char>,
) -> impl Parser<'i, &'i str, &'i str, Extra<'i>> + use<'i> {
    use chumsky::input::InputRef;

    let noun = noun(slot.shape);
    let Slot { values, .. } = slot.clone();

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        if let Values::Set(set) = &values {
            return i.parse(one_of(set));
        }

        let before = i.cursor();

        // Without a separator there is nothing to stop the run, so it
        // would eat the rest of the header.
        let Some(separator) = &separator else {
            let span = i.span_since(&before);
            let message = format!("a {noun} needs a separator after it");

            return Err(Rich::custom(span, message));
        };

        while let Some(c) = i.peek() {
            if !is_symbol(c) || openers.contains(&c) {
                break;
            }

            match separator {
                Values::Set(set) if set.iter().any(|s| s.starts_with(c)) => {
                    break;
                }
                // With an unrestricted separator, the last symbol
                // of the run is the separator, not the modifier.
                Values::Any => {
                    let checkpoint = i.save();

                    i.next();

                    if !i.peek().is_some_and(is_symbol) {
                        i.rewind(checkpoint);
                        break;
                    }
                }
                Values::Set(_) => {
                    i.next();
                }
            }
        }

        let s = i.slice_since(&before..);

        if s.is_empty() {
            let span = i.span_since(&before);

            return Err(Rich::custom(span, format!("expected a {noun}")));
        }

        Ok(s)
    })
}

fn symbol<'i>(
    slot: &Slot,
) -> impl Parser<'i, &'i str, &'i str, Extra<'i>> + use<'i> {
    use chumsky::input::InputRef;

    let noun = noun(slot.shape);
    let Slot { values, .. } = slot.clone();

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        if let Values::Set(set) = &values {
            return i.parse(one_of(set));
        }

        let before = i.cursor();

        match i.peek() {
            Some(c) if is_symbol(c) => {
                i.next();

                Ok(i.slice_since(&before..))
            }
            _ => Err(Rich::custom(
                i.span_since(&before),
                format!("expected a {noun}"),
            )),
        }
    })
}

fn bare<'i>(
    class: Class,
    slot: &Slot,
    separator: &Option<Values>,
    openers: &[char],
) -> Boxed<'i, 'i, &'i str, &'i str, Extra<'i>> {
    match class {
        Class::Word => word(slot).boxed(),
        Class::Symbols => {
            symbols(slot, separator.clone(), openers.to_vec()).boxed()
        }
        Class::Symbol => symbol(slot).boxed(),
    }
}

/// A run of delimited slots, in order and each at most once.
fn enclosures<'i>(
    run: Vec<(DelimitedBy, Slot)>,
) -> impl Parser<'i, &'i str, Vec<Enclosure<'i>>, Extra<'i>> {
    use chumsky::input::InputRef;

    fn parser<'i>(
        [start, end]: DelimitedBy,
        slot: &Slot,
    ) -> impl Parser<'i, &'i str, Enclosure<'i>, Extra<'i>> {
        match &slot.values {
            Values::Any => none_of::<'i, _, _, Extra>([start, end])
                .repeated()
                .to_slice()
                .delimited_by(just(start), just(end))
                .map(move |s| (s, [start, end]))
                .boxed(),
            Values::Set(allowed) => {
                let allowed = allowed.clone();
                let noun = noun(slot.shape);

                custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
                    let (s, span) = ident(i);

                    if allowed.iter().any(|value| value == s) {
                        return Ok(s);
                    }

                    let message = expected_one_of(s, noun, &allowed);

                    Err(Rich::custom(span, message))
                })
                .delimited_by(just(start), just(end))
                .map(move |s| (s, [start, end]))
                .boxed()
            }
        }
    }

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        let mut index = 0;
        let mut results = Vec::new();

        while index < run.len() {
            let before = i.cursor();
            let next = i.peek();
            let is_open = run[index..]
                .iter()
                .any(|([open, _], _)| Some(*open) == next);

            if !is_open {
                break;
            }

            let parsers = run[index..]
                .iter()
                .map(|(delimiters, slot)| parser(*delimiters, slot))
                .collect::<Vec<_>>();

            let (content, delimited_by) = i.parse(choice(parsers))?;
            // The position is within what is left of the run, since
            // everything before `index` is already spoken for.
            let position = run[index..]
                .iter()
                .position(|(delimiters, _)| *delimiters == delimited_by)
                .unwrap();
            let skipped = run[index..index + position]
                .iter()
                .find(|(_, slot)| slot.required);

            if let Some(([open, _], _)) = skipped {
                let message = format!("expected an opening `{open}`");

                return Err(Rich::custom(i.span_since(&before), message));
            }

            index += position + 1;
            results.push((content, delimited_by));
        }

        if let Some(([open, _], _)) =
            run[index..].iter().find(|(_, slot)| slot.required)
        {
            let here = i.cursor();
            let message = format!("expected an opening `{open}`");

            return Err(Rich::custom(i.span_since(&here), message));
        }

        Ok(results)
    })
}

/// One space, then the rest of the line with trailing whitespace trimmed.
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

/// Walks the slots in header order.
pub fn prefix<'i>() -> impl Parser<'i, &'i str, Prefix<'i>, Extra<'i>> {
    use chumsky::input::InputRef;

    custom(|i: &mut InputRef<&'i str, Extra<'i>>| {
        let slots = i.ctx().tokens.slots.clone();
        let separator = slots
            .iter()
            .find(|slot| slot.shape == Shape::Bare(Class::Symbol))
            .map(|slot| slot.values.clone());
        let openers = slots
            .iter()
            .filter_map(|slot| match slot.shape {
                Shape::Delimited([open, _]) => Some(open),
                Shape::Bare(_) => None,
            })
            .collect::<Vec<_>>();

        let mut prefix = Prefix {
            keyword: "",
            modifier: None,
            enclosures: Vec::new(),
        };
        let mut rest = slots.as_slice();

        while let Some(slot) = rest.first() {
            let Shape::Bare(class) = slot.shape else {
                let run = rest
                    .iter()
                    .map_while(|slot| match slot.shape {
                        Shape::Delimited(delimiters) => {
                            Some((delimiters, slot.clone()))
                        }
                        Shape::Bare(_) => None,
                    })
                    .collect::<Vec<_>>();

                rest = &rest[run.len()..];
                prefix.enclosures.extend(i.parse(enclosures(run))?);
                continue;
            };

            let parser = bare(class, slot, &separator, &openers);
            let s = if slot.required {
                Some(i.parse(parser)?)
            } else {
                i.parse(parser.or_not())?
            };

            match class {
                Class::Word => prefix.keyword = s.unwrap_or_default(),
                Class::Symbols => prefix.modifier = prefix.modifier.or(s),
                Class::Symbol => {}
            }

            rest = &rest[1..];
        }

        Ok(prefix)
    })
}

/// Parses against the context given through `with_ctx`, or the
/// unrestricted grammar without one.
pub fn header<'i>() -> impl Parser<'i, &'i str, Header<'i>, Extra<'i>> {
    group((prefix(), description())).map(|(prefix, description)| Header {
        prefix,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(name: &str, shape: Shape, values: Values) -> Slot {
        Slot {
            name: name.to_owned(),
            shape,
            values,
            required: false,
        }
    }

    fn set(spellings: &[&str]) -> Values {
        Values::Set(spellings.iter().map(|s| (*s).to_owned()).collect())
    }

    fn context(slots: Vec<Slot>) -> Result<ExtraContext, Ambiguous> {
        ExtraContext::new(&Tokens { slots })
    }

    #[test]
    fn test_the_default_context_is_unrestricted() {
        assert_eq!(ExtraContext::default().tokens, Tokens::default());
        assert!(!header().parse("add(lib)!: x").has_errors());
    }

    #[test]
    fn test_lowered_layouts_are_unambiguous() {
        assert!(ExtraContext::new(&Tokens::default()).is_ok());
    }

    #[test]
    fn test_ambiguity_says_which_pair() {
        let run = Ambiguous::Run {
            first: "modifiers".to_owned(),
            second: "flag".to_owned(),
        };

        assert_eq!(
            run.to_string(),
            "`modifiers` takes the whole run, leaving nothing for `flag`"
        );

        let prefix = Ambiguous::Prefix {
            first: "modifiers".to_owned(),
            second: "separator".to_owned(),
            spelling: "!".to_owned(),
        };

        assert_eq!(
            prefix.to_string(),
            "`modifiers` and `separator` both start with `!`"
        );

        let shadowed = Ambiguous::Shadowed {
            first: "modifiers".to_owned(),
            second: "separator".to_owned(),
        };

        assert_eq!(
            shadowed.to_string(),
            "`separator` takes anything, `modifiers`'s spellings included"
        );

        let delimiters = Ambiguous::Delimiters {
            first: "scope".to_owned(),
            second: "reason".to_owned(),
        };

        assert_eq!(
            delimiters.to_string(),
            "`scope` and `reason` carry the same delimiters"
        );
    }

    #[test]
    fn test_a_word_leaves_nothing_for_a_word() {
        assert_eq!(
            context(vec![
                slot("keywords", Shape::Bare(Class::Word), Values::Any),
                slot("scope", Shape::Bare(Class::Word), set(&["lib"])),
            ]),
            Err(Ambiguous::Run {
                first: "keywords".to_owned(),
                second: "scope".to_owned(),
            })
        );
    }

    #[test]
    fn test_an_unrestricted_run_leaves_nothing_for_a_run() {
        assert_eq!(
            context(vec![
                slot("modifiers", Shape::Bare(Class::Symbols), Values::Any),
                slot("flag", Shape::Bare(Class::Symbols), set(&["~"])),
            ]),
            Err(Ambiguous::Run {
                first: "modifiers".to_owned(),
                second: "flag".to_owned(),
            })
        );
    }

    #[test]
    fn test_a_single_symbol_may_follow_a_run() {
        assert!(
            context(vec![
                slot("modifiers", Shape::Bare(Class::Symbols), Values::Any),
                slot("separator", Shape::Bare(Class::Symbol), Values::Any),
            ])
            .is_ok()
        );
    }

    #[test]
    fn test_the_longer_spelling_may_come_second() {
        assert_eq!(
            context(vec![
                slot("modifiers", Shape::Bare(Class::Symbols), set(&["!"])),
                slot("flag", Shape::Bare(Class::Symbols), set(&["!!"])),
            ]),
            Err(Ambiguous::Prefix {
                first: "modifiers".to_owned(),
                second: "flag".to_owned(),
                spelling: "!".to_owned(),
            })
        );
    }

    #[test]
    fn test_neighbouring_sets_may_not_share_a_prefix() {
        assert_eq!(
            context(vec![
                slot(
                    "modifiers",
                    Shape::Bare(Class::Symbols),
                    set(&["!", "!!"])
                ),
                slot("separator", Shape::Bare(Class::Symbol), set(&["!"])),
            ]),
            Err(Ambiguous::Prefix {
                first: "modifiers".to_owned(),
                second: "separator".to_owned(),
                spelling: "!".to_owned(),
            })
        );
    }

    #[test]
    fn test_a_required_slot_separates_two_runs() {
        let mut scope =
            slot("enclosures[0]", Shape::Delimited(['(', ')']), Values::Any);

        scope.required = true;

        assert!(
            context(vec![
                slot("keywords", Shape::Bare(Class::Word), Values::Any),
                scope,
                slot("reason", Shape::Bare(Class::Word), Values::Any),
            ])
            .is_ok()
        );
    }

    #[test]
    fn test_an_optional_slot_between_separates_nothing() {
        assert_eq!(
            context(vec![
                slot(
                    "modifiers (pre)",
                    Shape::Bare(Class::Symbols),
                    Values::Any
                ),
                slot(
                    "enclosures[0]",
                    Shape::Delimited(['(', ')']),
                    Values::Any
                ),
                slot(
                    "modifiers (post)",
                    Shape::Bare(Class::Symbols),
                    Values::Any
                ),
            ]),
            Err(Ambiguous::Run {
                first: "modifiers (pre)".to_owned(),
                second: "modifiers (post)".to_owned(),
            })
        );
    }
}
