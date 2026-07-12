//! [`GameEnvironment`]: the shared, read-only, long-lived bundle of external data every
//! game needs to run (the league and the FAC deck template).
//!
//! It is the single place that loads these resources from disk. It is created once in
//! `main`, held in the server's application state, and **borrowed** (never consumed) when a
//! game is created, so one environment can back many games. See
//! `docs/design/game-management.md` for the layering and ownership model.

use spf_core::persist;
use spf_core::players::{Roster, TeamID, TeamList};

use super::fac::FacManager;

/// All external data a [`Game`](super::Game) depends on.
///
/// Fields are private and split by sharing profile:
/// - `league` is read-only during play and shared across all games (never cloned into a game).
/// - `fac_deck` is a *template*; each game receives its own clone via [`new_deck`](Self::new_deck)
///   because the deck is mutated (consumed) as a game runs.
pub struct GameEnvironment {
    league: TeamList,
    fac_deck: FacManager,
}

impl GameEnvironment {
    /// Load every external resource from disk. This is the *only* disk-loading site for game
    /// data; all resource errors are surfaced here as `Err(String)` rather than panicking.
    pub fn load(data_dir: &str, fac_path: &str) -> Result<Self, String> {
        let league = persist::load_league(data_dir)?;
        let fac_deck = FacManager::from_csv(fac_path).map_err(|e| e.to_string())?;
        Ok(Self { league, fac_deck })
    }

    /// Read-only access to the league, for endpoints that look players/teams up.
    // Public API for HTTP handlers; no in-crate caller yet (team resolution happens inside
    // `Game::create_game`), so allow dead_code until a league-reading endpoint lands.
    #[allow(dead_code)]
    pub fn league(&self) -> &TeamList {
        &self.league
    }

    /// Resolve a roster by team id (read-only borrow into the shared league).
    pub(crate) fn roster(&self, id: &TeamID) -> Option<&Roster> {
        self.league.get_team(id)
    }

    /// A fresh, independent deck for a new game (clones the shared template).
    pub(crate) fn new_deck(&self) -> FacManager {
        self.fac_deck.clone()
    }

    /// Test-only constructor: assemble an environment from an in-memory league and deck,
    /// bypassing disk loading.
    #[cfg(test)]
    pub(crate) fn from_parts(league: TeamList, fac_deck: FacManager) -> Self {
        Self { league, fac_deck }
    }
}
