// The preset files shipped in `presets/` stay loadable and lint the
// style they claim to.

use std::path::{Path, PathBuf};

use atypical_commit::Tokens;
use atypical_commit::config::{self, CommitConfig};
use chumsky::Parser;

fn preset(name: &str) -> CommitConfig {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../presets")
        .join(name);

    atypical_config::load(path, config::SECTION)
        .unwrap()
        .unwrap()
}

fn header_parser<'i>(
    tokens: &'i Tokens<'i>,
) -> impl Parser<'i, &'i str, atypical_commit::Header<'i>, atypical_commit::Extra<'i>>
{
    atypical_commit::header()
        .with_ctx(atypical_commit::ExtraContext::new(tokens))
}

fn parses(config: &CommitConfig, header: &str) -> bool {
    let tokens = Tokens::from(config);

    !header_parser(&tokens).parse(header).has_errors()
}

#[test]
fn standard_preset_mirrors_preset_standard() {
    assert_eq!(
        preset("standard.toml"),
        CommitConfig::from(&Tokens::preset_standard())
    );
}

#[test]
fn conventional_preset_lints_conventional_headers() {
    let config = preset("conventional.toml");

    for header in [
        "feat: an endpoint",
        "fix(parser): handle empty input",
        "feat(api)!: drop the v1 routes",
        "revert: feat: an endpoint",
    ] {
        assert!(parses(&config, header), "rejected: {header}");
    }

    for header in [
        "add(lib): standard style",
        "feat[api]: wrong enclosure",
        "feat!(api): modifier before the scope",
    ] {
        assert!(!parses(&config, header), "accepted: {header}");
    }
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

    assert!(parses(&config, "feat: through the preset"));
    assert!(!parses(&config, "add: standard style"));
}
