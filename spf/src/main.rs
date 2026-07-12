mod game;
mod webendpoint;

use std::process::ExitCode;

use crate::game::environment::GameEnvironment;
use crate::webendpoint::runserver;

/// Directory holding the pre-generated persistent card data for the season the
/// server runs. Produce it with:
///   `cargo run -p spf_cli -- convert --cards-dir cards/SPFB1983 --year 1983`
const DATA_DIR: &str = "data/1983";

/// FAC deck CSV, parsed at runtime (see `docs/design/data-pipeline.md`).
const FAC_PATH: &str = "cards/fac_cards.csv";

fn main() -> ExitCode {
    let env = match GameEnvironment::load(DATA_DIR, FAC_PATH) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to load game environment: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match runserver(env) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Server error: {}", e);
            ExitCode::FAILURE
        }
    }
}
