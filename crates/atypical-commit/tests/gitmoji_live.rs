// Drift check against the live gitmoji catalogue, run through the real
// parser rather than a text diff of the generated file. Ignored by
// default so the offline suite does not reach the network; CI runs it
// with `--run-ignored ignored-only`.

use std::path::Path;

use atypical_commit::config::{self, CommitConfig, SetConfig};
use chumsky::Parser;
use serde::Deserialize;

const SOURCE: &str = "https://gitmoji.dev/api/gitmojis";

#[derive(Deserialize)]
struct Catalogue {
    gitmojis: Vec<Gitmoji>,
}

#[derive(Deserialize)]
struct Gitmoji {
    emoji: String,
    code: String,
}

fn preset() -> CommitConfig {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../presets/gitmoji.toml");

    atypical_config::load(path, config::SECTION)
        .unwrap()
        .unwrap()
}

/// The `values` of a named option of the `intention` slot.
fn option(config: &CommitConfig, name: &str) -> Vec<String> {
    let slots = config.slots.as_ref().unwrap();
    let intention = slots.iter().find(|s| s.name == "intention").unwrap();
    let options = intention.one_of.as_ref().unwrap();

    match options
        .iter()
        .find(|s| s.name == name)
        .unwrap()
        .values
        .as_ref()
    {
        Some(SetConfig::OneOf(values)) => values.clone(),
        other => panic!("{name}: {other:?}"),
    }
}

/// The live catalogue, fetched and deserialized in process.
fn live() -> Vec<Gitmoji> {
    let body = ureq::get(SOURCE)
        .call()
        .unwrap()
        .body_mut()
        .read_to_string()
        .unwrap();

    serde_json::from_str::<Catalogue>(&body).unwrap().gitmojis
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn lints(config: &CommitConfig, header: &str) -> bool {
    let tokens = atypical_commit::Tokens::try_from(config).unwrap();
    let parser = atypical_commit::header()
        .with_ctx(atypical_commit::ExtraContext::new(&tokens).unwrap());

    Parser::<'_, _, atypical_commit::Header, atypical_commit::Extra>::parse(
        &parser, header,
    )
    .into_result()
    .is_ok()
}

#[test]
#[ignore = "network: fetches the live gitmoji catalogue"]
fn gitmoji_preset_matches_the_live_catalogue() {
    let config = preset();
    let catalogue = live();

    let emoji = catalogue
        .iter()
        .map(|g| g.emoji.clone())
        .collect::<Vec<_>>();
    // A delimited slot only holds a word, so the generator drops a
    // non-word shortcode; the check mirrors that to compare like for
    // like.
    let shortcodes = catalogue
        .iter()
        .map(|g| g.code.trim_matches(':').to_owned())
        .filter(|code| code.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .collect::<Vec<_>>();

    assert_eq!(
        sorted(option(&config, "emoji")),
        sorted(emoji.clone()),
        "emoji drift; run `bun scripts/gitmoji.ts`"
    );
    assert_eq!(
        sorted(option(&config, "shortcode")),
        sorted(shortcodes.clone()),
        "shortcode drift; run `bun scripts/gitmoji.ts`"
    );

    // The sets match, so the parser accepts every live spelling.
    for emoji in &emoji {
        assert!(lints(&config, &format!("{emoji} a change")), "{emoji}");
    }

    for code in &shortcodes {
        assert!(lints(&config, &format!(":{code}: a change")), "{code}");
    }
}
