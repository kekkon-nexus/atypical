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
    let root = tree("find-none");

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
