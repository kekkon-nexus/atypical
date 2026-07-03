#![feature(coverage_attribute)]

use std::process::{ExitCode, Termination};

use anyhow::Result;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
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
    long_about = "\
        Lint commit messages with atypical.\n\
        Check out the documentation for more details: https://github.com/kekkon-nexus/atypical\
    ",
    after_help = "\
        Exit codes:\n  \
          0  the commit message is valid\n  \
          1  the commit message failed linting, or input could not be read\n  \
          2  usage error, or no commit message to lint\
    "
)]
struct Args {
    input: Option<FileOrStdin>,
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

#[coverage(off)]
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
    let result = atypical_commit::header().parse(header);

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
