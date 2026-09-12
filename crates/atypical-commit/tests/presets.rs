// The preset files shipped in `presets/` stay loadable, and every header
// behavior is pinned through config alone, so the parser's representation
// can change underneath these rows, spans included.

use std::ops::Range;
use std::path::{Path, PathBuf};

use atypical_commit::config::{self, CommitConfig};
use chumsky::Parser;

type Row<'r> = (&'r str, Result<(), (Range<usize>, &'r str)>);

const STANDARD_KEYWORDS: &str = "release, undo, add, fix, ref, rem";

/// A shipped preset's `[commit]` section, with each top-level key in
/// `overrides` replacing the preset's outright, never merged into it.
fn preset(name: &str, overrides: &str) -> CommitConfig {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../presets")
        .join(name);

    let mut section: toml::Table = atypical_config::load(path, config::SECTION)
        .unwrap()
        .unwrap();

    section.extend(toml::from_str::<toml::Table>(overrides).unwrap());

    toml::Value::Table(section).try_into().unwrap()
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
        &preset("standard.toml", ""),
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
                    &*format!("expected keyword, one of: {STANDARD_KEYWORDS}"),
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
                        "unknown keyword `feat`, expected one of: {STANDARD_KEYWORDS}"
                    ),
                )),
            ),
            (
                "Merge branch 'main'",
                Err((
                    0..5,
                    &*format!(
                        "unknown keyword `Merge`, expected one of: {STANDARD_KEYWORDS}"
                    ),
                )),
            ),
            (
                "añadir: x",
                Err((
                    0..7,
                    &*format!(
                        "unknown keyword `añadir`, expected one of: {STANDARD_KEYWORDS}"
                    ),
                )),
            ),
            ("add??: x", Err((4..5, "found '?' expected ':'"))),
            ("add(exe)!: x", Err((8..9, "found '!' expected ':'"))),
            (
                "add(unsupported): x",
                Err((
                    4..15,
                    "unknown enclosure `unsupported`, expected one of: exe, lib, test, build, doc, ci, cd",
                )),
            ),
            (
                "add(): x",
                Err((
                    4..4,
                    "expected enclosure, one of: exe, lib, test, build, doc, ci, cd",
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
                    "expected enclosure, one of: exe, lib, test, build, doc, ci, cd",
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
        &preset("conventional.toml", ""),
        &[
            ("feat: an endpoint", Ok(())),
            ("fix(parser): handle empty input", Ok(())),
            ("feat(api)!: drop the v1 routes", Ok(())),
            ("revert: feat: an endpoint", Ok(())),
            ("feat(): empty scope", Ok(())),
            ("feat(a b): spaced scope", Ok(())),
            (
                "add(lib): standard style",
                Err((
                    0..3,
                    "unknown keyword `add`, expected one of: refactor, revert, build, chore, style, docs, feat, perf, test, fix, ci",
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
            ("no separator here", Err((2..2, "expected a modifier"))),
            (": no keyword", Err((0..0, "expected a keyword"))),
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
    check(
        &preset("standard.toml", r#"keywords = "any""#),
        &[
            ("feat: x", Ok(())),
            ("añadir: x", Ok(())),
            ("snake_case: x", Ok(())),
            (": x", Err((0..0, "expected a keyword"))),
        ],
    );
}

#[test]
fn any_modifier() {
    check(
        &preset("standard.toml", r#"modifiers = "any""#),
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
                    "expected enclosure, one of: exe, lib, test, build, doc, ci, cd",
                )),
            ),
        ],
    );
}

#[test]
fn modifier_on_either_side_is_rejected() {
    let config = preset("standard.toml", r#"modifier-sequence = "any""#);
    let tokens = atypical_commit::Tokens::try_from(&config).unwrap();

    assert_eq!(
        atypical_commit::ExtraContext::new(&tokens),
        // One key fills both slots, so they hold the same spellings.
        Err(atypical_commit::Ambiguous::Prefix {
            first: "modifier-sequence (pre)".to_owned(),
            second: "modifier-sequence (post)".to_owned(),
            spelling: "?".to_owned(),
        })
    );
}

#[test]
fn modifier_after_the_enclosures() {
    check(
        &preset("standard.toml", r#"modifier-sequence = "post""#),
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
    let overrides = r#"enclosures = [{ delimiters = ["(", ")"] }]"#;

    check(
        &preset("standard.toml", overrides),
        &[
            ("add(anything goes): x", Ok(())),
            ("add(): x", Ok(())),
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
    let overrides = r#"enclosures = [
        { delimiters = ["(", ")"], allowed = ["core"] },
        { delimiters = ["{", "}"] },
    ]"#;

    check(
        &preset("standard.toml", overrides),
        &[
            ("add(core){any thing}: x", Ok(())),
            ("add{free}: x", Ok(())),
            (
                "add(other): x",
                Err((4..9, "unknown enclosure `other`, expected one of: core")),
            ),
            ("add{x}(core): x", Err((6..7, "found '(' expected ':'"))),
        ],
    );
}

#[test]
fn another_separator() {
    check(
        &preset("standard.toml", r#"separator = ";""#),
        &[
            ("add; x", Ok(())),
            ("add: x", Err((3..4, "found ':' expected '!', '?', or ';'"))),
        ],
    );
}

#[test]
fn any_separator() {
    check(
        &preset("standard.toml", r#"separator = "any""#),
        &[
            ("add: x", Ok(())),
            ("add; x", Ok(())),
            ("add> x", Ok(())),
            ("add x", Err((3..4, "expected a separator"))),
            ("add : x", Err((3..4, "expected a separator"))),
        ],
    );
}

#[test]
fn any_modifier_leaves_the_any_separator() {
    let overrides = "modifiers = \"any\"\nseparator = \"any\"";

    check(
        &preset("standard.toml", overrides),
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
