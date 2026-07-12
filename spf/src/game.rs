pub mod engine;
pub mod environment;
pub mod events;
pub mod fac;
pub mod kickoff_play;
pub mod standard_play;

// The data model, loaders and stat primitives now live in the shared `spf_core`
// crate. Re-export the pieces the server references under the `game` namespace so
// existing intra-crate paths (`crate::game::players::*`, `super::stats::*`, etc.)
// continue to resolve.
pub use spf_core::{lineup, players, stats};

use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use engine::defs::GAMECONSTANTS;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use utoipa::ToSchema;

use self::{
    engine::{
        run_play, DefenseCall, DefenseIDLineup, Down, OffenseCall, OffenseIDLineup, PlayImpl,
        PlayResult, PlayType, Yard,
    },
    environment::GameEnvironment,
    events::GameEvent,
    fac::FacManager,
    kickoff_play::KickoffPlay,
    players::{Roster, TeamID},
    standard_play::StandardPlay,
};

/// Error returned by [`Game::create_game`] when the requested teams can't be resolved.
#[derive(Debug)]
pub enum CreateGameError {
    /// The given team id is not present in the league.
    UnknownTeam(TeamID),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum GameTeams {
    Home,
    Away,
}
impl GameTeams {
    fn other_team(&self) -> GameTeams {
        match self {
            GameTeams::Home => GameTeams::Away,
            GameTeams::Away => GameTeams::Home,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum GamePlayStatus {
    Touchdown,
    Safety,
    FieldGoal,
    PossessionChange,
    Ongoing,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// unused: superseded by the live `GamePlayStatus` state machine; kept pending removal.
#[allow(dead_code)]
pub enum GameStatus {
    Kickoff,
    StandardPlay,
    ExtraPoint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct GameState {
    pub last_status: GamePlayStatus,
    pub quarter: i32,
    pub time_remaining: i32,
    pub possession: GameTeams,
    pub down: Down,
    pub yard_line: Yard,
    pub first_down_target: Yard,
    pub home_score: i32,
    pub away_score: i32,
    pub play_counter: u32,
}

impl GameState {
    pub fn start_state() -> Self {
        return Self {
            last_status: GamePlayStatus::Start,
            quarter: 1,
            time_remaining: GAMECONSTANTS.sec_per_quarter,
            possession: GameTeams::Away,
            down: Down::First,
            yard_line: 50,
            first_down_target: 60,
            home_score: 0,
            away_score: 0,
            play_counter: 0,
        };
    }

    pub fn get_next_move_types(&self) -> Vec<PlayType> {
        match self.last_status {
            GamePlayStatus::Touchdown => vec![PlayType::ExtraPoint],
            GamePlayStatus::Safety => vec![PlayType::Punt],
            GamePlayStatus::FieldGoal => vec![PlayType::Kickoff],
            GamePlayStatus::PossessionChange | GamePlayStatus::Ongoing => {
                vec![PlayType::Standard, PlayType::Punt, PlayType::FieldGoal]
            }
            GamePlayStatus::Start => vec![PlayType::Kickoff],
            GamePlayStatus::End => vec![],
        }
    }

    pub fn get_next_move_default(&self) -> PlayType {
        match self.last_status {
            GamePlayStatus::Touchdown => PlayType::ExtraPoint,
            GamePlayStatus::Safety => PlayType::Punt,
            GamePlayStatus::FieldGoal => PlayType::Kickoff,
            GamePlayStatus::PossessionChange => PlayType::Standard,
            GamePlayStatus::Ongoing => PlayType::Standard,
            GamePlayStatus::Start => PlayType::Kickoff,
            GamePlayStatus::End => PlayType::Kickoff,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Play {
    StandardPlay(StandardPlay),
    Kickoff(KickoffPlay),
}

// impl Play {
//     fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
//         where D: serde::Deserializer<'de>
//     {
//         #[derive(Deserialize)]
//         #[serde(untagged)]
//         enum TaggedPlay {
//             OffensePlay(OffensePlay),
//             KickoffPlay(KickoffPlay)
//         }

//         let tagged_play = TaggedPlay::deserialize(deserializer)?;

//         Ok(match tagged_play {
//             TaggedPlay::OffensePlay(op) => Play::OffensePlay(op),
//             TaggedPlay::KickoffPlay(kp) => Play::KickoffPlay(kp),
//         })
//     }
// }

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlayAndState {
    #[schema(value_type = Object)]
    pub play: Play,
    pub result: PlayResult,
    pub new_state: GameState,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlayTypeInfo {
    pub allowed_types: Vec<PlayType>,
    pub next_type: Option<PlayType>,
}

/// Capacity of the per-`Game` event broadcast channel. Sized well above the largest
/// single-action burst (`run_current_play` emits 2 events) so normal use never lags; a
/// slow/absent consumer receives `Lagged` rather than blocking the producer. See
/// docs/design/ws-events-architecture.md §3.
const GAME_EVENT_CHANNEL_CAPACITY: usize = 128;

#[derive(Serialize)]
pub struct Game {
    #[serde(skip_serializing)]
    pub home: Roster,
    #[serde(skip_serializing)]
    pub away: Roster,
    pub state: GameState,
    pub past_plays: Vec<PlayAndState>,
    #[serde(skip_serializing)]
    pub next_play: Option<Box<dyn PlayImpl + Send>>,
    // pub next_play: Box<dyn PlayImpl + Send>,
    offlineup: Option<OffenseIDLineup>,
    deflineup: Option<DefenseIDLineup>,

    #[serde(skip_serializing)]
    pub fac_deck: FacManager,

    /// Runtime plumbing: broadcasts domain events to transport adapters. Not game data,
    /// so it is skipped in serialization.
    #[serde(skip_serializing)]
    event_tx: broadcast::Sender<GameEvent>,
}

impl Game {
    /// Create a game from the shared [`GameEnvironment`] and the two teams' ids.
    ///
    /// Resolves each team against the environment's league (moving the membership check out
    /// of the HTTP layer), then builds the game with its own cloned FAC deck. The environment
    /// is only borrowed, so one environment can back many games.
    pub fn create_game(
        env: &GameEnvironment,
        home: &TeamID,
        away: &TeamID,
    ) -> Result<Self, CreateGameError> {
        let home_roster = env
            .roster(home)
            .ok_or_else(|| CreateGameError::UnknownTeam(home.clone()))?;
        let away_roster = env
            .roster(away)
            .ok_or_else(|| CreateGameError::UnknownTeam(away.clone()))?;

        Ok(Self::build(
            home_roster.clone(),
            away_roster.clone(),
            env.new_deck(),
        ))
    }

    /// Pure dependency-injected constructor: builds a game from already-resolved rosters and
    /// an owned FAC deck. No disk access. This is the seam tests use to inject a deterministic
    /// deck (see `docs/design/testing-strategy.md` §5).
    fn build(home: Roster, away: Roster, fac_deck: FacManager) -> Self {
        let start_type = PlayType::Kickoff;
        let (event_tx, _rx) = broadcast::channel(GAME_EVENT_CHANNEL_CAPACITY);

        let game = Self {
            home,
            away,
            state: GameState::start_state(),
            past_plays: vec![],
            next_play: Some(start_type.create_impl()),
            offlineup: None,
            deflineup: None,
            fac_deck,
            event_tx,
        };

        // No subscribers exist yet at creation (see docs/plans/ws-events-stage2.md D2); this
        // is a deliberate no-op today but preserves the "every mutation emits" invariant.
        game.emit(GameEvent::GameStarted { state: game.state });

        game
    }

    /// Publish a domain event to all current subscribers. A send error means "no subscribers
    /// right now", which is normal and intentionally ignored (architecture §4).
    fn emit(&self, event: GameEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Obtain a receiver for this game's event stream. Each transport adapter (e.g. the WS
    /// handler) calls this to get its own independent receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<GameEvent> {
        self.event_tx.subscribe()
    }

    fn get_current_off_roster(&self) -> &Roster {
        match self.state.possession {
            GameTeams::Away => &self.away,
            GameTeams::Home => &self.home,
        }
    }

    fn get_current_def_roster(&self) -> &Roster {
        match self.state.possession {
            GameTeams::Away => &self.home,
            GameTeams::Home => &self.away,
        }
    }

    pub fn set_offensive_lineup_from_ids(
        &mut self,
        id_lineup: &OffenseIDLineup,
    ) -> Result<(), String> {
        let r = self.get_current_off_roster().clone();
        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_offense_lineup(id_lineup, &r)?;
        self.offlineup = Some(id_lineup.clone());
        self.emit(GameEvent::OffensiveLineupSet {
            lineup: id_lineup.clone(),
        });

        Ok(())
    }

    pub fn set_defensive_lineup_from_ids(
        &mut self,
        id_lineup: &DefenseIDLineup,
    ) -> Result<(), String> {
        let r = self.get_current_def_roster().clone();

        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_defense_lineup(id_lineup, &r)?;
        self.deflineup = Some(id_lineup.clone());
        self.emit(GameEvent::DefensiveLineupSet {
            lineup: id_lineup.clone(),
        });

        Ok(())
    }

    pub fn set_offense_call(&mut self, off_call: OffenseCall) -> Result<(), String> {
        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_offense_call(off_call)
    }

    pub fn set_defense_call(&mut self, def_call: DefenseCall) -> Result<(), String> {
        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_defense_call(def_call)
    }

    // unused: duplicate of the used `set_offensive_lineup_from_ids`; kept pending removal.
    #[allow(dead_code)]
    pub fn set_offense_lineup(&mut self, off_id: OffenseIDLineup) -> Result<(), String> {
        let r = self.get_current_off_roster().clone();
        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_offense_lineup(&off_id, &r)?;
        self.offlineup = Some(off_id);
        Ok(())
    }

    // unused: duplicate of the used `set_defensive_lineup_from_ids`; kept pending removal.
    #[allow(dead_code)]
    pub fn set_defense_lineup(&mut self, def_id: DefenseIDLineup) -> Result<(), String> {
        let r = self.get_current_def_roster().clone();

        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_defense_lineup(&def_id, &r)?;
        self.deflineup = Some(def_id);
        Ok(())
    }

    pub fn get_offensive_lineup_ids(&self) -> &Option<OffenseIDLineup> {
        &self.offlineup
    }

    pub fn get_defensive_lineup_ids(&self) -> &Option<DefenseIDLineup> {
        &self.deflineup
    }

    pub fn run_current_play(&mut self) -> Result<PlayAndState, String> {
        // Increment play counter in state

        let res = run_play(
            &self.state,
            &mut self.fac_deck,
            self.next_play.as_ref().ok_or("No Play Set")?,
        )?;

        self.past_plays.push(res.clone());

        // Update state, ensuring play counter is preserved
        self.state = GameState { ..res.new_state };
        self.set_next_play_type(self.state.get_next_move_default())?; // emits NextPlayTypeSet

        // Then announce the play itself. Net emission order for one play is
        // NextPlayTypeSet -> PlayRun (see docs/plans/ws-events-stage2.md D3).
        self.emit(GameEvent::PlayRun {
            play: Box::new(res.clone()),
        });

        return Ok(res);
    }

    // unused: abandoned stub (ignores its args and returns a fresh start_state); kept pending removal.
    #[allow(dead_code)]
    fn gen_new_state(
        _curr_state: &GameState,
        _play: &StandardPlay,
        _result: &PlayResult,
    ) -> GameState {
        return GameState::start_state();
    }

    pub fn allowed_play_types(&self) -> PlayTypeInfo {
        PlayTypeInfo {
            allowed_types: self.state.get_next_move_types(),
            next_type: self.next_play.as_ref().map(|play| play.get_type()),
        }
    }

    pub fn set_next_play_type(&mut self, playtype: PlayType) -> Result<(), String> {
        let allowed = self.state.get_next_move_types();
        if !allowed.contains(&playtype) {
            return Err(format!("Valid plays are {:?}", allowed));
        }
        let same_type = self.next_play.as_ref().unwrap().get_type() == playtype;
        self.next_play = Some(playtype.create_impl());
        if !same_type {
            self.offlineup = None;
            self.deflineup = None;
        }
        self.emit(GameEvent::NextPlayTypeSet {
            play_type: playtype,
        });
        Ok(())
    }

    fn write_json<T>(dir: &String, file: &str, obj: &T) -> std::io::Result<()>
    where
        T: Serialize,
    {
        let path = format!("{}/{}", dir, file);
        let file = File::create(path)?;

        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, obj)?;
        writer.flush()?;

        Ok(())
    }
    pub fn serialize_struct(&self, file_path: String) -> std::io::Result<()> {
        fs::create_dir(file_path.clone())?;
        Game::write_json(&file_path, "state.json", &self.state)?;
        Game::write_json(&file_path, "home.json", &self.home)?;
        Game::write_json(&file_path, "away.json", &self.away)?;
        Game::write_json(&file_path, "facs.json", &self.fac_deck)?;
        Ok(())
    }

    // unused: convenience accessor with no callers (see `get_all_plays`); kept pending removal.
    #[allow(dead_code)]
    pub fn get_last_play(&self) -> Option<&PlayAndState> {
        self.past_plays.last()
    }

    pub fn get_all_plays(&self) -> &Vec<PlayAndState> {
        &self.past_plays
    }

    // unused: convenience accessor with no callers; kept pending removal.
    #[allow(dead_code)]
    pub fn get_play_counter(&self) -> u32 {
        self.state.play_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spf_core::players::{Player, Roster, TeamID};

    fn empty_roster(name: &str) -> Roster {
        Roster::from_players(
            TeamID {
                name: name.into(),
                year: "1983".into(),
            },
            Vec::<Player>::new(),
        )
    }

    /// A game built from empty rosters and an empty (Z-only) injected deck. Needs no fixture
    /// files and no shuffle, so it is fully deterministic. The deck is irrelevant to the
    /// card-independent behaviors exercised here.
    fn game_with_injected_deck() -> Game {
        Game::build(
            empty_roster("Home"),
            empty_roster("Away"),
            fac::FacManager::from_cards(vec![]),
        )
    }

    #[test]
    fn test_set_next_play_type_emits_event() {
        // Arrange: a game with an injected deck (no disk, no shuffle, no self-skip).
        let mut game = game_with_injected_deck();

        // Subscribe *after* creation: the GameStarted emitted inside build had no
        // receiver and is already gone, so the channel is empty here. The next recv will be
        // whatever we emit below.
        let mut rx = game.subscribe();

        // Act: a deterministic, non-card state change. From the start state
        // (last_status == Start) Kickoff is the only legal next type.
        game.set_next_play_type(PlayType::Kickoff)
            .expect("Kickoff is a legal next play type from the start state");

        // Assert: the event arrived and carries the right PlayType (PlayType: PartialEq).
        match rx.try_recv() {
            Ok(GameEvent::NextPlayTypeSet { play_type }) => {
                assert_eq!(play_type, PlayType::Kickoff);
            }
            other => panic!("expected NextPlayTypeSet, got {:?}", other),
        }
    }

    fn team_id(name: &str) -> TeamID {
        TeamID {
            name: name.into(),
            year: "1983".into(),
        }
    }

    fn env_with_teams(names: &[&str]) -> environment::GameEnvironment {
        let rosters: Vec<Roster> = names.iter().map(|n| empty_roster(n)).collect();
        let league = spf_core::players::TeamList::from_rosters(rosters);
        environment::GameEnvironment::from_parts(league, fac::FacManager::from_cards(vec![]))
    }

    #[test]
    fn test_create_game_resolves_known_teams() {
        let env = env_with_teams(&["Home", "Away"]);
        let game = Game::create_game(&env, &team_id("Home"), &team_id("Away"))
            .expect("both teams are in the league");
        assert_eq!(game.home.get_team_name().name, "Home");
        assert_eq!(game.away.get_team_name().name, "Away");
    }

    #[test]
    fn test_create_game_unknown_home_team() {
        let env = env_with_teams(&["Away"]);
        match Game::create_game(&env, &team_id("Nope"), &team_id("Away")) {
            Err(CreateGameError::UnknownTeam(t)) => assert_eq!(t.name, "Nope"),
            Ok(_) => panic!("expected UnknownTeam(Nope), got Ok(game)"),
        }
    }

    #[test]
    fn test_create_game_unknown_away_team() {
        let env = env_with_teams(&["Home"]);
        match Game::create_game(&env, &team_id("Home"), &team_id("Nope")) {
            Err(CreateGameError::UnknownTeam(t)) => assert_eq!(t.name, "Nope"),
            Ok(_) => panic!("expected UnknownTeam(Nope), got Ok(game)"),
        }
    }
}
