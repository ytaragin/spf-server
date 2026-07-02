use std::process::ExitCode;

use clap::{Parser, Subcommand};

use spf_core::persist;
use spf_core::players::TeamList;

/// Statis Pro Football data tooling.
///
/// Converts the card text files (the output of `pdftotext` over the scanned
/// PDFs) into the persistent JSON data model consumed by the server.
#[derive(Parser)]
#[command(name = "spf-cli", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert card `.txt` files into the persistent JSON model.
    Convert {
        /// Directory containing the card text files (e.g. cards/SPFB1983).
        #[arg(long)]
        cards_dir: String,

        /// The year/season these cards represent (e.g. 1983).
        #[arg(long)]
        year: String,

        /// Output root directory. Data is written to `<out>/<year>/`.
        #[arg(long, default_value = "data")]
        out: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Convert {
            cards_dir,
            year,
            out,
        } => match convert(&cards_dir, &year, &out) {
            Ok(count) => {
                println!(
                    "Wrote {} team(s) to {}/{}/ (manifest: {})",
                    count,
                    out,
                    year,
                    persist::MANIFEST_FILE
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {}", e);
                ExitCode::FAILURE
            }
        },
    }
}

fn convert(cards_dir: &str, year: &str, out: &str) -> Result<usize, String> {
    println!("Loading cards from {} ...", cards_dir);
    let teams = TeamList::create_teams(cards_dir);
    let count = teams.teams.len();
    persist::write_league(out, year, &teams)?;
    Ok(count)
}
