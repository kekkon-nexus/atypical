use std::process::{Command, Output};

use anyhow::{Context, Result, bail};

/// Record and field separators, chosen because a commit message is
/// far likelier to hold a newline than either. A message that does
/// hold one splits early: the sha and the header before it survive,
/// since git writes them first, and the remnant is dropped for having
/// no field separator of its own.
const RECORD: char = '\x1e';
const FIELD: char = '\x1f';

fn git(args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .context("Cannot run `git`; is it on PATH?")
}

/// The `<sha><FIELD><message><RECORD>` records `git log` wrote, which
/// it separates from one another with a newline of its own.
fn records(log: &str) -> Vec<(String, String)> {
    log.split_terminator(RECORD)
        .filter_map(|commit| commit.trim_start_matches('\n').split_once(FIELD))
        .map(|(sha, message)| (sha.to_owned(), message.to_owned()))
        .collect()
}

/// `from..to` as `git log` spells it; without `from`, everything
/// reachable from `to`.
fn range(from: Option<&str>, to: &str) -> String {
    match from {
        Some(from) => format!("{from}..{to}"),
        None => to.to_owned(),
    }
}

/// The commits in `from..to`, newest first: `from` is excluded and
/// `to` included, as `git log from..to` walks them. `to` defaults to
/// `HEAD`; without `from` the whole history reachable from `to` is
/// walked.
pub fn commits(
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let to = to.unwrap_or("HEAD");

    if let Some(from) = from {
        // Without a common ancestor `from..to` yields only the commits
        // that happen to be present, as in a shallow clone.
        let base = git(&["merge-base", from, to])?;

        if base.status.code() == Some(1) {
            bail!(
                "Cannot find merge-base between '{from}' and '{to}'. This \
                 typically indicates incomplete git history (e.g. a shallow \
                 clone). Consider fetching more history."
            );
        }

        if !base.status.success() {
            bail!("{}", String::from_utf8_lossy(&base.stderr).trim());
        }
    }

    let format = format!("--format=%h{FIELD}%B{RECORD}");
    let log = git(&["log", &format, &range(from, to)])?;

    if !log.status.success() {
        bail!("{}", String::from_utf8_lossy(&log.stderr).trim());
    }

    // Git stores message bytes as they were given, so a commit made
    // under another encoding must still lint rather than abort.
    Ok(records(&String::from_utf8_lossy(&log.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log(commits: &[(&str, &str)]) -> String {
        commits
            .iter()
            .map(|(sha, message)| format!("{sha}{FIELD}{message}{RECORD}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn records_splits_every_commit() {
        let log = log(&[("abc1234", "add: one\n"), ("def5678", "fix: two\n")]);

        assert_eq!(
            records(&log),
            [
                ("abc1234".to_owned(), "add: one\n".to_owned()),
                ("def5678".to_owned(), "fix: two\n".to_owned()),
            ]
        );
    }

    #[test]
    fn records_keeps_bodies_whole() {
        let message = "add: one\n\nA body, with a blank line and a \x1c.\n";
        let log = log(&[("abc1234", message)]);

        assert_eq!(records(&log), [("abc1234".to_owned(), message.to_owned())]);
    }

    #[test]
    fn records_keeps_empty_messages() {
        let log = log(&[("abc1234", ""), ("def5678", "fix: two\n")]);

        assert_eq!(records(&log).first().unwrap().1, "");
    }

    #[test]
    fn records_of_an_empty_range_are_empty() {
        assert!(records("").is_empty());
    }

    #[test]
    fn records_keep_every_commit_around_a_stray_separator() {
        let log = log(&[
            ("abc1234", "add: one\x1e\nbody\n"),
            ("def5678", "fix: two"),
        ]);

        assert_eq!(
            records(&log),
            [
                ("abc1234".to_owned(), "add: one".to_owned()),
                ("def5678".to_owned(), "fix: two".to_owned()),
            ]
        );
    }

    #[test]
    fn range_excludes_from() {
        assert_eq!(range(Some("abc1234"), "HEAD"), "abc1234..HEAD");
        assert_eq!(range(None, "HEAD"), "HEAD");
    }
}
