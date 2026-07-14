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

    assert!(matches!(cycle, atypical_config::Error::Cycle(_)));
    assert!(cycle.to_string().contains("cyclic"));
    assert!(std::error::Error::source(&cycle).is_none());
}

#[test]
fn extends_must_be_a_path_or_an_array_of_paths() {
    let root = tree("extends-invalid");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(&file, "extends = 1\n").unwrap();

    let scalar = atypical_config::load::<Section>(&file, "commit").unwrap_err();

    assert!(matches!(scalar, atypical_config::Error::Extends(_)));
    assert!(scalar.to_string().contains("extends"));
    assert!(std::error::Error::source(&scalar).is_none());

    std::fs::write(&file, "extends = [1]\n").unwrap();

    assert!(matches!(
        atypical_config::load::<Section>(&file, "commit"),
        Err(atypical_config::Error::Extends(_))
    ));
}

#[test]
fn extends_to_a_missing_file_is_an_io_error() {
    let root = tree("extends-missing");
    let file = root.join(atypical_config::FILE_NAME);

    std::fs::write(&file, "extends = \"nowhere.toml\"\n").unwrap();

    assert!(matches!(
        atypical_config::load::<Section>(&file, "commit"),
        Err(atypical_config::Error::Io(_))
    ));
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
