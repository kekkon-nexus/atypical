// Headers that git and forges write on their own — merges, reverts,
// autosquash markers, semver release bumps — should never be linted.
// Mirrors commitlint's default ignores (@commitlint/is-ignored).

use std::borrow::Cow;

pub type Matcher = fn(&str) -> bool;

/// The default ignores, checked against the message header.
pub const WILDCARDS: &[Matcher] = &[
    is_merge,
    is_merge_tag,
    is_revert,
    is_reapply,
    is_autosquash,
    is_semver,
    is_merged,
    is_remote_tracking_merge,
    is_automatic_merge,
    is_auto_merged,
];

/// Whether the header is machine-generated and exempt from linting.
pub fn is_ignored(header: &str) -> bool {
    WILDCARDS.iter().any(|matches| matches(header))
}

/// `Merge pull request #1 ...`, `Merge branch 'x'`, `Merge x into y`.
fn is_merge(header: &str) -> bool {
    header.starts_with("Merge pull request")
        || header.starts_with("Merge branch ")
        || header
            .strip_prefix("Merge ")
            .is_some_and(|rest| rest.contains(" into "))
}

/// `Merge tag 'v1.2.3' ...`.
fn is_merge_tag(header: &str) -> bool {
    header.starts_with("Merge tag ")
}

/// `Revert "..."`, as `git revert` writes it.
fn is_revert(header: &str) -> bool {
    header.starts_with("Revert ") || header.starts_with("revert ")
}

/// `Reapply "..."`, as `git revert` writes for a revert of a revert.
fn is_reapply(header: &str) -> bool {
    header.starts_with("Reapply ") || header.starts_with("reapply ")
}

/// `fixup! ...`, `squash! ...`, `amend! ...`; git drops these prefixes
/// on `rebase --autosquash`, so only the target header gets recorded.
fn is_autosquash(header: &str) -> bool {
    ["amend!", "fixup!", "squash!"]
        .iter()
        .any(|prefix| header.starts_with(prefix))
}

/// A release bump: a bare version, optionally behind a `chore:`
/// prefix and a `[skip ci]`-style marker, e.g. `chore(release): v1.2.3`.
fn is_semver(header: &str) -> bool {
    let stripped = strip_chore_prefix(header);
    let stripped = remove_skip_marker(stripped, '[', ']');
    let stripped = remove_skip_marker(&stripped, '(', ')');
    let stripped = stripped.trim();

    let version = stripped.strip_prefix('v').unwrap_or(stripped);

    semver::Version::parse(version).is_ok()
}

/// `Merged x in(to) y` (Bitbucket), `Merged PR 1: ...` (Azure DevOps).
fn is_merged(header: &str) -> bool {
    header
        .strip_prefix("Merged PR ")
        .is_some_and(|rest| rest.contains(": "))
        || header
            .strip_prefix("Merged ")
            .is_some_and(|rest| rest.contains("in ") || rest.contains("into "))
}

/// `Merge remote-tracking branch '...'`.
fn is_remote_tracking_merge(header: &str) -> bool {
    header.starts_with("Merge remote-tracking branch")
}

/// `Automatic merge ...`.
fn is_automatic_merge(header: &str) -> bool {
    header.starts_with("Automatic merge")
}

/// `Auto-merged x into y`.
fn is_auto_merged(header: &str) -> bool {
    header
        .strip_prefix("Auto-merged ")
        .is_some_and(|rest| rest.contains(" into "))
}

/// Strips `chore:` or `chore(<scope>):`, as release tools write it.
fn strip_chore_prefix(header: &str) -> &str {
    let Some(rest) = header.strip_prefix("chore") else {
        return header;
    };

    if let Some(rest) = rest.strip_prefix(':') {
        return rest;
    }

    if let Some(rest) = rest.strip_prefix('(')
        && let Some((scope, rest)) = rest.split_once(')')
        && !scope.is_empty()
        && let Some(rest) = rest.strip_prefix(':')
    {
        return rest;
    }

    header
}

/// Removes the first `skip`/`ci` pair joined by `-` or whitespace
/// between the given delimiters, e.g. `[skip ci]` or `(CI-skip)`.
fn remove_skip_marker(header: &str, open: char, close: char) -> Cow<'_, str> {
    for (start, _) in header.match_indices(open) {
        let inner = start + open.len_utf8();

        let Some(length) = header[inner..].find(close) else {
            break;
        };

        if is_skip_marker(&header[inner..inner + length]) {
            let end = inner + length + close.len_utf8();

            return Cow::Owned(format!(
                "{}{}",
                &header[..start],
                &header[end..]
            ));
        }
    }

    Cow::Borrowed(header)
}

fn is_skip_marker(inner: &str) -> bool {
    let inner = inner.to_ascii_lowercase();

    let Some(rest) = inner
        .strip_prefix("skip")
        .or_else(|| inner.strip_prefix("ci"))
    else {
        return false;
    };

    let mut rest = rest.chars();
    let joined = rest.next().is_some_and(|c| c == '-' || c.is_whitespace());

    joined && matches!(rest.as_str(), "ci" | "skip")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merges() {
        assert!(is_ignored("Merge pull request #369 from repo/branch"));
        assert!(is_ignored("Merge branch 'main' of github.com:x/y"));
        assert!(is_ignored("Merge my-feature into develop"));
        assert!(is_ignored("Merge tag 'v1.2.3' into main"));
        assert!(is_ignored("Merge remote-tracking branch 'origin/main'"));

        assert!(!is_ignored("Merge conflicts resolved"));
        assert!(!is_ignored("merge branch 'main'"));
    }

    #[test]
    fn test_forge_merges() {
        assert!(is_ignored("Merged in feature/x (pull request #99)"));
        assert!(is_ignored("Merged develop into master"));
        assert!(is_ignored("Merged PR 123: fix stuff"));
        assert!(is_ignored("Automatic merge from CodeStream"));
        assert!(is_ignored("Auto-merged develop into master"));

        assert!(!is_ignored("Merged nothing"));
        assert!(!is_ignored("Merged PR 123"));
        assert!(!is_ignored("Auto-merged develop"));
    }

    #[test]
    fn test_reverts_and_reapplies() {
        assert!(is_ignored("Revert \"add(lib): something\""));
        assert!(is_ignored("revert \"add(lib): something\""));
        assert!(is_ignored("Reapply \"add(lib): something\""));
        assert!(is_ignored("reapply \"add(lib): something\""));

        assert!(!is_ignored("Reverted the change"));
        assert!(!is_ignored("Revert"));
    }

    #[test]
    fn test_autosquash() {
        assert!(is_ignored("fixup! add(lib): something"));
        assert!(is_ignored("squash! add(lib): something"));
        assert!(is_ignored("amend! add(lib): something"));

        assert!(!is_ignored("fixup add(lib): something"));
    }

    #[test]
    fn test_semver() {
        assert!(is_ignored("1.2.3"));
        assert!(is_ignored("v1.2.3"));
        assert!(is_ignored("1.2.3-alpha.1+build.5"));
        assert!(is_ignored("chore: 1.2.3"));
        assert!(is_ignored("chore(release): v1.2.3"));
        assert!(is_ignored("1.2.3 [skip ci]"));
        assert!(is_ignored("1.2.3 (CI-Skip)"));
        assert!(is_ignored("chore: v1.2.3 [skip-ci]"));

        assert!(!is_ignored("1.2"));
        assert!(!is_ignored("1.2.3.4"));
        assert!(!is_ignored("chore(): 1.2.3"));
        assert!(!is_ignored("chore(release: 1.2.3"));
        assert!(!is_ignored("release: v1.2.3"));
        assert!(!is_ignored("1.2.3 [skip ci] and more"));
        assert!(!is_ignored("1.2.3 [not ci]"));
        assert!(!is_ignored("1.2.3 [skip"));
    }

    #[test]
    fn test_plain_headers_are_not_ignored() {
        assert!(!is_ignored(""));
        assert!(!is_ignored("add(lib)[int]: something"));
        assert!(!is_ignored("feat: conventional but invalid here"));
    }
}
