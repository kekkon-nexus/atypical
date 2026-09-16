use std::assert_matches;
use std::path::PathBuf;

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Section {
    name: String,
}

fn tree(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);

    std::fs::create_dir_all(root.join("nested/deeper")).unwrap();

    root
}

#[test]
fn find_walks_up_to_the_nearest_file() {
    let root = tree("find-nearest");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(&file, "").unwrap();

    assert_eq!(
        atypical_config::find(root.join("nested/deeper")),
        Some(file)
    );
}

#[test]
fn find_without_a_file_is_none() {
    // Outside the repository: the ancestor walk would otherwise find
    // this repo's own atypical.toml above CARGO_TARGET_TMPDIR.
    let root = std::env::temp_dir().join("atypical-find-none");

    std::fs::create_dir_all(root.join("nested/deeper")).unwrap();

    assert_eq!(atypical_config::find(root.join("nested/deeper")), None);
}

#[test]
fn load_reads_the_section() {
    let root = tree("load-section");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(&file, "[commit]\nname = \"value\"\n").unwrap();

    assert_eq!(
        atypical_config::load::<Section>(&file, "commit").unwrap(),
        Some(Section {
            name: "value".into()
        })
    );
    assert_eq!(
        atypical_config::load::<Section>(&file, "branch").unwrap(),
        None
    );
}

#[test]
fn load_applies_extends_beneath_the_document() {
    let root = tree("extends-beneath");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(
        root.join("nested/preset.toml"),
        "[commit]\nname = \"preset\"\n",
    )
    .unwrap();
    std::fs::write(&file, "extends = \"nested/preset.toml\"\n").unwrap();

    assert_eq!(
        atypical_config::load::<Section>(&file, "commit").unwrap(),
        Some(Section {
            name: "preset".into()
        })
    );

    std::fs::write(
        &file,
        "extends = \"nested/preset.toml\"\n[commit]\nname = \"local\"\n",
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Section>(&file, "commit").unwrap(),
        Some(Section {
            name: "local".into()
        })
    );
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Merged {
    name: String,
    other: String,
}

#[test]
fn extends_chain_merges_tables_one_by_one() {
    let root = tree("extends-chain");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(
        root.join("a.toml"),
        "[commit]\nname = \"a\"\nother = \"a\"\n",
    )
    .unwrap();
    std::fs::write(root.join("b.toml"), "[commit]\nname = \"b\"\n").unwrap();
    std::fs::write(&file, "extends = [\"a.toml\", \"b.toml\"]\n").unwrap();

    assert_eq!(
        atypical_config::load::<Merged>(&file, "commit").unwrap(),
        Some(Merged {
            name: "b".into(),
            other: "a".into()
        })
    );
}

#[test]
fn extends_may_share_a_common_base() {
    let root = tree("extends-diamond");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(root.join("base.toml"), "[commit]\nname = \"base\"\n")
        .unwrap();
    std::fs::write(root.join("a.toml"), "extends = \"base.toml\"\n").unwrap();
    std::fs::write(root.join("b.toml"), "extends = \"base.toml\"\n").unwrap();
    std::fs::write(&file, "extends = [\"a.toml\", \"b.toml\"]\n").unwrap();

    assert_eq!(
        atypical_config::load::<Section>(&file, "commit").unwrap(),
        Some(Section {
            name: "base".into()
        })
    );
}

#[derive(Debug, Default, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Slot {
    name: String,
    kind: String,
    required: bool,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Slots {
    slots: Vec<Slot>,
}

fn slot(name: &str, kind: &str, required: bool) -> Slot {
    Slot {
        name: name.into(),
        kind: kind.into(),
        required,
    }
}

/// Two named entries, in this order, for an extending file to adjust.
fn base(root: &std::path::Path) {
    std::fs::write(
        root.join("base.toml"),
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "keywords"
            kind = "word"

            [[commit.slots]]
            name = "separator"
            kind = "symbol"
        "#},
    )
    .unwrap();
}

#[test]
fn named_entries_merge_field_by_field() {
    let root = tree("named-merge");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "keywords"
            required = true
        "#},
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![
                slot("keywords", "word", true),
                slot("separator", "symbol", false),
            ]
        })
    );
}

#[test]
fn an_unmatched_named_entry_appends() {
    let root = tree("named-append");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "scope"
            kind = "word"
            drop = false
        "#},
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![
                slot("keywords", "word", false),
                slot("separator", "symbol", false),
                slot("scope", "word", false),
            ]
        })
    );
}

#[test]
fn an_unmatched_named_entry_keeps_its_place_among_matched_ones() {
    let root = tree("named-in-place");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        root.join("peer.toml"),
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "gitmoji"
            kind = "symbols"

            [[commit.slots]]
            name = "keywords"
            required = true

            [[commit.slots]]
            name = "scope"
            kind = "word"

            [[commit.slots]]
            name = "gone"
            drop = true

            [[commit.slots]]
            name = "modifiers"
            kind = "symbols"
        "#},
    )
    .unwrap();
    std::fs::write(&file, "extends = [\"base.toml\", \"peer.toml\"]\n")
        .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![
                slot("gitmoji", "symbols", false),
                slot("keywords", "word", true),
                slot("scope", "word", false),
                slot("modifiers", "symbols", false),
                slot("separator", "symbol", false),
            ]
        })
    );
}

#[test]
fn before_places_a_new_entry_ahead_of_the_one_it_names() {
    let root = tree("named-before");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "gitmoji"
            kind = "word"
            before = "keywords"
        "#},
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![
                slot("gitmoji", "word", false),
                slot("keywords", "word", false),
                slot("separator", "symbol", false),
            ]
        })
    );
}

#[test]
fn before_moves_an_entry_that_is_already_there() {
    let root = tree("named-before-move");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "separator"
            required = true
            before = "keywords"
        "#},
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![
                slot("separator", "symbol", true),
                slot("keywords", "word", false),
            ]
        })
    );
}

#[test]
fn before_naming_nothing_present_is_an_error() {
    let root = tree("named-before-missing");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "gitmoji"
            kind = "word"
            before = "keywrods"
        "#},
    )
    .unwrap();

    let typo = atypical_config::load::<Slots>(&file, "commit").unwrap_err();

    assert_matches!(typo, atypical_config::Error::Before(_, ref name)
        if name == "keywrods");
    assert!(typo.to_string().contains("keywrods"));
    assert!(std::error::Error::source(&typo).is_none());
}

#[test]
fn before_naming_its_own_entry_is_an_error() {
    let root = tree("named-before-itself");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "keywords"
            before = "keywords"
        "#},
    )
    .unwrap();

    let itself = atypical_config::load::<Slots>(&file, "commit").unwrap_err();

    assert_matches!(itself, atypical_config::Error::Before(_, ref name)
        if name == "keywords");
}

#[test]
fn one_name_for_two_entries_is_an_error() {
    let root = tree("named-duplicate");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(
        &file,
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "keywords"
            kind = "word"

            [[commit.slots]]
            name = "keywords"
            kind = "symbol"
        "#},
    )
    .unwrap();

    let twice = atypical_config::load::<Slots>(&file, "commit").unwrap_err();

    assert_matches!(twice, atypical_config::Error::Duplicate(_, ref name)
        if name == "keywords");
    assert!(twice.to_string().contains("keywords"));
    assert!(std::error::Error::source(&twice).is_none());
}

#[test]
fn drop_removes_the_entry_it_names() {
    let root = tree("named-drop");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "separator"
            drop = true

            [[commit.slots]]
            name = "nowhere"
            drop = true
        "#},
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![slot("keywords", "word", false)]
        })
    );
}

#[test]
fn named_entries_merge_along_a_chain() {
    let root = tree("named-chain");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        root.join("middle.toml"),
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "separator"
            required = true

            [[commit.slots]]
            name = "scope"
            kind = "word"
        "#},
    )
    .unwrap();
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "middle.toml"

            [[commit.slots]]
            name = "keywords"
            drop = true

            [[commit.slots]]
            name = "scope"
            required = true
        "#},
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![
                slot("separator", "symbol", true),
                slot("scope", "word", true),
            ]
        })
    );
}

#[test]
fn a_named_array_is_normalised_without_a_base() {
    let root = tree("named-standalone");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(
        &file,
        indoc::indoc! {r#"
            [[commit.slots]]
            name = "keywords"
            kind = "word"
            drop = false

            [[commit.slots]]
            name = "gone"
            kind = "word"
            drop = true
        "#},
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Slots>(&file, "commit").unwrap(),
        Some(Slots {
            slots: vec![slot("keywords", "word", false)]
        })
    );
}

#[test]
fn a_drop_that_is_not_a_boolean_is_left_for_the_schema() {
    let root = tree("named-drop-invalid");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = "separator"
            drop = "true"
        "#},
    )
    .unwrap();

    assert!(atypical_config::load::<Slots>(&file, "commit").is_err());
}

#[test]
fn an_entry_whose_name_is_not_a_string_is_not_merged() {
    let root = tree("named-not-a-string");
    let file = root.join(atypical_config::FILE_NAME);

    base(&root);
    std::fs::write(
        &file,
        indoc::indoc! {r#"
            extends = "base.toml"

            [[commit.slots]]
            name = 1
        "#},
    )
    .unwrap();

    let table = atypical_config::resolve(&file).unwrap();
    let slots = table["commit"]["slots"].as_array().unwrap();

    // Replaced wholesale: with no name to match on, there is nothing to
    // merge entry by entry.
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0]["name"].as_integer(), Some(1));
}

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Keywords {
    keywords: Vec<String>,
}

#[test]
fn an_array_without_names_still_replaces() {
    let root = tree("array-replaced");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(
        root.join("preset.toml"),
        "[commit]\nkeywords = [\"add\", \"fix\"]\n",
    )
    .unwrap();
    std::fs::write(
        &file,
        "extends = \"preset.toml\"\n[commit]\nkeywords = [\"ref\"]\n",
    )
    .unwrap();

    assert_eq!(
        atypical_config::load::<Keywords>(&file, "commit").unwrap(),
        Some(Keywords {
            keywords: vec!["ref".into()]
        })
    );
}

#[test]
fn resolve_strips_the_extends_key() {
    let root = tree("extends-stripped");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(root.join("preset.toml"), "").unwrap();
    std::fs::write(&file, "extends = \"preset.toml\"\n").unwrap();

    let table = atypical_config::resolve(&file).unwrap();

    assert!(!table.contains_key("extends"));
}

#[test]
fn cyclic_extends_is_an_error() {
    let root = tree("extends-cycle");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(root.join("loop.toml"), "extends = \"atypical.toml\"\n")
        .unwrap();
    std::fs::write(&file, "extends = \"loop.toml\"\n").unwrap();

    let cycle = atypical_config::load::<Section>(&file, "commit").unwrap_err();

    assert_matches!(cycle, atypical_config::Error::Cycle(_));
    assert!(cycle.to_string().contains("cyclic"));
    assert!(std::error::Error::source(&cycle).is_none());
}

#[test]
fn extends_must_be_a_path_or_an_array_of_paths() {
    let root = tree("extends-invalid");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(&file, "extends = 1\n").unwrap();

    let scalar = atypical_config::load::<Section>(&file, "commit").unwrap_err();

    assert_matches!(scalar, atypical_config::Error::Extends(_));
    assert!(scalar.to_string().contains("extends"));
    assert!(std::error::Error::source(&scalar).is_none());

    std::fs::write(&file, "extends = [1]\n").unwrap();

    assert_matches!(
        atypical_config::load::<Section>(&file, "commit"),
        Err(atypical_config::Error::Extends(_))
    );
}

#[test]
fn extends_to_a_missing_file_is_an_io_error() {
    let root = tree("extends-missing");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(&file, "extends = \"./nowhere.toml\"\n").unwrap();

    assert_matches!(
        atypical_config::load::<Section>(&file, "commit"),
        Err(atypical_config::Error::Io(_))
    );
}

#[test]
fn extends_an_npm_package_file_loads() {
    let root = tree("extends-npm");
    let file = root.join(atypical_config::FILE_NAME);
    let package = root.join("node_modules/atypical-preset");

    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        "{ \"name\": \"atypical-preset\" }\n",
    )
    .unwrap();
    std::fs::write(package.join("preset.toml"), "[commit]\nname = \"npm\"\n")
        .unwrap();
    std::fs::write(&file, "extends = \"npm:atypical-preset/preset.toml\"\n")
        .unwrap();

    assert_eq!(
        atypical_config::load::<Section>(&file, "commit").unwrap(),
        Some(Section { name: "npm".into() })
    );
}

#[test]
fn extends_an_absent_npm_package_is_a_resolve_error() {
    // Outside the repository: resolution would otherwise walk up into
    // this repo's own node_modules above CARGO_TARGET_TMPDIR.
    let root = std::env::temp_dir().join("atypical-extends-npm-missing");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(&file, "extends = \"npm:missing-package\"\n").unwrap();

    let missing =
        atypical_config::load::<Section>(&file, "commit").unwrap_err();

    assert_matches!(missing, atypical_config::Error::Resolve(..));
    assert!(missing.to_string().contains("npm:missing-package"));
    assert!(std::error::Error::source(&missing).is_some());
}

#[test]
fn extends_dot_relative_path_loads() {
    let root = tree("extends-dot");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(root.join("base.toml"), "[commit]\nname = \"base\"\n")
        .unwrap();
    std::fs::write(&file, "extends = \"./base.toml\"\n").unwrap();

    assert_eq!(
        atypical_config::load::<Section>(&file, "commit").unwrap(),
        Some(Section {
            name: "base".into()
        })
    );
}

#[test]
fn extends_a_prefix_under_two_chars_is_a_file() {
    let root = tree("extends-short-prefix");
    let file = root.join(atypical_config::FILE_NAME);

    for (base, name) in [(":empty.toml", "empty"), ("c:drive.toml", "drive")] {
        std::fs::write(
            root.join(base),
            format!("[commit]\nname = \"{name}\"\n"),
        )
        .unwrap();
        std::fs::write(&file, format!("extends = \"{base}\"\n")).unwrap();

        assert_eq!(
            atypical_config::load::<Section>(&file, "commit").unwrap(),
            Some(Section { name: name.into() }),
            "{base}"
        );
    }
}

#[test]
fn extends_unknown_scheme_is_rejected() {
    let root = tree("extends-scheme");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(&file, "extends = \"bogus:preset.toml\"\n").unwrap();

    let bogus = atypical_config::load::<Section>(&file, "commit").unwrap_err();

    assert_matches!(
        &bogus,
        atypical_config::Error::Scheme(scheme) if scheme == "bogus"
    );
    assert!(bogus.to_string().contains("bogus:"));
    assert!(std::error::Error::source(&bogus).is_none());
}

#[test]
fn load_errors_are_displayed() {
    let root = tree("load-errors");
    let file = root.join(atypical_config::FILE_NAME);

    let missing =
        atypical_config::load::<Section>(root.join("nowhere.toml"), "commit")
            .unwrap_err();

    assert!(!missing.to_string().is_empty());
    assert!(std::error::Error::source(&missing).is_some());

    std::fs::write(&file, "not toml").unwrap();

    let invalid =
        atypical_config::load::<Section>(&file, "commit").unwrap_err();

    assert!(!invalid.to_string().is_empty());
    assert!(std::error::Error::source(&invalid).is_some());
}
