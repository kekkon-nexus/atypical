use anyhow::Result;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use clap::Parser;
use clap_stdin::FileOrStdin;

#[derive(Debug)]
#[derive(Parser)]
#[command(
    version,
    about = "Lint commit messages with atypical.",
    long_about = "\
        Lint commit messages with atypical.\n\
        Check out the documentation for more details: https://github.com/kekkon-nexus/atypical\
    "
)]
struct Args {
    input: Option<FileOrStdin>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(input) = args.input {
        let filename = input.filename().to_owned();
        let input = input.contents()?;

        println!("Input: {}", input);

        use chumsky::Parser;
        let result = atypical_commit::prefix().parse(&input);

        if result.has_errors() {
            let mut report =
                Report::build(ReportKind::Error, (&filename, 0..input.len()))
                    .with_message("Failed to parse commit message")
                    .with_code(3);

            let mut colors = ColorGenerator::new();

            for error in result.errors() {
                report = report
                .with_label(
                    Label::new((&filename, error.span().into_range()))
                        .with_message("Invalid commit message")
                        .with_color(colors.next()),
                );
            }

            report.finish().print((&filename, Source::from(&input)))?;
        }
    } else {
        println!("No input provided.");
    }

    Ok(())
}
