pub mod defs;
mod kickplay;
pub mod passplay;
mod playutils;
mod resulthandler;
pub mod runplay;

use std::{collections::HashMap, hash::Hash};

use enum_as_inner::EnumAsInner;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use strum_macros::EnumString;

use crate::game::{
    engine::{passplay::PassUtils, runplay::RunUtils},
    lineup::DefensiveBox,
};

use self::{
    defs::OFFENSIVE_PLAYS_LIST, kickplay::KickPlayContext, resulthandler::calculate_play_result,
};

use super::{
    fac::{FacCard, FacData, FacManager, PassTarget, RunDirection},
    lineup::{DefensiveLineup, OffensiveBox, OffensiveLineup},
    players::{KRStats, KStats, QBStats},
    stats::{LabeledStat, RangedStats, TwelveStats},
    GameState, Play, PlayAndState,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Down {
    First,
    Second,
    Third,
    Fourth,
}

impl Down {
    fn next_down(&self) -> Down {
        match self {
            Down::First => Down::Second,
            Down::Second => Down::Third,
            Down::Third => Down::Fourth,
            Down::Fourth => Down::Fourth,
        }
    }
}

pub trait Validatable {
    fn validate(&self, play: &StandardPlay) -> Result<(), String>;
}

pub type Yard = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDOffensivePlay {
    pub play_code: String,
    pub strategy: String,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq)]
pub enum PlayType {
    Kickoff,
    Punt,
    ExtraPoint,
    FieldGoal,
    OffensePlay,
}



#[derive(Debug, Clone, EnumAsInner)]
pub enum OffensivePlayCategory {
    Run(RunMetaData),
    Pass(PassMetaData),
}

type RunGetCardVal = for<'a> fn(card: &'a FacData) -> &'a RunDirection;
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

#[derive(Debug, Clone)]
pub struct PassMetaData {
    // max_loss: i32,
    // can_go_ob: bool,
    target: PassGetPassVal,
    completion_range: QBGetPassRange,
    pass_gain: String,
}

pub trait Shiftable<T> {
    fn get_first() -> T;
    fn get_second() -> T;
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

impl Shiftable<PassResult> for PassResult {
    fn get_first() -> PassResult {
        PassResult::Complete
    }

    fn get_second() -> PassResult {
        PassResult::Incomplete
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, EnumString)]
pub enum PassRushResult {
    #[strum(serialize = "Sack")]
    Sack,
    #[strum(serialize = "Runs")]
    Runs,
    #[strum(serialize = "Com")]
    Complete,
    #[strum(serialize = "Inc")]
    Incomplete,
}

impl Shiftable<PassRushResult> for PassRushResult {
    fn get_first() -> PassRushResult {
        PassRushResult::Sack
    }

    fn get_second() -> PassRushResult {
        PassRushResult::Runs
    }
}

#[derive(Debug, Clone)]
pub struct OffensivePlayInfo {
    play_type: OffensivePlayCategory,
    name: &'static str,
    code: &'static str,
    allowed_targets: Vec<OffensiveBox>,
    handler: PlayRunner,
}

// #[derive(Debug, Clone)]
// pub struct DefensivePlayInfo {
//     completion_impact: Map<String, i32>,
//     in20_completion_impact: Map<String, i32>,

//     qk: i32,
//     sh: i32,
//     lg: i32,

//     allowed_targets: Vec<OffensiveBox>,
//     handler: PlayRunner,
// }

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
pub struct StandardOffenseCall {
    play_type: OffensivePlayType,
    strategy: Option<OffensiveStrategy>,
    target: OffensiveBox,
}

impl Validatable for StandardOffenseCall {
    fn validate(&self, play: &StandardPlay) -> Result<(), String> {
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
pub struct StandardDefenseCall {
    defense_type: DefensivePlay,
    strategy: Option<DefensiveStrategy>,
    key: Option<OffensiveBox>,
    def_players: Vec<String>,
}
impl Validatable for StandardDefenseCall {
    fn validate(&self, play: &StandardPlay) -> Result<(), String> {
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
    pub offense_call: &'a StandardOffenseCall,
    pub defense: &'a DefensiveLineup,
    pub defense_call: &'a StandardDefenseCall,
    pub offense_metadata: &'a OffensivePlayInfo,
}

#[derive(Debug, Default, Clone)]
pub struct StandardPlay {
    pub offense: Option<OffensiveLineup>,
    pub offense_call: Option<StandardOffenseCall>,
    pub defense: Option<DefensiveLineup>,
    pub defense_call: Option<StandardDefenseCall>,
}

impl StandardPlay {
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
            StandardPlay::handle_z(&result);
        }

        let new_state = calculate_play_result(game_state, &result);

        return Ok(PlayAndState {
            play: Play::StandardPlay(self.clone()),
            result,
            new_state,
        });
    }

    fn handle_z(result: &PlayResult) -> PlayResult {
        return result.clone();
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ResultType {
    Regular,
    TurnOver,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayResult {
    pub result_type: ResultType,
    pub result: Yard,
    pub final_line: Yard,
    pub time: i32,
    pub details: Vec<String>,
    pub mechanic: Vec<String>,

    pub extra: Option<String>,
    pub cards: CardResults,
}

// pub trait PlayLogicState {
//     fn handle_card(
//         &self,
//         state: &GameState,
//         play: &PlaySetup,
//         card: &FacData,
//     ) -> Box<dyn PlayLogicState>;
//     fn get_result(&self) -> Option<OffensePlayResult> {
//         None
//     }
//     fn get_name(&self) -> &str;
// }

#[derive(Debug, Clone)]
pub enum OffenseCall {
    KickoffOffenseCall(KickoffOffenseCall),
    PuntOffenseCall(PuntOffenseCall),
}
impl<'de> Deserialize<'de> for OffenseCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TaggedOffenseCall {
            KickoffOffenseCall(KickoffOffenseCall),
            PuntOffenseCall(PuntOffenseCall),
        }

        let tagged_call = TaggedOffenseCall::deserialize(deserializer)?;

        Ok(match tagged_call {
            TaggedOffenseCall::KickoffOffenseCall(kc) => OffenseCall::KickoffOffenseCall(kc),
            TaggedOffenseCall::PuntOffenseCall(pc) => OffenseCall::PuntOffenseCall(pc),
        })
    }
}
impl OffenseCall {
    pub fn get_play_type(&self) -> PlayType {
        match self {
            OffenseCall::KickoffOffenseCall(_) => PlayType::Kickoff,
            OffenseCall::PuntOffenseCall(_) => PlayType::Punt,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KickoffOffenseCall {
    pub onside: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PuntOffenseCall {
    pub coffin_corner: i32,
}

#[derive(Debug, Clone)]
pub struct KickoffPlay {
    pub onside: bool,
    pub kr: KRStats,
}

impl KickoffPlay {
    pub fn runplay(&self, game_state: &GameState, fac_deck: &mut FacManager) -> PlayResult {
        let mut card_streamer = CardStreamer::new(fac_deck);

        return KickPlayContext::run_play(game_state, self.clone(), &mut card_streamer);
    }
}
