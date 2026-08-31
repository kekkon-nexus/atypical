use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_commit-lint");

const STANDARD_PRESET: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../presets/standard.toml");

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

/// As `lint_in`, but with nothing on `PATH`, so that `git` cannot be
/// found.
fn lint_without_git(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .env("PATH", "")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn fixture(name: &str, contents: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);

    std::fs::write(&path, contents).unwrap();

    path
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn git(dir: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

/// A throwaway repository holding one empty commit per message, in
/// order, pinned to the standard preset.
fn repo(name: &str, messages: &[&str]) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);

    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("atypical.toml"),
        format!("extends = '{STANDARD_PRESET}'\n"),
    )
    .unwrap();

    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["config", "user.email", "lint@example.com"]);
    git(&dir, &["config", "user.name", "Lint"]);

    for message in messages {
        commit(&dir, message);
    }

    dir
}

fn commit(dir: &Path, message: &str) {
    let output = git(
        dir,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "--allow-empty",
            "--allow-empty-message",
            "--no-verify",
            "-m",
            message,
        ],
    );

    assert!(output.status.success(), "{}", stderr(&output));
}

fn rev(dir: &Path, spec: &str) -> String {
    let output = git(dir, &["rev-parse", spec]);

    assert!(output.status.success(), "{spec}: {}", stderr(&output));

    String::from_utf8(output.stdout).unwrap().trim().to_owned()
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
fn generated_messages_are_ignored() {
    for header in [
        "Merge pull request #1 from repo/branch\n",
        "Merge branch 'main' into fix/default-ignore\n",
        "Revert \"add(lib)[int]: something\"\n",
        "fixup! add(lib)[int]: something\n",
        "chore(release): v1.2.3 [skip ci]\n",
    ] {
        let output = lint(&["-"], Some(header));

        assert_eq!(output.status.code(), Some(0), "not ignored: {header}");
    }
}

#[test]
fn default_ignores_can_be_disabled() {
    let config = fixture(
        "no-ignores.toml",
        &format!(
            "extends = '{STANDARD_PRESET}'\n[commit]\ndefault-ignores = false\n"
        ),
    );
    let config = config.to_str().unwrap();
    let header = Some("Merge branch 'main'\n");

    let output = lint(&["--config", config, "-"], header);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("unknown keyword `Merge`"));
}

#[test]
fn invalid_keyword_reports_and_fails() {
    let output = lint(
        &["--config", STANDARD_PRESET, "-"],
        Some("feat: wrong style\n"),
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("unknown keyword `feat`"));
}

#[test]
fn unconfigured_lints_nothing() {
    let config = fixture("no-section.toml", "");
    let config = config.to_str().unwrap();

    for header in [
        "add(lib)[int]: standard style\n",
        "feat(api)!: conventional style\n",
        "not a header at all\n",
    ] {
        let output = lint(&["--config", config, "-"], Some(header));

        assert_eq!(output.status.code(), Some(0), "rejected: {header}");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn empty_commit_section_enforces_only_the_shape() {
    let config = fixture("empty-section.toml", "[commit]\n");
    let config = config.to_str().unwrap();

    for header in [
        "add(lib)[int]: standard style\n",
        "feat(api)!: conventional style\n",
        "yolo(whatever)> ship it\n",
    ] {
        let output = lint(&["--config", config, "-"], Some(header));

        assert_eq!(output.status.code(), Some(0), "rejected: {header}");
    }

    let output = lint(&["--config", config, "-"], Some("no separator here\n"));

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("Failed to parse commit message"));
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

#[test]
fn range_lints_every_commit_after_from() {
    let dir = repo(
        "range-valid",
        &["add(exe)[int]: one", "fix(ci): two", "ref(lib)[eff]: three"],
    );
    let root = rev(&dir, "HEAD~2");

    let output = lint_in(&dir, &["--from", &root, "--to", "HEAD"], None);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn range_reports_every_invalid_commit() {
    let dir = repo(
        "range-invalid",
        &["add(exe)[int]: one", "feat: two", "chore: three"],
    );
    let root = rev(&dir, "HEAD~2");
    let (second, third) = (rev(&dir, "HEAD~1"), rev(&dir, "HEAD"));

    let output = lint_in(&dir, &["--from", &root], None);
    let stderr = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("unknown keyword `feat`"), "{stderr}");
    assert!(stderr.contains("unknown keyword `chore`"), "{stderr}");
    assert!(stderr.contains(&second[..7]), "{stderr}");
    assert!(stderr.contains(&third[..7]), "{stderr}");
}

#[test]
fn range_excludes_from() {
    let dir = repo("range-exclusive", &["feat: bad root", "fix(ci): good"]);
    let root = rev(&dir, "HEAD~1");

    let output = lint_in(&dir, &["--from", &root, "--to", "HEAD"], None);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn range_without_from_walks_whole_history() {
    let dir = repo("range-to-only", &["add(exe)[int]: one", "feat: two"]);

    let output = lint_in(&dir, &["--to", "HEAD"], None);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("unknown keyword `feat`"));
}

#[test]
fn range_applies_default_ignores() {
    let dir = repo(
        "range-ignores",
        &["add(exe)[int]: one", "Merge branch 'main' into topic"],
    );
    let root = rev(&dir, "HEAD~1");

    let output = lint_in(&dir, &["--from", &root], None);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn empty_range_passes() {
    let dir = repo("range-empty", &["add(exe)[int]: one"]);

    let output = lint_in(&dir, &["--from", "HEAD", "--to", "HEAD"], None);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn empty_commit_message_in_range_fails() {
    let dir = repo("range-empty-message", &["add(exe)[int]: one", ""]);
    let root = rev(&dir, "HEAD~1");

    let output = lint_in(&dir, &["--from", &root], None);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("no commit message to lint."));
}

#[test]
fn range_without_merge_base_fails() {
    let dir = repo("range-unrelated", &["add(exe)[int]: one"]);

    git(&dir, &["checkout", "-q", "--orphan", "other"]);
    commit(&dir, "add(exe)[int]: unrelated");

    let output = lint_in(&dir, &["--from", "main", "--to", "other"], None);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("Cannot find merge-base"));
}

#[test]
fn unknown_revision_fails() {
    let dir = repo("range-unknown-rev", &["add(exe)[int]: one"]);

    let output = lint_in(&dir, &["--from", "nope", "--to", "HEAD"], None);

    assert_eq!(output.status.code(), Some(1));

    let output = lint_in(&dir, &["--to", "nope"], None);

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn range_conflicts_with_input() {
    let output = lint(&["--from", "HEAD", "-"], Some("add: message\n"));

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn range_without_git_fails() {
    let dir = repo("range-no-git", &["add(exe)[int]: one"]);

    for args in [["--from", "HEAD"], ["--to", "HEAD"]] {
        let output = lint_without_git(&dir, &args);

        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(stderr(&output).contains("Cannot run `git`"), "{args:?}");
    }
}

#[test]
fn range_lints_messages_of_another_encoding() {
    let dir = repo("range-encoding", &["add(exe)[int]: one"]);
    let message = dir.join("latin1-message");

    // Latin-1 `é`, which is not valid UTF-8 on its own.
    std::fs::write(&message, b"add(exe)[int]: caf\xe9\n").unwrap();

    let output = git(
        &dir,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "--allow-empty",
            "--no-verify",
            "-F",
            message.to_str().unwrap(),
        ],
    );

    assert!(output.status.success(), "{}", stderr(&output));

    let root = rev(&dir, "HEAD~1");
    let output = lint_in(&dir, &["--from", &root], None);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn unconfigured_range_walks_nothing() {
    let dir = repo("range-unconfigured", &["add(exe)[int]: one"]);
    let config = fixture("range-no-section.toml", "");
    let config = config.to_str().unwrap();

    // Without `git` there is no range to walk, so exiting 0 proves the
    // walk never started.
    let output =
        lint_without_git(&dir, &["--config", config, "--from", "HEAD"]);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(output.stderr.is_empty(), "{}", stderr(&output));
}
