mod args;

use aoc_client::{AocClient, AocError, AocResult};
use args::{Args, Command, GenerateCompletionCommand};
use clap::{crate_description, crate_name, CommandFactory, Parser};
use clap_complete::generate;
use env_logger::{Builder, Env};
use exit_code::*;
use log::{error, info, warn, LevelFilter};
use std::io;
use std::process::exit;

fn main() {
    let args = Args::parse();

    setup_log(&args);

    info!("🎄 {} - {}", crate_name!(), crate_description!());

    match build_client(&args).and_then(|client| run(&args, client)) {
        Ok(_) => exit(SUCCESS),
        Err(err) => {
            error!("🔔 {err}");
            let exit_code = match err {
                AocError::InvalidPuzzleDate(..) => USAGE_ERROR,
                AocError::InvalidEventYear(..) => USAGE_ERROR,
                AocError::InvalidPuzzleDay(..) => USAGE_ERROR,
                AocError::LockedPuzzle(..) => USAGE_ERROR,
                AocError::SessionFileNotFound => NO_INPUT,
                AocError::SessionFileReadError { .. } => IO_ERROR,
                AocError::InvalidSessionCookie { .. } => DATA_ERROR,
                AocError::HttpRequestError { .. } => FAILURE,
                AocError::AocResponseError => FAILURE,
                AocError::PrivateLeaderboardNotAvailable => FAILURE,
                AocError::FileWriteError { .. } => CANNOT_CREATE,
                AocError::ClientFieldMissing(..) => USAGE_ERROR,
                AocError::InvalidPuzzlePart => USAGE_ERROR,
                AocError::InvalidOutputWidth => USAGE_ERROR,
            };

            if exit_code == FAILURE {
                // Unexpected responses from adventofcode.com including
                // HTTP 302/400/500 may be due to invalid or expired cookies
                warn!(
                    "🍪 Your session cookie may be invalid or expired, try \
                    logging in again"
                );
            }

            exit(exit_code);
        }
    };
}

fn setup_log(args: &Args) {
    let mut log_builder =
        Builder::from_env(Env::default().default_filter_or("info"));

    if args.quiet {
        log_builder.filter_module("aoc", LevelFilter::Error);
    } else if args.debug {
        log_builder.filter_module("aoc", LevelFilter::Debug);
    }

    log_builder.format_timestamp(None).init();
}

fn build_client(args: &Args) -> AocResult<AocClient> {
    let mut builder = AocClient::builder();

    if let Some(file) = &args.session_file {
        builder.session_cookie_from_file(file)?;
    } else {
        builder.session_cookie_from_default_locations()?;
    }

    match (args.year, args.day) {
        (Some(year), Some(day)) => builder.year(year)?.day(day)?,
        (Some(year), None) => builder.year(year)?.latest_puzzle_day()?,
        (None, Some(day)) => builder.latest_event_year()?.day(day)?,
        (None, None) => builder.latest_puzzle_day()?,
    };

    if let Some(width) = args.width {
        builder.output_width(width)?;
    }

    builder
        .input_filename(&args.input_file)
        .puzzle_filename(&args.puzzle_file)
        .overwrite_files(args.overwrite)
        .show_html_markup(args.show_html_markup)
        .build()
}

fn run(args: &Args, client: AocClient) -> AocResult<()> {
    match &args.command {
        Some(command) => match command {
            Command::Calendar => client.show_calendar(),
            Command::Download => {
                if !args.input_only {
                    client.save_puzzle_markdown()?;
                }
                if !args.puzzle_only {
                    client.save_input()?;
                }
                Ok(())
            }
            Command::Submit { part, answer } => {
                client.submit_answer_and_show_outcome(part, answer)
            }
            Command::PrivateLeaderboard { leaderboard_id } => {
                client.show_private_leaderboard(*leaderboard_id)
            }
            Command::Read => client.show_puzzle(),
            Command::GenerateCompletion(command) => {
                generate_completion(command);
                Ok(())
            }
        },
        None => client.show_puzzle(),
    }
}

/// Generate a completion script.
fn generate_completion(command: &GenerateCompletionCommand) {
    let shell = command.shell;
    let mut app = Args::command();
    let bin_name = env!("CARGO_BIN_NAME");
    generate(shell, &mut app, bin_name, &mut io::stdout());
}
