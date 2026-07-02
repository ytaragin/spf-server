//! Persistent JSON representation of the card data model.
//!
//! Layout on disk:
//! ```text
//! <root>/<year>/index.json      # LeagueManifest: lists every team file
//! <root>/<year>/<TeamName>.json # TeamData: one team's full roster
//! ```
//!
//! The player list inside each team file is a JSON array of the internally
//! tagged [`Player`] enum, e.g. `{ "QB": { ... } }`. Persisting the tagged enum
//! (rather than trait objects) is what makes the round-trip type-safe: on load
//! each entry deserializes into a concrete `*Stats` and is rebuilt into a
//! `Box<dyn BasePlayer>` via [`Roster::from_players`].

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::players::{Player, Roster, TeamID, TeamList};

pub const FORMAT_VERSION: u32 = 1;
pub const MANIFEST_FILE: &str = "index.json";

/// One team's persisted roster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamData {
    pub team: TeamID,
    pub players: Vec<Player>,
}

/// An entry in the league manifest pointing at a single team file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamEntry {
    /// The true team name (may contain characters not allowed in file names).
    pub name: String,
    /// The team's year (part of its identity).
    pub year: String,
    /// The (sanitized) file name of this team's roster, relative to the year dir.
    pub file: String,
}

/// Top-level manifest for a single year's data directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueManifest {
    pub year: String,
    pub format_version: u32,
    pub teams: Vec<TeamEntry>,
}

/// Sanitize a team name into a safe file stem.
///
/// Replaces path separators, whitespace and other awkward characters with `_`
/// so names like `N.Y. Giants` become `N.Y._Giants`. The manifest preserves the
/// real name, so this transformation never needs to be reversed.
pub fn sanitize_file_stem(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' => c,
            _ => '_',
        })
        .collect()
}

/// Serialize an in-memory [`TeamList`] to disk under `root/<year>/`.
///
/// Writes one `<TeamName>.json` per team plus an `index.json` manifest.
pub fn write_league(root: &str, year: &str, teams: &TeamList) -> Result<(), String> {
    let year_dir: PathBuf = Path::new(root).join(year);
    fs::create_dir_all(&year_dir)
        .map_err(|e| format!("Could not create {}: {}", year_dir.display(), e))?;

    let mut entries: Vec<TeamEntry> = Vec::new();

    for (team_id, roster) in &teams.teams {
        let players: Vec<Player> = roster
            .get_all_players()
            .iter()
            .map(|p| p.get_full_player())
            .collect();

        let data = TeamData {
            team: team_id.clone(),
            players,
        };

        let file_name = format!("{}.json", sanitize_file_stem(&team_id.name));
        let file_path = year_dir.join(&file_name);

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Serializing {}: {}", team_id.name, e))?;
        fs::write(&file_path, json)
            .map_err(|e| format!("Writing {}: {}", file_path.display(), e))?;

        entries.push(TeamEntry {
            name: team_id.name.clone(),
            year: team_id.year.clone(),
            file: file_name,
        });
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));

    let manifest = LeagueManifest {
        year: year.to_string(),
        format_version: FORMAT_VERSION,
        teams: entries,
    };

    let manifest_path = year_dir.join(MANIFEST_FILE);
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Serializing manifest: {}", e))?;
    fs::write(&manifest_path, manifest_json)
        .map_err(|e| format!("Writing {}: {}", manifest_path.display(), e))?;

    Ok(())
}

/// Read a single team file into a [`Roster`].
pub fn load_team(path: &Path) -> Result<Roster, String> {
    let json =
        fs::read_to_string(path).map_err(|e| format!("Reading {}: {}", path.display(), e))?;
    let data: TeamData =
        serde_json::from_str(&json).map_err(|e| format!("Parsing {}: {}", path.display(), e))?;
    Ok(Roster::from_players(data.team, data.players))
}

/// Load a full [`TeamList`] from a year directory (e.g. `data/1983`).
///
/// Reads the manifest, then each referenced team file, and rebuilds the runtime
/// model (including the per-position id lookup maps).
pub fn load_league(year_dir: &str) -> Result<TeamList, String> {
    let dir = Path::new(year_dir);
    let manifest_path = dir.join(MANIFEST_FILE);

    let manifest_json = fs::read_to_string(&manifest_path).map_err(|e| {
        format!(
            "Could not read data manifest {} ({}). Has the data been generated with `spf-cli convert`?",
            manifest_path.display(),
            e
        )
    })?;
    let manifest: LeagueManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| format!("Parsing {}: {}", manifest_path.display(), e))?;

    if manifest.format_version != FORMAT_VERSION {
        return Err(format!(
            "Data format version mismatch in {}: found {}, expected {}",
            manifest_path.display(),
            manifest.format_version,
            FORMAT_VERSION
        ));
    }

    let mut rosters: Vec<Roster> = Vec::with_capacity(manifest.teams.len());
    for entry in &manifest.teams {
        let team_path = dir.join(&entry.file);
        rosters.push(load_team(&team_path)?);
    }

    Ok(TeamList::from_rosters(rosters))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_file_stem_replaces_unsafe_chars() {
        assert_eq!(sanitize_file_stem("Atlanta"), "Atlanta");
        assert_eq!(sanitize_file_stem("N.Y. Giants"), "N.Y._Giants");
        assert_eq!(sanitize_file_stem("L.A. Raiders"), "L.A._Raiders");
        assert_eq!(sanitize_file_stem("San Francisco"), "San_Francisco");
    }

    // Full round-trip: parse cards -> write JSON -> load JSON, and verify the
    // in-memory model survives. Skipped automatically if the source cards are
    // not present (e.g. in a checkout without the card data).
    #[test]
    fn test_convert_and_reload_round_trip() {
        let cards_dir = "../cards/SPFB1983";
        if !Path::new(cards_dir).join("83QB.txt").exists() {
            eprintln!("skipping round-trip test: {} not present", cards_dir);
            return;
        }

        let out_root = std::env::temp_dir().join("spf_core_persist_test");
        let _ = fs::remove_dir_all(&out_root);
        let out = out_root.to_str().unwrap();

        let original = TeamList::create_teams(cards_dir);
        write_league(out, "1983", &original).expect("write_league");

        let year_dir = format!("{}/1983", out);
        let reloaded = load_league(&year_dir).expect("load_league");

        assert_eq!(original.teams.len(), reloaded.teams.len());

        // Spot-check a known team and player id survive with identical stats.
        for (team_id, roster) in &original.teams {
            let reloaded_roster = reloaded
                .get_team(team_id)
                .unwrap_or_else(|| panic!("missing team {}", team_id.name));
            assert_eq!(
                roster.get_all_players().len(),
                reloaded_roster.get_all_players().len(),
                "player count mismatch for {}",
                team_id.name
            );
        }

        // A QB looked up by id should deserialize back with its passing stats.
        let qb = reloaded
            .get_player(&"QB-0".to_string())
            .expect("QB-0 present after reload");
        let json = qb.get_json();
        assert!(json.get("quick").is_some(), "QB quick stats present");

        let _ = fs::remove_dir_all(&out_root);
    }
}
