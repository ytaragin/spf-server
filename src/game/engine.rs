pub mod passplay;
pub mod runplay;

use std::collections::HashMap;

use enum_as_inner::EnumAsInner;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use spf_macros::ToBasePlayer;

use lazy_static::lazy_static;
use strum_macros::EnumString;

use crate::game::{
    engine::{passplay::PassUtils, runplay::RunUtils},
    fac,
};

use super::{
    fac::{FacCard, FacData, FacManager, PassTarget, RunResult, RunResultActual},
    lineup::{DefensiveBox, DefensiveLineup, OffensiveBox, OffensiveLineup},
    players::{BasePlayer, Player, QBStats},
    stats::RangedStats,
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

#[derive(Debug, Clone, Copy, EnumAsInner)]
pub enum OffensivePlayCategory {
    Run(RunMetaData),
    Pass(PassMetaData),
}

type RunGetCardVal = for<'a> fn(card: &'a FacData) -> &'a RunResult;
type PassGetPassVal = for<'a> fn(card: &'a FacData) -> &'a PassTarget;
type QBGetPassRange = for<'a> fn(qb: &'a QBStats) -> &'a RangedStats<PassResult>;

type PlayRunner =
    for<'a> fn(&'a GameState, &'a PlaySetup<'a>, &'a mut CardStreamer<'a>) -> PlayResult;

#[derive(Debug, Clone, Copy)]
pub struct RunMetaData {
    max_loss: i32,
    can_go_ob: bool,
    card_val: RunGetCardVal,
}

#[derive(Debug, Clone, Copy)]
pub struct PassMetaData {
    // max_loss: i32,
    // can_go_ob: bool,
    target: PassGetPassVal,
    complete: QBGetPassRange,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, EnumString)]
pub enum PassResult {
    #[strum(serialize = "Com")]
    Complete,
    #[strum(serialize = "Inc")]
    Incomplete,
    #[strum(serialize = "Int")]
    Interception,
}

#[derive(Debug, Clone)]
pub struct OffensivePlayInfo {
    play_type: OffensivePlayCategory,
    name: &'static str,
    code: &'static str,
    allowed_targets: Vec<OffensiveBox>,
    handler: PlayRunner,
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

struct TimeTable {
    run_play: i32,
    run_play_ob: i32,
    pass_play_complete: i32,
    pass_play_incomplete: i32,
}

lazy_static! {
    static ref TIMES: TimeTable = TimeTable {
        run_play: 40,
        run_play_ob: 10,
        pass_play_complete: 40,
        pass_play_incomplete: 10,
    };
    static ref OFFENSIVE_PLAYS_LIST: HashMap<OffensivePlayType, OffensivePlayInfo> = {
        let mut map = HashMap::new();
        map.insert(
            OffensivePlayType::SL,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -100,
                    can_go_ob: true,
                    card_val: RunUtils::get_sl_fac_result,
                }),
                name: "Sweep Left",
                code: "SL",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::SR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -100,
                    can_go_ob: true,
                    card_val: RunUtils::get_sr_fac_result,
                }),
                name: "Sweep Right",
                code: "SR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IL,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -3,
                    can_go_ob: false,
                    card_val: RunUtils::get_il_fac_result,
                }),
                name: "Inside Left",
                code: "IL",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -3,
                    can_go_ob: false,
                    card_val: RunUtils::get_ir_fac_result,
                }),
                name: "Inside Right",
                code: "IR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::ER,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -3,
                    can_go_ob: false,
                    card_val: RunUtils::get_ir_fac_result,
                }),
                name: "End Around",
                code: "ER",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::QK,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_qk_fac_target,
                    complete: PassUtils::get_qk_qb_range,
                }),
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
                handler: PassUtils::handle_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::SH,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_sh_fac_target,
                    complete: PassUtils::get_sh_qb_range,
                }),
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
                handler: PassUtils::handle_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::LG,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_lg_fac_target,
                    complete: PassUtils::get_lg_qb_range,
                }),
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
                handler: PassUtils::handle_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::SC,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_qk_fac_target,
                    complete: PassUtils::get_qk_qb_range,
                }),
                name: "Screen",
                code: "SC",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: PassUtils::handle_pass_play,
            },
        );
        map
    };
}

fn get_offensive_play_info(play: &OffensivePlayType) -> &OffensivePlayInfo {
    return &OFFENSIVE_PLAYS_LIST[play];
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize)]
pub struct CardResults {
    had_z: bool,
    cards_flipped: Vec<i32>,
}

pub struct CardStreamer<'a> {
    fac_deck: &'a mut FacManager,
    cards_flipped: Vec<i32>,
    had_z: bool,
}

impl<'a> CardStreamer<'a> {
    fn new(fac_deck: &'a mut FacManager) -> Self {
        return Self {
            fac_deck,
            cards_flipped: vec![],
            had_z: false,
        };
    }

    fn get_fac(&mut self) -> FacData {
        let mut ret_data: Option<FacData> = None;
        while ret_data.is_none() {
            let card = self.fac_deck.get_fac(false);
            match card {
                FacCard::Z => {
                    if self.cards_flipped.len() <= 3 {
                        println!("Z Event");
                        self.had_z = true;
                    }
                }
                FacCard::Data(c) => {
                    self.cards_flipped.push(c.id);

                    ret_data = Some(c);
                }
            };
        }

        return ret_data.unwrap();
    }

    fn get_results(&self) -> CardResults {
        CardResults {
            had_z: self.had_z,
            cards_flipped: self.cards_flipped.clone(),
        }
    }
}

pub struct PlaySetup<'a> {
    pub offense: &'a OffensiveLineup,
    pub offense_call: &'a OffenseCall,
    pub defense: &'a DefensiveLineup,
    pub defense_call: &'a DefenseCall,
    pub offense_metadata: &'a OffensivePlayInfo,
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

        let offense_metadata = get_offensive_play_info(&offense_call.play_type);

        return Ok(PlaySetup {
            offense,
            defense,
            offense_call,
            defense_call,
            offense_metadata,
        });
    }

    pub fn run_play(
        &self,
        game_state: &GameState,
        fac_deck: &mut FacManager,
    ) -> Result<PlayAndState, String> {
        let details = self.play_ready()?;

        let mut card_streamer = CardStreamer::new(fac_deck);

        let result = (details.offense_metadata.handler)(game_state, &details, &mut card_streamer);

        if result.cards.had_z {
            Play::handle_z(&result);
        }

        let new_state = Play::create_new_state(game_state, &result);

        return Ok(PlayAndState {
            play: self.clone(),
            result,
            new_state,
        });
    }

    fn handle_z(result: &PlayResult) -> PlayResult {
        return result.clone();
    }

    fn create_new_state(old_state: &GameState, result: &PlayResult) -> GameState {
        GameState {
            down: Down::Second,
            yardline: 19,
            ..old_state.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayResult {
    pub result: Yard,
    pub time: i32,
    pub details: Vec<String>,
    pub extra: Option<String>,
    pub cards: CardResults,
}

pub trait PlayLogicState {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState>;
    fn get_result(&self) -> Option<PlayResult> {
        None
    }
    fn get_name(&self) -> &str;
}
