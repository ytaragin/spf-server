use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use spf_macros::ToBasePlayer;

use lazy_static::lazy_static;

use super::{
    fac::FacManager,
    lineup::{DefensiveLineup, OffensiveBox, OffensiveLineup},
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffensivePlayCategory {
    Run,
    Pass,
}

#[derive(Debug, Clone)]
pub struct OffensivePlayInfo {
    pub play_type: OffensivePlayCategory,
    pub name: &'static str,
    pub code: &'static str,
    pub allowed_targets: Vec<OffensiveBox>,
    pub handler: PlayRunner,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
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

// impl OffensivePlayType {
//     fn validate(&self, lineup: &OffensiveLineup) -> Result<(), String> {
//         match self {
//             OffensivePlayType::SL |
//             OffensivePlayType::SR
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OffensiveStrategy {
    Sneak,
    Flop,
    Draw,
    PlayAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffenseCall {
    play_type: OffensivePlayType,
    strategy: OffensiveStrategy,
    target: OffensiveBox,
}

impl Validatable for OffenseCall {
    fn validate(&self, play: &Play) -> Result<(), String> {
        let meta = get_offensive_play_info(&self.play_type);
        if !meta.allowed_targets.contains(&self.target) {
            return Err(format!(
                "{:?} is not a valid target for {:?}",
                self.target, self.play_type
            ));
        }

        let off: &OffensiveLineup = play.offense.as_ref().unwrap();
        let player = off
            .get_player_in_pos(&self.target)
            .ok_or(format!("No player in {:?}", self.target))?;

        // use player for further validations
        return Ok(());
    }
}

type PlayRunner = fn(&PlaySetup, &GameState) -> PlayResult;
type OffenseCallValidator = fn(&OffenseCall, &OffensiveLineup) -> Result<(), String>;

lazy_static! {
    static ref OFFENSIVE_PLAYS_LIST: HashMap<OffensivePlayType, OffensivePlayInfo> = {
        let mut map = HashMap::new();
        map.insert(
            OffensivePlayType::SL,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Sweep Left",
                code: "SL",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: run_run_play,
            },
        );
        map.insert(
            OffensivePlayType::SR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Sweep Right",
                code: "SR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: run_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IL,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Inside Left",
                code: "IL",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: run_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Inside Right",
                code: "IR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: run_run_play,
            },
        );
        map.insert(
            OffensivePlayType::ER,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "End Around",
                code: "ER",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: run_run_play,
            },
        );
        map.insert(
            OffensivePlayType::QK,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Quick",
                code: "QK",
                allowed_targets: vec![
                    OffensiveBox::B1,
                    OffensiveBox::B2,
                    OffensiveBox::B3,
                    OffensiveBox::RE,
                    OffensiveBox::LE,
                    OffensiveBox::FL1,
                    OffensiveBox::FL2,
                ],
                handler: run_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::SH,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Short",
                code: "SH",
                allowed_targets: vec![
                    OffensiveBox::B1,
                    OffensiveBox::B2,
                    OffensiveBox::B3,
                    OffensiveBox::RE,
                    OffensiveBox::LE,
                    OffensiveBox::FL1,
                    OffensiveBox::FL2,
                ],
                handler: run_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::LG,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Long",
                code: "LG",
                allowed_targets: vec![
                    OffensiveBox::B1,
                    OffensiveBox::B2,
                    OffensiveBox::B3,
                    OffensiveBox::RE,
                    OffensiveBox::LE,
                    OffensiveBox::FL1,
                    OffensiveBox::FL2,
                ],
                handler: run_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::SC,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Screen",
                code: "SC",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: run_pass_play,
            },
        );
        map
    };
}

pub fn get_offensive_play_info(play: &OffensivePlayType) -> &OffensivePlayInfo {
    return &OFFENSIVE_PLAYS_LIST[play];
    // return match play {
    //     OffensivePlayType::ER => OFFENSIVE_PLAYS_LIST["ER"],
    //     OffensivePlayType::SL => OFFENSIVE_PLAYS_LIST["SL"],
    //     OffensivePlayType::SR => OFFENSIVE_PLAYS_LIST["SR"],
    //     OffensivePlayType::IL => OFFENSIVE_PLAYS_LIST["IL"],
    //     OffensivePlayType::IR => OFFENSIVE_PLAYS_LIST["IR"],
    //     OffensivePlayType::QK => OFFENSIVE_PLAYS_LIST["QK"],
    //     OffensivePlayType::SH => OFFENSIVE_PLAYS_LIST["SH"],
    //     OffensivePlayType::LG => OFFENSIVE_PLAYS_LIST["LG"],
    //     OffensivePlayType::SC => OFFENSIVE_PLAYS_LIST["SC"],
    // };
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
    targets: Vec<String>,
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

    pub fn run_play(
        &self,
        curr_state: &GameState,
        fac_deck: &mut FacManager,
    ) -> Result<PlayAndState, String> {
        let details = self.play_ready()?;

        let info = get_offensive_play_info(&details.offense_call.play_type);
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
    return PlayResult {
        result: 10,
        time: 10,
    };
}
fn run_pass_play(details: &PlaySetup, curr_state: &GameState) -> PlayResult {
    return PlayResult {
        result: 10,
        time: 10,
    };
}

#[derive(Debug, Copy, Clone)]
pub struct PlayResult {
    pub result: Yard,
    pub time: i32,
}
