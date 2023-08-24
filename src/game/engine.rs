use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use spf_macros::ToBasePlayer;

use lazy_static::lazy_static;

use super::{
    lineup::{DefensiveLineup, OffensiveLineup},
    GameState, PlayAndState,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Down {
    First,
    Second,
    Third,
    Fourth,
}
pub trait Validatable {
    fn validate(&self, play: &Play) -> Result<(), String>;
}

pub type Yard = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDOffensivePlay {
    pub play_code: String,
    pub strategy: String,
    pub target_id: String,
}

#[derive(Debug, Clone, Copy)]
pub enum OffensivePlayCategory {
    Run,
    Pass,
}

#[derive(Debug, Clone, Copy)]
pub struct OffensivePlayInfo {
    pub play_type: OffensivePlayCategory,
    pub name: &'static str,
    pub code: &'static str,
    pub handler: PlayRunner,
}

#[derive(Debug, Clone)]
pub enum OffensivePlayType {
    SL,
    SR,
    IL,
    IR,
    ER,
    QK,
    SH,
    LG,
    SC,
}

#[derive(Debug, Clone)]
pub enum OffensiveStrategy {
    Sneak,
    Flop,
    Draw,
    PlayAction,
}

#[derive(Debug, Clone)]
pub struct OffenseCall {
    play_type: OffensivePlayType,
    strategy: OffensiveStrategy,
    target: String,
}
impl Validatable for OffenseCall {
    fn validate(&self, play: &Play) -> Result<(), String> {
        return Ok(());
    }
}

type PlayRunner = fn(&PlaySetup, &GameState) -> PlayResult;

lazy_static! {
    static ref OFFENSIVE_PLAYS_LIST: HashMap<&'static str, OffensivePlayInfo> = {
        let mut map = HashMap::new();
        map.insert(
            "SL",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Sweep Left",
                code: "SL",
                handler: run_run_play,
            },
        );
        map.insert(
            "SR",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Sweep Right",
                code: "SR",
                handler: run_run_play,
            },
        );
        map.insert(
            "IL",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Inside Left",
                code: "IL",
                handler: run_run_play,
            },
        );
        map.insert(
            "IR",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Inside Right",
                code: "IR",
                handler: run_run_play,
            },
        );
        map.insert(
            "ER",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "End Around",
                code: "ER",
                handler: run_run_play,
            },
        );
        map.insert(
            "QK",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Quick",
                code: "QK",
                handler: run_pass_play,
            },
        );
        map.insert(
            "SH",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Short",
                code: "SH",
                handler: run_pass_play,
            },
        );
        map.insert(
            "LG",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Long",
                code: "LG",
                handler: run_pass_play,
            },
        );
        map.insert(
            "SC",
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Screen",
                code: "SC",
                handler: run_pass_play,
            },
        );
        map
    };
}

pub fn getOffensivePlayInfo(play: &OffensivePlayType) -> OffensivePlayInfo {
    return match play {
        OffensivePlayType::ER => OFFENSIVE_PLAYS_LIST["ER"],
        OffensivePlayType::SL => OFFENSIVE_PLAYS_LIST["SL"],
        OffensivePlayType::SR => OFFENSIVE_PLAYS_LIST["SR"],
        OffensivePlayType::IL => OFFENSIVE_PLAYS_LIST["IL"],
        OffensivePlayType::IR => OFFENSIVE_PLAYS_LIST["IR"],
        OffensivePlayType::QK => OFFENSIVE_PLAYS_LIST["QK"],
        OffensivePlayType::SH => OFFENSIVE_PLAYS_LIST["SH"],
        OffensivePlayType::LG => OFFENSIVE_PLAYS_LIST["LG"],
        OffensivePlayType::SC => OFFENSIVE_PLAYS_LIST["SC"],
    };
}

#[derive(Debug, Clone, Copy)]
pub enum DefensivePlay {
    RunDefense,
    PassDefense,
    PreventDefense,
    Blitz,
}

#[derive(Debug, Clone)]
pub enum DefensiveStrategy {
    DoubleCover,
    TripleCover,
    DoubleCoverX2,
}

#[derive(Debug, Clone)]
pub struct DefenseCall {
    play: DefensivePlay,
    strategy: DefensiveStrategy,
}
impl Validatable for DefenseCall {
    fn validate(&self, play: &Play) -> Result<(), String> {
        return Ok(());
    }
}

pub struct PlaySetup<'a> {
    pub offense: &'a OffensiveLineup,
    pub offense_call: &'a OffenseCall,
    pub defense: &'a DefensiveLineup,
    pub defense_call: &'a DefenseCall,
}

#[derive(Debug, Default, Clone)]
pub struct Play {
    pub offense: Option<OffensiveLineup>,
    pub offense_call: Option<OffenseCall>,
    pub defense: Option<DefensiveLineup>,
    pub defense_call: Option<DefenseCall>,
}

impl Play {
    pub fn new() -> Self {
        return Self {
            ..Default::default()
        };
    }

    fn play_ready(&self) -> Result<PlaySetup, String> {
        let offense = self.offense.as_ref().ok_or("Offense not set")?;
        offense.is_legal_lineup()?;

        let defense = self.defense.as_ref().ok_or("Defense not set")?;
        defense.is_legal_lineup()?;

        let offense_call = self.offense_call.as_ref().ok_or("No offense play")?;
        offense_call.validate(self)?;

        let defense_call = self.defense_call.as_ref().ok_or("No defense play")?;
        defense_call.validate(self)?;

        return Ok(PlaySetup {
            offense,
            defense,
            offense_call,
            defense_call,
        });
    }

    pub fn run_play(&self, curr_state: &GameState) -> Result<PlayAndState, String> {
        let details = self.play_ready()?;

        let info = getOffensivePlayInfo(&details.offense_call.play_type);
        let result = (info.handler)(&details, curr_state);        

        let new_state = GameState {
            down: Down::Second,
            yardline: 19,
            ..curr_state.clone()
        };

        return Ok(PlayAndState {
            play: self.clone(),
            result,
            new_state,
        });
    }
}

fn run_run_play(details: &PlaySetup, curr_state: &GameState) -> PlayResult {




    return PlayResult { result:10, time: 10 }
}
fn run_pass_play(details: &PlaySetup, curr_state: &GameState) -> PlayResult {

    return PlayResult { result:10, time: 10 }
}

#[derive(Debug, Copy, Clone)]
pub struct PlayResult {
    pub result: Yard,
    pub time: i32,
}
