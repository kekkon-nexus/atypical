use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_commit-lint");

fn lint(args: &[&str], stdin: Option<&str>) -> Output {
    lint_in(Path::new("."), args, stdin)
}

fn lint_in(dir: &Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut child = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(input) = stdin {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }

    child.wait_with_output().unwrap()
}

fn fixture(name: &str, contents: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);

    std::fs::write(&path, contents).unwrap();

    path
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn valid_message_from_stdin() {
    let output = lint(&["-"], Some("add(lib)[int]: something\n"));

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
}

#[test]
fn valid_message_file_with_body_and_comments() {
    let path = fixture(
        "valid-body",
        "fix(ci): report separately\n\nSome body.\n\n# comment\n",
    );

    let output = lint(&[path.to_str().unwrap()], None);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn valid_header_after_blank_lines_and_crlf() {
    let path = fixture("valid-blanks", "\n\nadd(exe)[ux]: from line three\r\n");

    let output = lint(&["--", path.to_str().unwrap()], None);

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn invalid_keyword_reports_and_fails() {
    let output = lint(&["-"], Some("feat: wrong style\n"));

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("unknown keyword `feat`"));
}

#[test]
fn missing_description_fails() {
    let output = lint(&["-"], Some("add:\n"));

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("expected a description"));
}

#[test]
fn comments_only_is_a_usage_error() {
    let path = fixture("comments-only", "# only\n# comments\n");

    let output = lint(&[path.to_str().unwrap()], None);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("No commit message to lint."));
}

#[test]
fn no_input_is_a_usage_error() {
    let output = lint(&[], None);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("No input provided."));
}

#[test]
fn unreadable_file_fails() {
    let output = lint(&["/nonexistent/commit-msg"], None);

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn config_flag_overrides_the_keywords() {
    let config = fixture(
        "keywords.toml",
        "[commit]\nkeywords = [\"feat\", \"fix\"]\n",
    );
    let config = config.to_str().unwrap();

    let output = lint(&["--config", config, "-"], Some("feat: now valid\n"));

    assert_eq!(output.status.code(), Some(0));

    let output = lint(&["--config", config, "-"], Some("add: now unknown\n"));

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("expected one of: feat, fix"));
}

#[test]
fn config_is_discovered_from_the_working_directory() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("discovery");

    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("atypical.toml"),
        "[commit]\nkeywords = [\"feat\"]\n",
    )
    .unwrap();

    let output = lint_in(&dir, &["-"], Some("feat: discovered\n"));

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn invalid_config_fails() {
    let config = fixture("invalid.toml", "[commit]\nkeyword = [\"typo\"]\n");

    let output = lint(
        &["--config", config.to_str().unwrap(), "-"],
        Some("add: message\n"),
    );

    assert_eq!(output.status.code(), Some(1));

    let output = lint(
        &["--config", "/nonexistent/atypical.toml", "-"],
        Some("add: message\n"),
    );

    assert_eq!(output.status.code(), Some(1));
}
