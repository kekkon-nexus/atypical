// The preset files shipped in `presets/` stay loadable, and every header
// behavior is pinned through config alone, so the parser's representation
// can change underneath these rows, spans included.

use std::ops::Range;
use std::path::{Path, PathBuf};

use atypical_commit::config::{self, Any, CommitConfig, SetConfig, SlotConfig};
use chumsky::Parser;

type Row<'r> = (&'r str, Result<(), (Range<usize>, &'r str)>);

const STANDARD_KEYWORDS: &str = "release, undo, add, fix, ref, rem";

/// A shipped preset's `[commit]` section.
fn preset(name: &str) -> CommitConfig {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../presets")
        .join(name);

    atypical_config::load(path, config::SECTION)
        .unwrap()
        .unwrap()
}

/// A preset's slots, for a variant to rearrange: a TOML array cannot be
/// patched in place, so a variant edits the list rather than the file.
fn slots(name: &str) -> Vec<SlotConfig> {
    preset(name).slots.unwrap()
}

fn grammar(slots: Vec<SlotConfig>) -> CommitConfig {
    CommitConfig {
        slots: Some(slots),
        ..CommitConfig::default()
    }
}

fn index(slots: &[SlotConfig], name: &str) -> usize {
    slots.iter().position(|slot| slot.name == name).unwrap()
}

fn anything() -> Option<SetConfig> {
    Some(SetConfig::Any(Any::Any))
}

fn header_parser<'i>(
    tokens: &atypical_commit::Tokens,
) -> impl Parser<'i, &'i str, atypical_commit::Header<'i>, atypical_commit::Extra<'i>>
{
    atypical_commit::header()
        .with_ctx(atypical_commit::ExtraContext::new(tokens).unwrap())
}

fn errors(config: &CommitConfig, header: &str) -> Vec<(Range<usize>, String)> {
    let tokens = atypical_commit::Tokens::try_from(config).unwrap();

    header_parser(&tokens)
        .parse(header)
        .errors()
        .map(|error: &atypical_commit::ExtraError| {
            (error.span().into_range(), error.to_string())
        })
        .collect()
}

fn check(config: &CommitConfig, rows: &[Row]) {
    for (header, expected) in rows {
        let expected = expected
            .clone()
            .err()
            .into_iter()
            .map(|(span, message)| (span, message.to_owned()))
            .collect::<Vec<_>>();

        assert_eq!(errors(config, header), expected, "{header:?}");
    }
}

#[test]
fn standard_preset() {
    check(
        &preset("standard.toml"),
        &[
            ("add: x", Ok(())),
            ("rem?(lib): x", Ok(())),
            ("ref!![eff]: x", Ok(())),
            ("fix!(ci)[sec]: x", Ok(())),
            ("add(exe)[int]: initial", Ok(())),
            ("release: v1.2.3", Ok(())),
            ("undo: x", Ok(())),
            ("add: trailing ", Ok(())),
            (
                "",
                Err((
                    0..0,
                    &*format!(
                        "expected `keywords`, one of: {STANDARD_KEYWORDS}"
                    ),
                )),
            ),
            (
                "add",
                Err((3..3, "found end of input expected '!', '?', or ':'")),
            ),
            (
                "add:",
                Err((4..4, "expected a description after the separator")),
            ),
            (
                "add: ",
                Err((4..5, "expected a description after the separator")),
            ),
            (
                "add:no space",
                Err((4..12, "expected a space before the description")),
            ),
            (
                "feat: x",
                Err((
                    0..4,
                    &*format!(
                        "`feat` is not in `keywords`, expected one of: {STANDARD_KEYWORDS}"
                    ),
                )),
            ),
            (
                "Merge branch 'main'",
                Err((
                    0..5,
                    &*format!(
                        "`Merge` is not in `keywords`, expected one of: {STANDARD_KEYWORDS}"
                    ),
                )),
            ),
            (
                "añadir: x",
                Err((
                    0..7,
                    &*format!(
                        "`añadir` is not in `keywords`, expected one of: {STANDARD_KEYWORDS}"
                    ),
                )),
            ),
            ("add??: x", Err((4..5, "found '?' expected ':'"))),
            ("add(exe)!: x", Err((8..9, "found '!' expected ':'"))),
            (
                "add(unsupported): x",
                Err((
                    4..15,
                    "`unsupported` is not in `scope`, expected one of: exe, lib, test, build, doc, ci, cd",
                )),
            ),
            (
                "add(): x",
                Err((
                    4..4,
                    "expected `scope`, one of: exe, lib, test, build, doc, ci, cd",
                )),
            ),
            (
                "add{lib}: x",
                Err((3..4, "found '{' expected '!', '?', or ':'")),
            ),
            ("add[pre](lib): x", Err((8..9, "found '(' expected ':'"))),
            ("add(exe)(lib): x", Err((8..9, "found '(' expected ':'"))),
            (
                "add(: x",
                Err((
                    4..4,
                    "expected `scope`, one of: exe, lib, test, build, doc, ci, cd",
                )),
            ),
            ("add(lib: x", Err((7..8, "found ':' expected ')'"))),
            ("add; x", Err((3..4, "found ';' expected '!', '?', or ':'"))),
            (
                "add : x",
                Err((3..4, "found ' ' expected '!', '?', or ':'")),
            ),
        ],
    );
}

#[test]
fn conventional_preset() {
    check(
        &preset("conventional.toml"),
        &[
            ("feat: an endpoint", Ok(())),
            ("fix(parser): handle empty input", Ok(())),
            ("feat(api)!: drop the v1 routes", Ok(())),
            ("revert: feat: an endpoint", Ok(())),
            ("feat(): empty scope", Err((5..6, "expected `scope`"))),
            ("feat(a b): spaced scope", Ok(())),
            (
                "add(lib): standard style",
                Err((
                    0..3,
                    "`add` is not in `keywords`, expected one of: refactor, revert, build, chore, style, docs, feat, perf, test, fix, ci",
                )),
            ),
            (
                "feat[api]: wrong enclosure",
                Err((4..5, "found '[' expected '!', or ':'")),
            ),
            (
                "feat!(api): modifier before the scope",
                Err((5..6, "found '(' expected ':'")),
            ),
            ("feat!!: doubled", Err((5..6, "found '!' expected ':'"))),
            (
                "feat(api)?: unknown modifier",
                Err((9..10, "found '?' expected '!', or ':'")),
            ),
        ],
    );
}

#[test]
fn unrestricted() {
    check(
        &CommitConfig::default(),
        &[
            ("add(lib)[int]: standard style", Ok(())),
            ("feat(api)!: conventional style", Ok(())),
            ("yolo(whatever)> ship it", Ok(())),
            ("wip!!~ kitchen sink", Ok(())),
            ("añadir: x", Ok(())),
            // The modifier slot sits after the enclosures, so one
            // before them is read as the separator.
            (
                "feat!(api): x",
                Err((6..13, "expected a space before the description")),
            ),
            ("add! x", Ok(())),
            ("add!!; x", Ok(())),
            (
                "add(lib)(lib): x",
                Err((9..16, "expected a space before the description")),
            ),
            (
                "add[int](lib): x",
                Err((9..16, "expected a space before the description")),
            ),
            ("no separator here", Err((2..2, "expected `modifiers`"))),
            (": no keyword", Err((0..0, "expected `keywords`"))),
            (
                "add:",
                Err((4..4, "expected a description after the separator")),
            ),
            (
                "add:no space",
                Err((4..12, "expected a space before the description")),
            ),
            (
                "add{x}: y",
                Err((4..9, "expected a space before the description")),
            ),
        ],
    );
}

#[test]
fn any_keyword() {
    let mut slots = slots("standard.toml");
    let keywords = index(&slots, "keywords");

    slots[keywords].values = anything();

    check(
        &grammar(slots),
        &[
            ("feat: x", Ok(())),
            ("añadir: x", Ok(())),
            ("snake_case: x", Ok(())),
            (": x", Err((0..0, "expected `keywords`"))),
        ],
    );
}

#[test]
fn any_modifier() {
    let mut slots = slots("standard.toml");
    let modifiers = index(&slots, "modifiers");

    slots[modifiers].values = anything();

    check(
        &grammar(slots),
        &[
            ("add??: x", Ok(())),
            ("add~+!: x", Ok(())),
            ("add!(lib): x", Ok(())),
            ("add: x", Ok(())),
            ("add!!!: x", Ok(())),
            // The modifier stops at an opener instead of eating it.
            (
                "add!(: x",
                Err((
                    5..5,
                    "expected `scope`, one of: exe, lib, test, build, doc, ci, cd",
                )),
            ),
        ],
    );
}

#[test]
fn modifier_on_either_side_is_rejected() {
    let mut slots = slots("standard.toml");
    let modifiers = index(&slots, "modifiers");
    let mut post = slots[modifiers].clone();

    slots[modifiers].name = "modifiers (pre)".to_owned();
    post.name = "modifiers (post)".to_owned();

    let separator = index(&slots, "separator");

    slots.insert(separator, post);

    let tokens = atypical_commit::Tokens::try_from(&grammar(slots)).unwrap();

    assert_eq!(
        atypical_commit::ExtraContext::new(&tokens),
        // The same spellings in both slots, so they share a prefix.
        Err(atypical_commit::Ambiguous::Prefix {
            first: "modifiers (pre)".to_owned(),
            second: "modifiers (post)".to_owned(),
            spelling: "?".to_owned(),
        })
    );
}

#[test]
fn modifier_after_the_enclosures() {
    let mut slots = slots("standard.toml");
    let modifiers = slots.remove(index(&slots, "modifiers"));
    let separator = index(&slots, "separator");

    slots.insert(separator, modifiers);

    check(
        &grammar(slots),
        &[
            ("add(lib)!: x", Ok(())),
            (
                "add!(lib): x",
                Err((4..5, "found '(' expected '!', or ':'")),
            ),
        ],
    );
}

#[test]
fn flexible_enclosure() {
    let mut slots = slots("standard.toml");
    let reason = index(&slots, "reason");

    slots.remove(reason);

    let scope = index(&slots, "scope");

    slots[scope].values = anything();

    check(
        &grammar(slots),
        &[
            ("add(anything goes): x", Ok(())),
            ("add(): x", Err((4..5, "expected `scope`"))),
            (
                "add(unclosed: x",
                Err((
                    15..15,
                    "found end of input expected something else, or ')'",
                )),
            ),
            (
                "add(nested()): x",
                Err((10..11, "found '(' expected something else, or ')'")),
            ),
            (
                "add[pre]: x",
                Err((3..4, "found '[' expected '!', '?', or ':'")),
            ),
        ],
    );
}

#[test]
fn strict_and_flexible_enclosures() {
    let mut slots = slots("standard.toml");
    let scope = index(&slots, "scope");
    let reason = index(&slots, "reason");

    slots[scope].values = Some(SetConfig::OneOf(vec!["core".to_owned()]));
    slots[reason].delimiters = Some(['{', '}']);
    slots[reason].values = anything();

    check(
        &grammar(slots),
        &[
            ("add(core){any thing}: x", Ok(())),
            ("add{free}: x", Ok(())),
            (
                "add(other): x",
                Err((4..9, "`other` is not in `scope`, expected one of: core")),
            ),
            ("add{x}(core): x", Err((6..7, "found '(' expected ':'"))),
        ],
    );
}

#[test]
fn another_separator() {
    let mut slots = slots("standard.toml");
    let separator = index(&slots, "separator");

    slots[separator].values = Some(SetConfig::OneOf(vec![";".to_owned()]));

    check(
        &grammar(slots),
        &[
            ("add; x", Ok(())),
            ("add: x", Err((3..4, "found ':' expected '!', '?', or ';'"))),
        ],
    );
}

#[test]
fn a_set_before_an_unrestricted_separator_is_rejected() {
    let mut slots = slots("standard.toml");
    let separator = index(&slots, "separator");

    slots[separator].values = anything();

    let tokens = atypical_commit::Tokens::try_from(&grammar(slots)).unwrap();

    assert_eq!(
        atypical_commit::ExtraContext::new(&tokens),
        // Every modifier spelling would serve as the separator too.
        Err(atypical_commit::Ambiguous::Shadowed {
            first: "modifiers".to_owned(),
            second: "separator".to_owned(),
        })
    );
}

#[test]
fn any_separator() {
    let mut slots = slots("standard.toml");

    slots.retain(|slot| slot.name != "modifiers");

    let separator = index(&slots, "separator");

    slots[separator].values = anything();

    check(
        &grammar(slots),
        &[
            ("add: x", Ok(())),
            ("add; x", Ok(())),
            ("add> x", Ok(())),
            ("add x", Err((3..3, "expected `separator`"))),
            ("add : x", Err((3..3, "expected `separator`"))),
        ],
    );
}

#[test]
fn any_modifier_leaves_the_any_separator() {
    let mut slots = slots("standard.toml");
    let modifiers = index(&slots, "modifiers");
    let separator = index(&slots, "separator");

    slots[modifiers].values = anything();
    slots[separator].values = anything();

    check(
        &grammar(slots),
        &[
            ("add!!; x", Ok(())),
            ("add; x", Ok(())),
            ("add!(lib): x", Ok(())),
            // A lone symbol is the separator, not a modifier.
            ("add! x", Ok(())),
        ],
    );
}

#[test]
fn every_enclosure_in_a_row_is_reachable() {
    let mut slots = slots("standard.toml");
    let separator = index(&slots, "separator");

    slots.insert(
        separator,
        SlotConfig {
            name: "extra".to_owned(),
            kind: None,
            delimiters: Some(['{', '}']),
            one_of: None,
            values: anything(),
            required: false,
            gap: false,
        },
    );

    check(
        &grammar(slots),
        &[
            ("add(lib)[int]{third}: x", Ok(())),
            ("add(lib){third}: x", Ok(())),
            ("add[int]{third}: x", Ok(())),
            ("add{third}: x", Ok(())),
        ],
    );
}

#[test]
fn a_required_enclosure_is_demanded() {
    let mut slots = slots("standard.toml");
    let scope = index(&slots, "scope");

    slots[scope].required = true;

    check(
        &grammar(slots),
        &[
            ("add(lib): x", Ok(())),
            ("add: x", Err((3..4, "expected an opening `(`"))),
            ("add[int]: x", Err((3..4, "expected an opening `(`"))),
        ],
    );
}

#[test]
fn a_required_enclosure_is_demanded_with_its_gap() {
    let mut slots = slots("standard.toml");
    let scope = index(&slots, "scope");

    let keywords = index(&slots, "keywords");

    slots[scope].required = true;
    slots[keywords].gap = true;

    let config = grammar(slots);
    let missing = errors(&config, "add: x");

    assert!(errors(&config, "add (lib): x").is_empty());
    assert!(
        missing
            .iter()
            .any(|(_, message)| message == "expected an opening ` (`"),
        "{missing:?}"
    );
}

#[test]
fn enclosures_may_share_an_opener() {
    let mut slots = slots("standard.toml");
    let reason = index(&slots, "reason");

    slots[reason].delimiters = Some(['(', ']']);
    slots[reason].values = anything();

    check(
        &grammar(slots),
        &[
            ("add(lib): x", Ok(())),
            ("add(free]: x", Ok(())),
            ("add(lib)(free]: x", Ok(())),
        ],
    );
}

#[test]
fn enclosures_may_not_share_delimiters() {
    let mut slots = slots("standard.toml");
    let scope = index(&slots, "scope");
    let reason = index(&slots, "reason");

    slots[reason].delimiters = slots[scope].delimiters;

    let tokens = atypical_commit::Tokens::try_from(&grammar(slots)).unwrap();

    assert_eq!(
        atypical_commit::ExtraContext::new(&tokens),
        Err(atypical_commit::Ambiguous::Delimiters {
            first: "scope".to_owned(),
            second: "reason".to_owned(),
        })
    );
}

#[test]
fn a_run_without_a_separator_slot_is_refused() {
    let mut slots = slots("standard.toml");

    slots.retain(|slot| slot.name != "separator");

    let modifiers = index(&slots, "modifiers");

    slots[modifiers].values = anything();

    check(
        &grammar(slots),
        &[(
            "add!: x",
            Err((3..3, "`modifiers` needs a separator after it")),
        )],
    );
}

const INTENTION: &str = indoc::indoc! {r#"
    [[commit.slots]]
    name = "intention"
    required = true
    gap = true

    [[commit.slots.one-of]]
    name = "emoji"
    kind = "symbols"
    values = ["✨", "🐛"]

    [[commit.slots.one-of]]
    name = "shortcode"
    delimiters = [":", ":"]
    values = ["sparkles", "bug"]

    [[commit.slots]]
    name = "scope"
    delimiters = ["(", ")"]

    [[commit.slots]]
    name = "separator"
    kind = "symbol"
    values = [":"]
"#};

fn load(name: &str, contents: &str) -> CommitConfig {
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);

    std::fs::write(&file, contents).unwrap();

    atypical_config::load(&file, config::SECTION)
        .unwrap()
        .unwrap()
}

#[test]
fn one_of_takes_exactly_one_form() {
    let config = load("one-of.toml", INTENTION);

    for header in ["✨ Add", ":sparkles: Add", "🐛 (auth): Fix", ":bug: (x) y"]
    {
        assert!(errors(&config, header).is_empty(), "{header:?}");
    }

    for header in [" Add", "Add", "✨:bug: both", ":bogus: Add"] {
        assert!(!errors(&config, header).is_empty(), "{header:?}");
    }
}

#[test]
fn a_gap_before_a_bare_slot_is_owed() {
    let config = load(
        "gap-bare.toml",
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "intention"
            kind = "symbols"
            values = ["✨"]
            required = true
            gap = true

            [[commit.slots]]
            name = "keywords"
            kind = "word"
            values = ["feat"]
            required = true

            [[commit.slots]]
            name = "separator"
            kind = "symbol"
            values = [":"]
            required = true
        "#},
    );

    assert!(errors(&config, "✨ feat: x").is_empty());
    assert!(!errors(&config, "✨feat: x").is_empty());
}

#[test]
fn an_optional_option_commits_on_its_opener() {
    let config = load(
        "one-of-optional.toml",
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "intention"
            kind = "symbols"
            values = ["✨"]
            required = true
            gap = true

            [[commit.slots]]
            name = "ticket"

            [[commit.slots.one-of]]
            name = "issue"
            delimiters = ["[", "]"]
            values = ["ABC"]

            [[commit.slots.one-of]]
            name = "epic"
            delimiters = ["<", ">"]
            values = ["ABC"]
        "#},
    );

    for header in ["✨ [ABC] Add", "✨ <ABC> Add", "✨ Add"] {
        assert!(errors(&config, header).is_empty(), "{header:?}");
    }

    for header in ["✨ [BOGUS] Add", "✨ <BOGUS> Add"] {
        assert!(!errors(&config, header).is_empty(), "{header:?}");
    }
}

#[test]
fn an_opener_shared_with_a_later_slot_still_backtracks() {
    let config = load(
        "one-of-shared-opener.toml",
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "intention"
            kind = "symbols"
            values = ["✨"]
            required = true
            gap = true

            [[commit.slots]]
            name = "ticket"

            [[commit.slots.one-of]]
            name = "issue"
            delimiters = ["[", "]"]
            values = ["ABC"]

            [[commit.slots]]
            name = "reason"
            delimiters = ["[", ")"]
        "#},
    );

    for header in ["✨ [ABC] Add", "✨ [x) Add", "✨ Add"] {
        assert!(errors(&config, header).is_empty(), "{header:?}");
    }

    // A `[` no slot accepts falls open to the description, the same as
    // when the enclosure comes first, so the run's order does not
    // decide it.
    assert!(errors(&config, "✨ [BOGUS] Add").is_empty());
    assert!(parts(&config, "✨ [BOGUS] Add") == ["intention"]);
}

fn parts(config: &CommitConfig, header: &str) -> Vec<String> {
    let tokens = atypical_commit::Tokens::try_from(config).unwrap();

    header_parser(&tokens)
        .parse(header)
        .into_output()
        .map(|header| header.prefix.iter().map(|p| p.name.clone()).collect())
        .unwrap_or_default()
}

fn enclosure_shared(name: &str, required: bool) -> CommitConfig {
    load(
        name,
        &format!(
            indoc::indoc! {r#"
                [[commit.slots]]
                name = "intention"
                kind = "symbols"
                values = ["✨"]
                required = true
                gap = true

                [[commit.slots]]
                name = "reason"
                delimiters = ["[", ")"]
                required = {required}

                [[commit.slots]]
                name = "ticket"

                [[commit.slots.one-of]]
                name = "issue"
                delimiters = ["[", "]"]
                values = ["ABC"]
            "#},
            required = required
        ),
    )
}

#[test]
fn an_enclosure_opener_shared_with_a_later_slot_still_backtracks() {
    let config = enclosure_shared("enclosure-shared-opener.toml", false);

    for header in ["✨ [ABC] Add", "✨ [x) Add", "✨ Add"] {
        assert!(errors(&config, header).is_empty(), "{header:?}");
    }

    // A shared opener is tried, not committed on: `reason` claims `[x)`
    // rather than leaving it to the description.
    assert!(parts(&config, "✨ [x) Add").contains(&"reason".to_owned()));

    // A `[` no slot accepts falls open, the same as with the enclosure
    // last, rather than `reason` committing because nothing follows it.
    assert!(errors(&config, "✨ [BOGUS] Add").is_empty());
    assert!(parts(&config, "✨ [BOGUS] Add") == ["intention"]);

    // Required, the same header parses instead of failing for a `[` the
    // run declined to open.
    let required = enclosure_shared("enclosure-shared-required.toml", true);

    assert!(errors(&required, "✨ [x) Add").is_empty());
    assert!(!errors(&required, "✨ Add").is_empty());
}

#[test]
fn a_project_drops_one_form() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));

    load("one-of-base.toml", INTENTION);

    let config = load(
        "one-of-drop.toml",
        &format!(
            indoc::indoc! {r#"
                extends = '{}'

                [[commit.slots]]
                name = "intention"

                [[commit.slots.one-of]]
                name = "shortcode"
                drop = true
            "#},
            dir.join("one-of-base.toml").display()
        ),
    );

    assert!(errors(&config, "✨ Add").is_empty());
    assert!(!errors(&config, ":sparkles: Add").is_empty());
}

#[test]
fn options_that_start_alike_are_rejected() {
    let mut slots = slots("standard.toml");
    let keywords = index(&slots, "keywords");
    let option = |name: &str, values: &[&str]| SlotConfig {
        name: name.to_owned(),
        kind: Some(config::KindConfig::Word),
        delimiters: None,
        one_of: None,
        values: Some(SetConfig::OneOf(
            values.iter().map(|&value| value.to_owned()).collect(),
        )),
        required: false,
        gap: false,
    };

    slots[keywords].kind = None;
    slots[keywords].values = None;
    slots[keywords].one_of =
        Some(vec![option("short", &["fix"]), option("long", &["fixup"])]);

    let tokens = atypical_commit::Tokens::try_from(&grammar(slots)).unwrap();

    assert_eq!(
        atypical_commit::ExtraContext::new(&tokens),
        Err(atypical_commit::Ambiguous::Overlap {
            first: "short".to_owned(),
            second: "long".to_owned(),
        })
    );
}

#[test]
fn a_description_may_not_start_with_whitespace() {
    // The separator's one space is stripped; a second would otherwise
    // fall into the description and pass, the gap fail-open one space
    // wider.
    let conv = preset("conventional.toml");

    assert!(!errors(&conv, "feat:  x").is_empty());
    assert!(errors(&conv, "feat: x").is_empty());

    let gap = load(
        "whitespace-gap.toml",
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "intention"
            kind = "symbols"
            values = ["✨"]
            required = true
            gap = true

            [[commit.slots]]
            name = "scope"
            delimiters = ["(", ")"]
            values = ["auth"]

            [[commit.slots]]
            name = "separator"
            kind = "symbol"
            values = [":"]
        "#},
    );

    assert!(!errors(&gap, "✨  (bogus): Add").is_empty());
    assert!(!errors(&gap, "✨  Add").is_empty());
    assert!(errors(&gap, "✨ (auth): Add").is_empty());
}

#[test]
fn an_unrestricted_enclosure_may_not_be_empty() {
    let conv = preset("conventional.toml");

    assert!(!errors(&conv, "feat(): x").is_empty());
    assert!(errors(&conv, "feat(scope): x").is_empty());
}

#[test]
fn a_symbol_slot_before_the_keyword_is_not_the_separator() {
    // `modifiers` must stop on the real separator, not on the first
    // `symbol` slot in the grammar.
    let config = load(
        "marker.toml",
        indoc::indoc! {r##"
            [[commit.slots]]
            name = "marker"
            kind = "symbol"
            values = ["#"]

            [[commit.slots]]
            name = "keywords"
            kind = "word"
            required = true

            [[commit.slots]]
            name = "modifiers"
            kind = "symbols"

            [[commit.slots]]
            name = "separator"
            kind = "symbol"
            values = [":"]
            required = true
        "##},
    );

    assert!(
        errors(&config, "#feat!: x").is_empty(),
        "{:?}",
        errors(&config, "#feat!: x")
    );
    assert!(
        errors(&config, "#feat: x").is_empty(),
        "{:?}",
        errors(&config, "#feat: x")
    );
}

#[test]
fn gitmoji_preset() {
    let config = preset("gitmoji.toml");

    for header in [
        "✨ Add a feature",
        ":sparkles: Add a feature",
        "🐛 (auth): Fix a bug",
        ":bug: (auth): Fix a bug",
        ":t-rex: Add old code", // hyphen in a delimited shortcode
        "⚡️ Improve performance",
    ] {
        assert!(errors(&config, header).is_empty(), "{header:?}");
    }

    for header in [
        "feat: no intention", // neither form
        "✨:bug: both forms", // both forms
        ":bogus: unknown shortcode",
        "✨ (): empty scope",
    ] {
        assert!(!errors(&config, header).is_empty(), "{header:?}");
    }
}

#[test]
fn gitmoji_every_emoji_and_shortcode_lints() {
    let config = preset("gitmoji.toml");
    let slots = config.slots.as_ref().unwrap();
    let intention = &slots[index(slots, "intention")];
    let options = intention.one_of.as_ref().unwrap();
    let values =
        |name: &str| match options[index(options, name)].values.as_ref() {
            Some(SetConfig::OneOf(values)) => values.clone(),
            other => panic!("{name}: {other:?}"),
        };

    // The preset is the source of truth: every spelling it ships must
    // lint, not just the handful the examples name.
    for emoji in values("emoji") {
        let header = format!("{emoji} a change");

        assert!(errors(&config, &header).is_empty(), "{header:?}");
    }

    for code in values("shortcode") {
        let header = format!(":{code}: a change");

        assert!(errors(&config, &header).is_empty(), "{header:?}");
    }
}

#[test]
fn gitmoji_layers_onto_conventional() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let presets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../presets");
    let file = dir.join("gitmoji-conventional.toml");

    std::fs::write(
        &file,
        format!(
            "extends = ['{}', '{}']\n",
            presets.join("conventional.toml").display(),
            presets.join("gitmoji.toml").display(),
        ),
    )
    .unwrap();

    let config: CommitConfig = atypical_config::load(&file, config::SECTION)
        .unwrap()
        .unwrap();

    assert!(errors(&config, "✨ feat(api): add an endpoint").is_empty());
    assert!(errors(&config, "🐛 fix(api)!: drop the v1 routes").is_empty());
    // The conventional keyword is still enforced, and the emoji is now
    // required in front of it.
    assert!(!errors(&config, "✨ nope: unknown keyword").is_empty());
    assert!(!errors(&config, "feat(api): no emoji").is_empty());
}

#[test]
fn presets_are_reachable_through_extends() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let preset = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../presets/conventional.toml");
    let file = dir.join("extends-preset.toml");

    std::fs::write(&file, format!("extends = '{}'\n", preset.display()))
        .unwrap();

    let config: CommitConfig = atypical_config::load(&file, config::SECTION)
        .unwrap()
        .unwrap();

    assert!(errors(&config, "feat: through the preset").is_empty());
    assert!(!errors(&config, "add: standard style").is_empty());
}

#[test]
fn a_preset_slot_is_narrowed_without_restating_the_rest() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let preset = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../presets/conventional.toml");
    let file = dir.join("extends-narrowed.toml");

    std::fs::write(
        &file,
        format!(
            indoc::indoc! {r#"
                extends = '{}'

                [[commit.slots]]
                name = "keywords"
                values = ["feat", "fix", "docs"]
            "#},
            preset.display()
        ),
    )
    .unwrap();

    let config: CommitConfig = atypical_config::load(&file, config::SECTION)
        .unwrap()
        .unwrap();

    assert!(errors(&config, "docs: narrowed").is_empty());
    // The slots the override never named are still the preset's.
    assert!(errors(&config, "feat(api)!: untouched").is_empty());
    assert_eq!(
        errors(&config, "chore: dropped by the override"),
        [(
            0..5,
            "`chore` is not in `keywords`, expected one of: docs, feat, fix"
                .to_owned()
        )]
    );
}
