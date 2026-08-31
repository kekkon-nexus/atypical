use std::process::{Command, Output};

use anyhow::{Context, Result, bail};

/// Record and field separators, so that a commit message can hold
/// anything a commit message can hold.
const RECORD: char = '\x1e';
const FIELD: char = '\x1f';

fn git(args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .context("Cannot run `git`; is it on PATH?")
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

    let range = match from {
        Some(from) => format!("{from}..{to}"),
        None => to.to_owned(),
    };

    let format = format!("--format=%h{FIELD}%B{RECORD}");
    let log = git(&["log", &format, &range])?;

    if !log.status.success() {
        bail!("{}", String::from_utf8_lossy(&log.stderr).trim());
    }

    let log = String::from_utf8(log.stdout)?;

    Ok(log
        .split_terminator(RECORD)
        .filter_map(|commit| commit.trim_start_matches('\n').split_once(FIELD))
        .map(|(sha, message)| (sha.to_owned(), message.to_owned()))
        .collect())
}
