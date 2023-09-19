pub mod runplay;

use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use spf_macros::ToBasePlayer;

use lazy_static::lazy_static;

use crate::game::engine::runplay::RunUtils;

use super::{
    fac::{FacCard, FacData, FacManager, RunResult, RunResultActual},
    lineup::{DefensiveBox, DefensiveLineup, OffensiveBox, OffensiveLineup},
    players::{BasePlayer, Player},
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

type CreateStartState = fn() -> Box<dyn PlayLogicState>;

#[derive(Debug, Clone)]
struct OffensivePlayInfo {
    play_type: OffensivePlayCategory,
    name: &'static str,
    code: &'static str,
    allowed_targets: Vec<OffensiveBox>,
    handler: CreateStartState,
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
    strategy: Option<OffensiveStrategy>,
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

        let off: &OffensiveLineup = play
            .offense
            .as_ref()
            .ok_or("Set Lineup before setting Call")?;
        let player = off
            .get_player_in_pos(&self.target)
            .ok_or(format!("No player in {:?}", self.target))?;

        // use player for further validations
        return Ok(());
    }
}

// type PlayRunner = fn(&PlaySetup, &GameState, &mut FacManager) -> PlayResult;

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
                handler: RunUtils::create_run_play,
            },
        );
        map.insert(
            OffensivePlayType::SR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Sweep Right",
                code: "SR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::create_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IL,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Inside Left",
                code: "IL",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::create_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "Inside Right",
                code: "IR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::create_run_play,
            },
        );
        map.insert(
            OffensivePlayType::ER,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run,
                name: "End Around",
                code: "ER",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::create_run_play,
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
                handler: create_pass_play,
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
                handler: create_pass_play,
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
                handler: create_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::SC,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass,
                name: "Screen",
                code: "SC",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: create_pass_play,
            },
        );
        map
    };
}

fn get_offensive_play_info(play: &OffensivePlayType) -> &OffensivePlayInfo {
    return &OFFENSIVE_PLAYS_LIST[play];
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DefensivePlay {
    RunDefense,
    PassDefense,
    PreventDefense,
    Blitz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefensiveStrategy {
    DoubleCover,
    TripleCover,
    DoubleCoverX2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseCall {
    defense_type: DefensivePlay,
    strategy: Option<DefensiveStrategy>,
    key: Option<OffensiveBox>,
    def_players: Vec<String>,
}
impl Validatable for DefenseCall {
    fn validate(&self, play: &Play) -> Result<(), String> {
        let lineup = play.defense.as_ref().ok_or("Set lineup before Call")?;
        let res = self
            .def_players
            .iter()
            .try_for_each(|id| match lineup.find_player(&id) {
                Some(_) => return Ok(()),
                None => return Err(format!("{} is not in lineup", id)),
            });
        return res;
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
        game_state: &GameState,
        fac_deck: &mut FacManager,
    ) -> Result<PlayAndState, String> {
        let details = self.play_ready()?;

        let info = get_offensive_play_info(&details.offense_call.play_type);

        let mut play_state = (info.handler)();

        let mut cards_flipped = 0;
        let mut had_a_z = false;

        while play_state.get_result().is_none() {
            let card = fac_deck.get_fac(false);
            cards_flipped += 1;
            match card {
                FacCard::Z => {
                    if cards_flipped < 3 {
                        had_a_z = true;
                    }
                }
                FacCard::Data(c) => {
                    play_state = play_state.handle_card(game_state, &details, &c);
                }
            };
        }

        let mut result = play_state.get_result().unwrap();

        if had_a_z {
            Play::handle_z(&mut result);
        }

        let new_state = Play::create_new_state(game_state, &result);

        return Ok(PlayAndState {
            play: self.clone(),
            result,
            new_state,
        });
    }

    fn handle_z(result: &mut PlayResult) {}

    fn create_new_state(old_state: &GameState, result: &PlayResult) -> GameState {
        GameState {
            down: Down::Second,
            yardline: 19,
            ..old_state.clone()
        }
    }
}

fn create_pass_play() -> Box<dyn PlayLogicState> {
    let data = PassPlayData { details: vec![] };
    // return Box::new(p);
    return Box::new(PassStateStart { data });
}

#[derive(Clone)]
pub struct PassPlayData {
    details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlayResult {
    pub result: Yard,
    pub time: i32,
    pub details: Vec<String>,
    pub extra: Option<String>,
}

trait PlayLogicState {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState>;
    fn get_result(&self) -> Option<PlayResult> {
        None
    }
}

struct PassStateStart {
    data: PassPlayData,
}

impl PlayLogicState for PassStateStart {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        return Box::new(PassStateStart {
            data: self.data.clone(),
        });
    }
}
