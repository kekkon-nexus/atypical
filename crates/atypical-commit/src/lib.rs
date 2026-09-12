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
    /// Names the slot in diagnostics.
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
    /// Unrestricted: any keyword, any modifier on either side of
    /// free-form `(...)`/`[...]` enclosures, and any single-symbol
    /// separator. Only the header shape itself is enforced.
    fn default() -> Self {
        (&config::CommitConfig::default()).into()
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
        }
    }
}

impl core::error::Error for Ambiguous {}

/// A bare slot is only locatable because its alphabet is disjoint from
/// its neighbour's, so the pairs are checked before any input is seen.
fn ambiguity(slots: &[Slot]) -> Option<Ambiguous> {
    for pair in slots.windows(2) {
        let [first, second] = pair else { continue };

        let (Shape::Bare(class), Shape::Bare(next)) =
            (first.shape, second.shape)
        else {
            continue;
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

        let (Values::Set(spellings), Values::Set(next_spellings)) =
            (&first.values, &second.values)
        else {
            continue;
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
    }

    None
}

impl ExtraContext {
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
    fn default() -> Self {
        Self::new(&Tokens::default())
            .expect("the unrestricted grammar is unambiguous")
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

/// A visible char that can't belong to a keyword or a description.
fn is_symbol(c: char) -> bool {
    !c.is_alphanumeric() && c != '_' && !c.is_whitespace()
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

    let Slot { name, values, .. } = slot.clone();

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        let (s, span) = ident(i);

        match &values {
            Values::Any if !s.is_empty() => Ok(s),
            Values::Any => {
                Err(Rich::custom(span, format!("expected a {name}")))
            }
            Values::Set(set) if set.iter().any(|value| value == s) => Ok(s),
            Values::Set(set) => {
                let message = expected_one_of(s, &name, set);

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

    let Slot { name, values, .. } = slot.clone();

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        if let Values::Set(set) = &values {
            return i.parse(one_of(set));
        }

        let before = i.cursor();

        // Without a separator there is nothing to stop the run, so it
        // would eat the rest of the header.
        let Some(separator) = &separator else {
            let span = i.span_since(&before);
            let message = format!("a {name} needs a separator after it");

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

            return Err(Rich::custom(span, format!("expected a {name}")));
        }

        Ok(s)
    })
}

fn symbol<'i>(
    slot: &Slot,
) -> impl Parser<'i, &'i str, &'i str, Extra<'i>> + use<'i> {
    use chumsky::input::InputRef;

    let Slot { name, values, .. } = slot.clone();

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
                format!("expected a {name}"),
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

/// A run of delimited slots, each optional and at most once, in order.
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
                let name = slot.name.clone();

                custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
                    let (s, span) = ident(i);

                    if allowed.iter().any(|value| value == s) {
                        return Ok(s);
                    }

                    let message = expected_one_of(s, &name, &allowed);

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
            let position = run
                .iter()
                .position(|(delimiters, _)| *delimiters == delimited_by)
                .unwrap();
            index += position + 1;
            results.push((content, delimited_by));
        }

        Ok(results)
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
    fn test_lowered_layouts_are_unambiguous() {
        assert!(ExtraContext::new(&Tokens::default()).is_ok());
    }

    #[test]
    fn test_ambiguity_says_which_pair() {
        let run = Ambiguous::Run {
            first: "modifier".to_owned(),
            second: "flag".to_owned(),
        };

        assert_eq!(
            run.to_string(),
            "`modifier` takes the whole run, leaving nothing for `flag`"
        );

        let prefix = Ambiguous::Prefix {
            first: "modifier".to_owned(),
            second: "separator".to_owned(),
            spelling: "!".to_owned(),
        };

        assert_eq!(
            prefix.to_string(),
            "`modifier` and `separator` both start with `!`"
        );
    }

    #[test]
    fn test_a_word_leaves_nothing_for_a_word() {
        assert_eq!(
            context(vec![
                slot("keyword", Shape::Bare(Class::Word), Values::Any),
                slot("scope", Shape::Bare(Class::Word), set(&["lib"])),
            ]),
            Err(Ambiguous::Run {
                first: "keyword".to_owned(),
                second: "scope".to_owned(),
            })
        );
    }

    #[test]
    fn test_an_unrestricted_run_leaves_nothing_for_a_run() {
        assert_eq!(
            context(vec![
                slot("modifier", Shape::Bare(Class::Symbols), Values::Any),
                slot("flag", Shape::Bare(Class::Symbols), set(&["~"])),
            ]),
            Err(Ambiguous::Run {
                first: "modifier".to_owned(),
                second: "flag".to_owned(),
            })
        );
    }

    #[test]
    fn test_a_single_symbol_may_follow_a_run() {
        assert!(
            context(vec![
                slot("modifier", Shape::Bare(Class::Symbols), Values::Any),
                slot("separator", Shape::Bare(Class::Symbol), Values::Any),
            ])
            .is_ok()
        );
    }

    #[test]
    fn test_neighbouring_sets_may_not_share_a_prefix() {
        assert_eq!(
            context(vec![
                slot(
                    "modifier",
                    Shape::Bare(Class::Symbols),
                    set(&["!", "!!"])
                ),
                slot("separator", Shape::Bare(Class::Symbol), set(&["!"])),
            ]),
            Err(Ambiguous::Prefix {
                first: "modifier".to_owned(),
                second: "separator".to_owned(),
                spelling: "!".to_owned(),
            })
        );
    }

    #[test]
    fn test_a_delimited_slot_separates_two_runs() {
        assert!(
            context(vec![
                slot("keyword", Shape::Bare(Class::Word), Values::Any),
                slot("scope", Shape::Delimited(['(', ')']), Values::Any),
                slot("reason", Shape::Bare(Class::Word), Values::Any),
            ])
            .is_ok()
        );
    }
}
