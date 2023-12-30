use enum_as_inner::EnumAsInner;
use serde_derive::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::{
    game::{
        lineup::{StandardDefensiveLineup, StandardOffensiveLineup},
        players::Roster,
        GameState, Play,
    },
    validate_field,
};

use super::{
    engine::{defs::OFFENSIVE_PLAYS_LIST, CardStreamer},
    fac::{FacData, PassTarget, RunDirection},
    lineup::OffensiveBox,
    players::QBStats,
    stats::RangedStats,
    DefenseCall, DefenseIDLineup, OffenseCall, OffenseIDLineup, PlayImpl, PlayResult, PlayType,
};

type RunGetCardVal = for<'a> fn(card: &'a FacData) -> &'a RunDirection;
type PassGetPassVal = for<'a> fn(card: &'a FacData) -> &'a PassTarget;
type QBGetPassRange = for<'a> fn(qb: &'a QBStats) -> &'a RangedStats<PassResult>;

type PlayRunner = for<'a> fn(&'a GameState, PlaySetup<'a>, &'a mut CardStreamer<'a>) -> PlayResult;

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

pub trait Validatable {
    fn validate(&self, play: &StandardPlay) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy)]
pub struct RunMetaData {
    pub max_loss: i32,
    pub can_go_ob: bool,
    pub card_val: RunGetCardVal,
}

#[derive(Debug, Clone)]
pub struct PassMetaData {
    // max_loss: i32,
    // can_go_ob: bool,
    pub target: PassGetPassVal,
    pub completion_range: QBGetPassRange,
    pub pass_gain: String,
}

#[derive(Debug, Clone, EnumAsInner)]
pub enum OffensivePlayCategory {
    Run(RunMetaData),
    Pass(PassMetaData),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OffensiveStrategy {
    NoStrategy,
    Sneak,
    Flop,
    Draw,
    PlayAction,
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
    Straight,
    DoubleCover,
    TripleCover,
    DoubleCoverX2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardDefenseCall {
    pub defense_type: DefensivePlay,
    pub strategy: DefensiveStrategy,
    pub key: Option<OffensiveBox>,
    pub def_players: Vec<String>,
}
impl Validatable for StandardDefenseCall {
    fn validate(&self, play: &StandardPlay) -> Result<(), String> {
        let lineup = play.defense.as_ref().ok_or("Set lineup before Call")?;
        self.def_players
            .iter()
            .try_for_each(|id| match lineup.find_player(&id) {
                Some(_) => return Ok(()),
                None => return Err(format!("{} is not in lineup", id)),
            })?;

        if self.defense_type == DefensivePlay::Blitz
            && (self.def_players.len() < 2 || self.def_players.len() > 5)
        {
            return Err("Must blitz between 2 and 5 players".to_string());
        }

        return Ok(());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardOffenseCall {
    pub play_type: OffensivePlayType,
    pub strategy: OffensiveStrategy,
    pub target: OffensiveBox,
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

        let off: &StandardOffensiveLineup = play
            .offense
            .as_ref()
            .ok_or("Set Lineup before setting Call")?;
        off.get_player_in_pos(&self.target)
            .ok_or(format!("No player in {:?}", self.target))?;

        match self.strategy {
            OffensiveStrategy::Draw => {
                validate_strategy(
                    "Draw",
                    &self.play_type,
                    vec![OffensivePlayType::IL, OffensivePlayType::IR],
                )?;
            }
            OffensiveStrategy::PlayAction => {
                validate_strategy(
                    "PlayAction",
                    &self.play_type,
                    vec![OffensivePlayType::SH, OffensivePlayType::LG],
                )?;
            }
            OffensiveStrategy::NoStrategy => {}
            OffensiveStrategy::Sneak => {}
            OffensiveStrategy::Flop => {}
        }

        // use player for further validations
        return Ok(());
    }
}

fn validate_strategy(
    strategy: &str,
    actual: &OffensivePlayType,
    allowed: Vec<OffensivePlayType>,
) -> Result<(), String> {
    if !allowed.contains(actual) {
        return Err(format!("{:?} can not be played on {:?}", strategy, actual));
    }
    Ok(())
}

fn get_offensive_play_info(play: &OffensivePlayType) -> &OffensivePlayInfo {
    return &OFFENSIVE_PLAYS_LIST[play];
}

#[derive(Clone)]
pub struct PlaySetup<'a> {
    pub offense: &'a StandardOffensiveLineup,
    pub offense_call: &'a StandardOffenseCall,
    pub defense: StandardDefensiveLineup,
    pub defense_call: &'a StandardDefenseCall,
    pub offense_metadata: &'a OffensivePlayInfo,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct StandardPlay {
    pub offense: Option<StandardOffensiveLineup>,
    pub offense_call: Option<StandardOffenseCall>,
    pub defense: Option<StandardDefensiveLineup>,
    pub defense_call: Option<StandardDefenseCall>,
}

impl PlayImpl for StandardPlay {
    fn validate(&self) -> Result<(), String> {
        validate_field!(self.offense, "Offense not set");
        validate_field!(self.defense, "Defense not set");
        validate_field!(self.offense_call, "Offense Call not set");
        validate_field!(self.defense_call, "Defense Call not set");
        Ok(()) // offense.is_legal_lineup()?;
    }

    fn set_offense_call(&mut self, call: OffenseCall) -> Result<(), String> {
        println!("Offense Call {:?}", call);
        let c = call
            .as_standard_offense_call()
            .ok_or("Bad type".to_string())?;
        self.offense_call = Some(c.clone());
        Ok(())
    }

    fn set_defense_call(&mut self, call: DefenseCall) -> Result<(), String> {
        let c = call
            .as_standard_defense_call()
            .ok_or("Bad type".to_string())?;
        self.defense_call = Some(c.clone());
        Ok(())
    }

    fn set_offense_lineup(
        &mut self,
        lineup: &OffenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String> {
        let l = lineup
            .as_standard_id_offense_lineup()
            .ok_or("Bad type".to_string())?;

        self.offense = Some(StandardOffensiveLineup::create_lineup(l, roster)?);

        self.offense.as_ref().unwrap().is_legal_lineup()?;

        Ok(())
    }

    fn set_defense_lineup(
        &mut self,
        lineup: &DefenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String> {
        let l = lineup
            .as_standard_id_defense_lineup()
            .ok_or("Bad type".to_string())?;

        self.defense = Some(StandardDefensiveLineup::create_lineup(l, roster)?);

        self.defense.as_ref().unwrap().is_legal_lineup()?;

        Ok(())
    }

    fn run_play<'a>(
        &'a self,
        game_state: &'a GameState,
        card_streamer: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        let offense_metadata =
            get_offensive_play_info(&self.offense_call.as_ref().unwrap().play_type);

        let def_call = self.defense_call.as_ref().unwrap();
        let def_lineup = self.defense.as_ref().unwrap();

        let filtered_lineup = def_lineup.filter_players(&def_call.def_players);

        let real_def = if def_call.defense_type == DefensivePlay::Blitz {
            filtered_lineup
        } else {
            def_lineup.clone()
        };

        let details = PlaySetup {
            offense_metadata,
            offense: self.offense.as_ref().unwrap(),
            offense_call: self.offense_call.as_ref().unwrap(),
            defense: real_def,
            defense_call: def_call,
        };

        (details.offense_metadata.handler)(game_state, details, card_streamer)
    }

    fn get_play(&self) -> Play {
        Play::StandardPlay(self.clone())
    }

    fn get_type(&self) -> PlayType {
        return PlayType::Standard;
    }
}

impl StandardPlay {
    pub fn new() -> Self {
        return Self {
            ..Default::default()
        };
    }

    pub fn handle_z(result: &PlayResult) -> PlayResult {
        return result.clone();
    }
}
