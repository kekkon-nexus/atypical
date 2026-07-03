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
          2  usage error or no input provided\
    "
)]
struct Args {
    input: Option<FileOrStdin>,
}

fn main() -> Result<Exit> {
    let args = Args::parse();

    let Some(input) = args.input else {
        eprintln!("No input provided.");
        return Ok(Exit::Usage);
    };

    let filename = input.filename().to_owned();
    let input = input.contents()?;

    if input.trim().is_empty() {
        eprintln!("No input provided.");
        return Ok(Exit::Usage);
    }

    println!("Input: {}", input);

    use chumsky::Parser;
    let result = atypical_commit::prefix().lazy().parse(&input);

    if !result.has_errors() {
        return Ok(Exit::Success);
    }

    let mut report =
        Report::build(ReportKind::Error, (&filename, 0..input.len()))
            .with_message("Failed to parse commit message")
            .with_code(3);

    let mut colors = ColorGenerator::new();

    for error in result.errors() {
        report = report.with_label(
            Label::new((&filename, error.span().into_range()))
                .with_message(error.to_string())
                .with_color(colors.next()),
        );
    }

    report.finish().eprint((&filename, Source::from(&input)))?;

    Ok(Exit::Invalid)
}
