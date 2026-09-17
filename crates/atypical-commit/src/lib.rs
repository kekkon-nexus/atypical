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
//! let prefix = parsed.into_result().unwrap().prefix;
//!
//! assert_eq!((prefix[0].name.as_str(), prefix[0].value), ("keywords", "add"));
//! ```

use chumsky::prelude::*;

pub mod config;
pub mod ignore;

/// Opening and closing delimiter, in that order.
pub type DelimitedBy = [char; 2];

/// What one slot matched. A delimited slot's value and span leave out
/// its delimiters.
#[derive(Debug, Clone, PartialEq)]
pub struct Part<'i> {
    pub name: String,
    pub value: &'i str,
    pub span: SimpleSpan,
}

/// Every slot present, in header order.
pub type Prefix<'i> = Vec<Part<'i>>;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Delimited(DelimitedBy),
    Bare(Class),
    /// Exactly one of these options, each named and shaped as a slot of
    /// its own. Whether it is required, has a gap or is tight is the
    /// outer slot's.
    OneOf(Vec<Slot>),
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
    /// What diagnostics and the config that declared it call the slot.
    pub name: String,
    pub shape: Shape,
    pub values: Values,
    pub required: bool,
    /// One space after the slot, owed by whichever slot comes next, is
    /// present and takes it. With none, the description's own space
    /// serves.
    pub gap: bool,
    /// Attaches to whatever precedes it: never takes an owed space,
    /// and passes the debt on to the slot behind it.
    pub tight: bool,
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
            gap: false,
            tight: false,
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
    /// Two options of one slot can start on the same input.
    Overlap { first: String, second: String },
    /// An optional bare slot behind a gap, where the space is also the
    /// description's and a word or free symbols read as the slot.
    Description { slot: String },
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
            Ambiguous::Overlap { first, second } => {
                write!(
                    f,
                    "`{first}` and `{second}` can start on the same input"
                )
            }
            Ambiguous::Description { slot } => write!(
                f,
                "`{slot}` is optional after a gap, so it can take the start \
                 of the description"
            ),
        }
    }
}

impl core::error::Error for Ambiguous {}

/// A bare slot is only locatable because its alphabet is disjoint from
/// its neighbour's, so the pairs are checked before any input is seen.
fn ambiguity(slots: &[Slot]) -> Option<Ambiguous> {
    for (index, slot) in slots.iter().enumerate() {
        // A delimited slot commits on its opener and a closed set of
        // symbols is not prose, so only these are mistaken for the
        // description.
        let prose = |form: &Slot| {
            matches!(
                (&form.shape, &form.values),
                (Shape::Bare(Class::Word), _) | (Shape::Bare(_), Values::Any)
            )
        };
        let landing = slots[index + 1..]
            .iter()
            .take_while(|_| slot.gap)
            .take_while(|next| !next.required)
            .filter(|next| !next.tight)
            .find(|next| forms(next).iter().any(prose));

        if let Some(next) = landing {
            return Some(Ambiguous::Description {
                slot: next.name.clone(),
            });
        }

        if let Shape::OneOf(options) = &slot.shape {
            for (at, first) in options.iter().enumerate() {
                let overlapping = options[at + 1..]
                    .iter()
                    .find(|second| overlap(first, second));

                if let Some(second) = overlapping {
                    return Some(Ambiguous::Overlap {
                        first: first.name.clone(),
                        second: second.name.clone(),
                    });
                }
            }
        }

        for first in forms(slot) {
            // An optional slot can be absent, which makes the slot
            // behind it a neighbour too, up to the first required one.
            for next in &slots[index + 1..] {
                for second in forms(next) {
                    if let Some(ambiguous) = ambiguous_pair(first, second) {
                        return Some(ambiguous);
                    }
                }

                if next.required {
                    break;
                }
            }

            // Delimiters are matched wherever they sit, so a repeat is
            // unreachable however far away it is. A shared opener with
            // its own closer still tells them apart.
            if !matches!(first.shape, Shape::Delimited(_)) {
                continue;
            }

            let repeat = slots[index + 1..]
                .iter()
                .flat_map(forms)
                .find(|second| second.shape == first.shape);

            if let Some(second) = repeat {
                return Some(Ambiguous::Delimiters {
                    first: first.name.clone(),
                    second: second.name.clone(),
                });
            }
        }
    }

    None
}

/// The shapes a slot can take where it sits: its options, or itself.
pub(crate) fn forms(slot: &Slot) -> &[Slot] {
    match &slot.shape {
        Shape::OneOf(options) => options,
        _ => std::slice::from_ref(slot),
    }
}

fn starts_class(class: Class, c: char) -> bool {
    match class {
        Class::Word => c.is_alphanumeric() || c == '_',
        Class::Symbols | Class::Symbol => is_symbol(c),
    }
}

/// Whether two options of one slot could both start on some input,
/// leaving the one tried second unreachable there.
fn overlap(first: &Slot, second: &Slot) -> bool {
    let alike = |class: Class, next: Class| {
        (class == Class::Word) == (next == Class::Word)
    };

    match (&first.shape, &second.shape) {
        (Shape::Delimited([open, _]), Shape::Delimited([next, _])) => {
            open == next
        }
        (&Shape::Delimited([open, _]), &Shape::Bare(class))
        | (&Shape::Bare(class), &Shape::Delimited([open, _])) => {
            let bare = if matches!(first.shape, Shape::Bare(_)) {
                first
            } else {
                second
            };

            match &bare.values {
                Values::Any => starts_class(class, open),
                Values::Set(set) => set.iter().any(|s| s.starts_with(open)),
            }
        }
        (&Shape::Bare(class), &Shape::Bare(next)) if alike(class, next) => {
            match (&first.values, &second.values) {
                (Values::Set(set), Values::Set(next)) => set.iter().any(|s| {
                    next.iter().any(|n| {
                        s.starts_with(n.as_str()) || n.starts_with(s.as_str())
                    })
                }),
                _ => true,
            }
        }
        _ => false,
    }
}

fn ambiguous_pair(first: &Slot, second: &Slot) -> Option<Ambiguous> {
    let (&Shape::Bare(class), &Shape::Bare(next)) =
        (&first.shape, &second.shape)
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

        longest_first(&mut tokens.slots);

        Ok(Self { tokens })
    }
}

/// Bare slots match by prefix, so `!!` must be tried before `!`.
fn longest_first(slots: &mut [Slot]) {
    for slot in slots {
        match (&mut slot.shape, &mut slot.values) {
            (Shape::OneOf(options), _) => longest_first(options),
            (Shape::Bare(_), Values::Set(set)) => {
                set.sort_unstable_by(|a, b| {
                    b.len().cmp(&a.len()).then(a.cmp(b))
                });
            }
            _ => {}
        }
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

fn expected_one_of(found: &str, name: &str, expected: &[String]) -> String {
    let expected = expected.join(", ");

    if found.is_empty() {
        format!("expected `{name}`, one of: {expected}")
    } else {
        format!("`{found}` is not in `{name}`, expected one of: {expected}")
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
                Err(Rich::custom(span, format!("expected `{name}`")))
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
            let message = format!("`{name}` needs a separator after it");

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

            return Err(Rich::custom(span, format!("expected `{name}`")));
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
                format!("expected `{name}`"),
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

fn opening([open, _]: DelimitedBy, spaced: bool) -> String {
    let gap = if spaced { " " } else { "" };

    format!("expected an opening `{gap}{open}`")
}

fn enclosure<'i>(
    [start, end]: DelimitedBy,
    slot: &Slot,
) -> impl Parser<'i, &'i str, (&'i str, SimpleSpan, DelimitedBy), Extra<'i>> + use<'i>
{
    use chumsky::input::InputRef;

    let contents = match &slot.values {
        Values::Any => {
            let Slot { name, .. } = slot.clone();

            none_of::<'i, _, _, Extra>([start, end])
                .repeated()
                .at_least(1)
                .to_slice()
                .map_err(move |error: Rich<'i, char>| {
                    Rich::custom(*error.span(), format!("expected `{name}`"))
                })
                .boxed()
        }
        Values::Set(allowed) => {
            let Slot { name, .. } = slot.clone();
            let allowed = allowed.clone();

            custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
                // Word chars plus hyphen, so a shortcode like `t-rex`
                // can match against the set; a non-word char still ends
                // the read, keeping the empty-value error for a bad open.
                let before = i.cursor();

                while i.peek().is_some_and(|c: char| {
                    c.is_alphanumeric() || c == '_' || c == '-'
                }) {
                    i.next();
                }

                let s = i.slice_since(&before..);
                let span = i.span_since(&before);

                if allowed.iter().any(|value| value == s) {
                    return Ok(s);
                }

                let message = expected_one_of(s, &name, &allowed);

                Err(Rich::custom(span, message))
            })
            .boxed()
        }
    };

    contents
        .map_with(|s, e| (s, e.span()))
        .delimited_by(just(start), just(end))
        .map(move |(s, span)| (s, span, [start, end]))
}

/// One slot outside a run of enclosures.
fn single<'i>(
    slot: &Slot,
    separator: &Option<Values>,
    openers: &[char],
) -> Boxed<'i, 'i, &'i str, (&'i str, SimpleSpan), Extra<'i>> {
    match &slot.shape {
        Shape::Bare(class) => bare(*class, slot, separator, openers)
            .map_with(|s, e| (s, e.span()))
            .boxed(),
        Shape::Delimited(delimiters) => enclosure(*delimiters, slot)
            .map(|(s, span, _)| (s, span))
            .boxed(),
        Shape::OneOf(options) => choice(
            options
                .iter()
                .map(|option| single(option, separator, openers))
                .collect::<Vec<_>>(),
        )
        .boxed(),
    }
}

/// A run of delimited slots, in order and each at most once. `spaced`
/// is whether a gap is owed before the first one present; what comes
/// back is whether one is still owed after the run.
fn enclosures<'i>(
    run: Vec<(DelimitedBy, Slot)>,
    openers: Vec<char>,
    spaced: bool,
) -> impl Parser<'i, &'i str, (Vec<Part<'i>>, bool), Extra<'i>> {
    use chumsky::input::InputRef;

    custom(move |i: &mut InputRef<&'i str, Extra<'i>>| {
        let mut index = 0;
        let mut results = Vec::new();
        let mut spaced = spaced;

        while index < run.len() {
            let before = i.cursor();
            let checkpoint = i.save();

            let gapped = spaced && i.peek() == Some(' ');

            if gapped {
                i.next();
            }

            let next = i.peek();
            let eligible = |slot: &Slot| {
                if slot.tight {
                    !gapped
                } else {
                    gapped == spaced
                }
            };
            let is_open = run[index..]
                .iter()
                .any(|([open, _], slot)| Some(*open) == next && eligible(slot));

            if !is_open {
                i.rewind(checkpoint);
                break;
            }

            let parsers = choice(
                run[index..]
                    .iter()
                    .filter(|(_, slot)| eligible(slot))
                    .map(|(delimiters, slot)| enclosure(*delimiters, slot))
                    .collect::<Vec<_>>(),
            );

            // Seeing the opener commits to the slot, so a bad value
            // errors instead of backtracking into the description. An
            // opener another slot outside the run shares is no such
            // promise, since the value may yet be that slot's, so it is
            // tried without committing, as an optional slot is; sharing
            // within the run is a promise, as `choice` tries each. This
            // matches a lone slot, which commits only on a unique opener,
            // so the run's order does not decide it.
            let count = |it: &mut dyn Iterator<Item = char>| {
                it.filter(|open| Some(*open) == next).count()
            };
            let outside = count(&mut openers.iter().copied())
                - count(&mut run.iter().map(|([open, _], _)| *open));
            let shared = outside > 0;
            let matched = if shared {
                // `or_not` recovers the inner failure, so it never errors.
                i.parse(parsers.or_not()).unwrap_or(None)
            } else {
                Some(i.parse(parsers)?)
            };

            let Some((value, span, delimited_by)) = matched else {
                i.rewind(checkpoint);
                break;
            };
            // The position is within what is left of the run, since
            // everything before `index` is already spoken for.
            let position = run[index..]
                .iter()
                .position(|(delimiters, _)| *delimiters == delimited_by)
                .unwrap();
            let skipped = run[index..index + position]
                .iter()
                .find(|(_, slot)| slot.required);

            if let Some((delimiters, slot)) = skipped {
                let message = opening(*delimiters, spaced && !slot.tight);

                return Err(Rich::custom(i.span_since(&before), message));
            }

            let slot = &run[index + position].1;

            results.push(Part {
                name: slot.name.clone(),
                value,
                span,
            });
            spaced = slot.gap || (slot.tight && spaced);
            index += position + 1;
        }

        if let Some((delimiters, slot)) =
            run[index..].iter().find(|(_, slot)| slot.required)
        {
            let here = i.cursor();
            let message = opening(*delimiters, spaced && !slot.tight);

            return Err(Rich::custom(i.span_since(&here), message));
        }

        Ok((results, spaced))
    })
}

/// One space, then the rest of the line with trailing whitespace
/// trimmed. A second leading space is refused: it is not part of the
/// separator's one space, so it would otherwise be a slot that fell open
/// into the description.
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

        if rest.starts_with(char::is_whitespace) {
            return Err(Rich::custom(
                span,
                "the description must not start with a space",
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
        let openers = slots
            .iter()
            .flat_map(forms)
            .filter_map(|slot| match slot.shape {
                Shape::Delimited([open, _]) => Some(open),
                _ => None,
            })
            .collect::<Vec<_>>();

        let mut prefix = Prefix::new();
        let mut rest = slots.as_slice();
        // A gap belongs to the slot before it but is taken by the next
        // one present, so an absent slot passes it on.
        let mut spaced = false;

        while let Some(slot) = rest.first() {
            if let Shape::Delimited(_) = slot.shape {
                let run = rest
                    .iter()
                    .map_while(|slot| match slot.shape {
                        Shape::Delimited(delimiters) => {
                            Some((delimiters, slot.clone()))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                rest = &rest[run.len()..];

                let (parts, owed) =
                    i.parse(enclosures(run, openers.clone(), spaced))?;

                prefix.extend(parts);
                spaced = owed;
                continue;
            }

            // A `symbols` run stops for the separator that follows it,
            // which is the nearest `symbol` slot after this one, not the
            // first in the grammar.
            let separator = rest
                .iter()
                .skip(1)
                .find(|slot| slot.shape == Shape::Bare(Class::Symbol))
                .map(|slot| slot.values.clone());
            let mut parser = single(slot, &separator, &openers);
            let owed = spaced && !slot.tight;

            if owed {
                parser = just(' ').ignore_then(parser).boxed();
            }

            // An opener commits an optional slot, as within a run of
            // enclosures, so a bad value errors instead of backtracking
            // into the description. An opener another slot shares is no
            // such promise: the value may yet be that slot's.
            let checkpoint = i.save();

            if owed && i.peek() == Some(' ') {
                i.next();
            }

            let next = i.peek();
            let shared = openers.iter().filter(|open| Some(**open) == next);
            let opened = shared.count() == 1
                && forms(slot).iter().any(|form| {
                    matches!(form.shape, Shape::Delimited([open, _])
                        if Some(open) == next)
                });

            i.rewind(checkpoint);

            let matched = if slot.required || opened {
                Some(i.parse(parser)?)
            } else {
                // `or_not` recovers the inner failure, so it never errors.
                i.parse(parser.or_not()).unwrap_or(None)
            };

            if let Some((value, span)) = matched {
                prefix.push(Part {
                    name: slot.name.clone(),
                    value,
                    span,
                });
                spaced = slot.gap || (slot.tight && spaced);
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
            gap: false,
            tight: false,
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

        let overlap = Ambiguous::Overlap {
            first: "emoji".to_owned(),
            second: "shortcode".to_owned(),
        };

        assert_eq!(
            overlap.to_string(),
            "`emoji` and `shortcode` can start on the same input"
        );
    }

    fn options(options: Vec<Slot>) -> Result<ExtraContext, Ambiguous> {
        context(vec![slot("intention", Shape::OneOf(options), Values::Any)])
    }

    fn overlap(first: &str, second: &str) -> Result<ExtraContext, Ambiguous> {
        Err(Ambiguous::Overlap {
            first: first.to_owned(),
            second: second.to_owned(),
        })
    }

    #[test]
    fn test_prose_behind_a_gap_takes_the_description() {
        let mut intention =
            slot("intention", Shape::Bare(Class::Symbols), set(&["✨"]));
        let separator =
            slot("separator", Shape::Bare(Class::Symbol), set(&[":"]));
        let scope = slot("scope", Shape::Delimited(['(', ')']), Values::Any);
        let keywords =
            slot("keywords", Shape::Bare(Class::Word), set(&["feat"]));

        intention.required = true;
        intention.gap = true;

        assert_eq!(
            context(vec![intention.clone(), scope.clone(), keywords.clone()]),
            Err(Ambiguous::Description {
                slot: "keywords".to_owned()
            })
        );
        assert_eq!(
            Ambiguous::Description {
                slot: "keywords".to_owned()
            }
            .to_string(),
            "`keywords` is optional after a gap, so it can take the start of \
             the description"
        );
        assert!(context(vec![intention.clone(), scope, separator]).is_ok());

        let mut keywords = keywords;

        keywords.required = true;

        assert!(context(vec![intention, keywords]).is_ok());
    }

    #[test]
    fn test_options_that_start_alike_overlap() {
        let code = || slot("code", Shape::Delimited([':', ':']), Values::Any);
        let emoji = |values| slot("emoji", Shape::Bare(Class::Symbols), values);
        let symbol = slot("symbol", Shape::Bare(Class::Symbol), Values::Any);
        let paren = slot("paren", Shape::Delimited([':', ')']), Values::Any);

        assert_eq!(options(vec![code(), paren]), overlap("code", "paren"));
        assert_eq!(
            options(vec![code(), emoji(Values::Any)]),
            overlap("code", "emoji")
        );
        assert_eq!(
            options(vec![emoji(set(&[":)"])), code()]),
            overlap("emoji", "code")
        );
        assert_eq!(
            options(vec![emoji(set(&["✨"])), symbol]),
            overlap("emoji", "symbol")
        );
    }

    #[test]
    fn test_options_that_start_apart_do_not_overlap() {
        let code = || slot("code", Shape::Delimited([':', ':']), Values::Any);
        let word = || slot("word", Shape::Bare(Class::Word), Values::Any);
        let emoji = |values| slot("emoji", Shape::Bare(Class::Symbols), values);

        assert!(options(vec![code(), word()]).is_ok());
        assert!(options(vec![emoji(set(&["✨"])), code()]).is_ok());
        assert!(options(vec![word(), emoji(Values::Any)]).is_ok());
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
