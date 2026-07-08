use std::path::PathBuf;
use std::process::{ExitCode, Termination};

use anyhow::Result;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use atypical_commit::config::{self, CommitConfig};
use clap::Parser;
use clap_stdin::FileOrStdin;

#[repr(u8)]
enum Exit {
    /// The commit message is valid.
    Success = 0,
    /// The commit message failed linting.
    /// Unexpected errors (e.g. unreadable input) also exit with 1,
    /// via the std `Termination` impl for `Result`.
    Invalid = 1,
    /// Nothing to lint, or an invalid invocation (clap also uses 2).
    Usage = 2,
}

impl Termination for Exit {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

#[derive(Debug)]
#[derive(Parser)]
#[command(
    version,
    about = "Lint commit messages with atypical.",
    long_about = indoc::indoc! {r#"
        Lint commit messages with atypical.
        Check out the documentation for more details: https://github.com/kekkon-nexus/atypical
    "#},
    after_help = indoc::indoc! {r#"
        Exit codes:
          0  the commit message is valid
          1  the commit message failed linting, or input could not be read
          2  usage error, or no commit message to lint
    "#}
)]
struct Args {
    input: Option<FileOrStdin>,

    /// Path to atypical.toml; the nearest one from the current
    /// directory upward is used when omitted.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

/// The `[commit]` section of the nearest (or given) atypical.toml,
/// falling back to the standard preset.
fn commit_config(path: Option<PathBuf>) -> Result<CommitConfig> {
    let path = match path {
        Some(path) => Some(path),
        None => atypical_config::find(std::env::current_dir()?),
    };

    let config = match path {
        Some(path) => atypical_config::load(path, config::SECTION)?,
        None => None,
    };

    Ok(config.unwrap_or_default())
}

/// The header is the first line git keeps: leading blank lines and
/// `#` comments are stripped from the recorded message.
fn message_header(input: &str) -> Option<(usize, &str)> {
    let mut offset = 0;

    for line in input.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);

        if !content.trim().is_empty() && !content.trim_start().starts_with('#')
        {
            return Some((offset, content));
        }

        offset += line.len();
    }

    None
}

fn header_parser<'i>(
    tokens: &'i atypical_commit::Tokens<'i>,
) -> impl chumsky::Parser<
    'i,
    &'i str,
    atypical_commit::Header<'i>,
    atypical_commit::Extra<'i>,
> {
    use chumsky::Parser;

    atypical_commit::header()
        .with_ctx(atypical_commit::ExtraContext::new(tokens))
}

fn main() -> Result<Exit> {
    let args = Args::parse();

    let Some(input) = args.input else {
        eprintln!("No input provided.");
        return Ok(Exit::Usage);
    };

    let filename = input.filename().to_owned();
    let input = input.contents()?;

    let Some((offset, header)) = message_header(&input) else {
        eprintln!("No commit message to lint.");
        return Ok(Exit::Usage);
    };

    use chumsky::Parser;
    let config = commit_config(args.config)?;

    if config.default_ignores && atypical_commit::ignore::is_ignored(header) {
        return Ok(Exit::Success);
    }

    let tokens = atypical_commit::Tokens::from(&config);
    let result = header_parser(&tokens).parse(header);

    if !result.has_errors() {
        return Ok(Exit::Success);
    }

    let mut report = Report::build(
        ReportKind::Error,
        (&filename, offset..offset + header.len()),
    )
    .with_message("Failed to parse commit message")
    .with_code(3);

    let mut colors = ColorGenerator::new();

    for error in result.errors() {
        let range = error.span().into_range();

        report = report.with_label(
            Label::new((&filename, offset + range.start..offset + range.end))
                .with_message(error.to_string())
                .with_color(colors.next()),
        );
    }

    report.finish().eprint((&filename, Source::from(&input)))?;

    Ok(Exit::Invalid)
}
