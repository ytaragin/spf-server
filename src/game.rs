pub mod engine;
pub mod fac;
pub mod lineup;
pub mod loader;
pub mod players;
pub mod stats;

use serde::{Deserialize, Serialize};

use self::{
    engine::{
        defs::GAMECONSTANTS, run_play, DefenseCall, DefenseIDLineup, Down, KickoffPlay,
        OffenseCall, OffenseIDLineup, PlayImpl, PlayResult, PlayType, StandardDefenseCall,
        StandardOffenseCall, StandardPlay, Validatable, Yard,
    },
    fac::FacManager,
    lineup::{
        StandardDefensiveLineup, StandardIDDefenseLineup, StandardIDOffenseLineup,
        StandardOffensiveLineup,
    },
    players::Roster,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GamePlayStatus {
    Touchdown,
    Safety,
    FieldGoal,
    PossesionChange,
    Ongoing,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameStatus {
    Kickoff,
    StandardPlay,
    ExtraPoint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameState {
    pub last_status: GamePlayStatus,
    pub quarter: i32,
    pub time_remaining: i32,
    pub possesion: GameTeams,
    pub down: Down,
    pub yardline: Yard,
    pub first_down_target: Yard,
    pub home_score: i32,
    pub away_score: i32,
}

impl GameState {
    pub fn start_state() -> Self {
        return Self {
            last_status: GamePlayStatus::Start,
            quarter: 1,
            time_remaining: GAMECONSTANTS.sec_per_quarter,
            possesion: GameTeams::Away,
            down: Down::First,
            yardline: 50,
            first_down_target: 60,
            home_score: 0,
            away_score: 0,
        };
    }

    pub fn get_next_move_types(&self) -> Vec<PlayType> {
        match self.last_status {
            GamePlayStatus::Touchdown => vec![PlayType::ExtraPoint],
            GamePlayStatus::Safety => vec![PlayType::Punt],
            GamePlayStatus::FieldGoal => vec![PlayType::Kickoff],
            GamePlayStatus::PossesionChange | GamePlayStatus::Ongoing => {
                vec![PlayType::Standard, PlayType::Punt, PlayType::FieldGoal]
            }
            GamePlayStatus::Start => vec![PlayType::Kickoff],
            GamePlayStatus::End => vec![],
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct PlayAndState {
    pub play: Play,
    pub result: PlayResult,
    pub new_state: GameState,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayTypeInfo {
    pub allowed_types: Vec<PlayType>,
    pub next_type: Option<PlayType>,
}

// #[derive(Debug, Clone)]
pub struct Game {
    pub home: Roster,
    pub away: Roster,
    pub state: GameState,
    pub past_plays: Vec<PlayAndState>,
    pub next_play: Option<Box<dyn PlayImpl + Send>>,
    offlineup: Option<OffenseIDLineup>,
    defflineup: Option<DefenseIDLineup>,

    // pub current_play: StandardPlayOrig,
    pub fac_deck: FacManager,
}

impl Game {
    pub fn create_game(home: Roster, away: Roster) -> Self {
        let start_type = PlayType::Kickoff;

        return Self {
            home,
            away,
            state: GameState::start_state(),
            past_plays: vec![],
            // current_play: StandardPlayOrig::new(),
            next_play: Some(start_type.create_impl()),
            offlineup: None,
            defflineup: None,
            fac_deck: FacManager::new("cards/fac_cards.csv"),
        };
    }

    fn get_current_off_roster(&self) -> &Roster {
        match self.state.possesion {
            GameTeams::Away => &self.away,
            GameTeams::Home => &self.home,
        }
    }

    fn get_current_def_roster(&self) -> &Roster {
        match self.state.possesion {
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
            .set_offense_lineup(id_lineup, &r)
    }

    pub fn set_defensive_lineup_from_ids(
        &mut self,
        id_lineup: &DefenseIDLineup,
    ) -> Result<(), String> {
        let r = self.get_current_def_roster().clone();

        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_defense_lineup(id_lineup, &r)
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

    pub fn set_offense_lineup(&mut self, off_id: OffenseIDLineup) -> Result<(), String> {
        let r = self.get_current_off_roster().clone();
        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_offense_lineup(&off_id, &r)?;
        self.offlineup = Some(off_id);
        Ok(())
    }

    pub fn set_defense_lineup(&mut self, def_id: DefenseIDLineup) -> Result<(), String> {
        let r = self.get_current_def_roster().clone();

        self.next_play
            .as_mut()
            .ok_or("No Play Set")?
            .set_defense_lineup(&def_id, &r)?;
        self.defflineup = Some(def_id);
        Ok(())
    }

    pub fn get_offensive_lineup_ids(&self) -> &Option<OffenseIDLineup> {
        &self.offlineup
    }

    pub fn get_defensive_lineup_ids(&self) -> &Option<DefenseIDLineup> {
        &self.defflineup
    }

    pub fn run_play(&mut self) -> Result<PlayAndState, String> {
        let res = run_play(
            &self.state,
            &mut self.fac_deck,
            self.next_play.as_ref().ok_or("No Play Set")?,
        )?;

        self.past_plays.push(res.clone());

        self.state = res.new_state;
        self.next_play = None;

        return Ok(res);
    }

    fn gen_new_state(
        curr_state: &GameState,
        play: &StandardPlay,
        result: &PlayResult,
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
        self.next_play = Some(playtype.create_impl());
        self.offlineup = None;
        self.defflineup = None;
        Ok(())
    }
}
