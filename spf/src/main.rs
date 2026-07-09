mod game;
mod webendpoint;

use std::process::ExitCode;

use spf_core::persist;
use spf_core::players::TeamList;

use crate::webendpoint::runserver;

/// Directory holding the pre-generated persistent card data for the season the
/// server runs. Produce it with:
///   `cargo run -p spf_cli -- convert --cards-dir cards/SPFB1983 --year 1983`
const DATA_DIR: &str = "data/1983";

fn main() -> ExitCode {
    let league: TeamList = match persist::load_league(DATA_DIR) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to load card data: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match runserver(league) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Server error: {}", e);
            ExitCode::FAILURE
        }
    }
}
