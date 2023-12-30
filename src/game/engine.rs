pub mod defs;
mod kickplay;
pub mod passplay;
mod playutils;
mod resulthandler;
pub mod runplay;

use std::{collections::HashMap, hash::Hash};

use enum_as_inner::EnumAsInner;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use spf_macros::CustomDeserialize;
use strum_macros::EnumString;

use crate::game::{
    engine::{passplay::PassUtils, runplay::RunUtils},
    lineup::{DefensiveBox, KickoffIDDefenseLineup},
};

use self::{
    defs::OFFENSIVE_PLAYS_LIST, kickplay::KickPlayImpl, resulthandler::calculate_play_result,
};

use super::{
    fac::{FacCard, FacData, FacManager, PassTarget, RunDirection},
    lineup::{
        KickoffIDOffenseLineup, OffensiveBox, StandardDefensiveLineup, StandardIDDefenseLineup,
        StandardIDOffenseLineup, StandardOffensiveLineup,
    },
    players::{KRStats, KStats, Player, QBStats, Roster},
    standard_play::{self, StandardDefenseCall, StandardOffenseCall, StandardPlay},
    stats::{LabeledStat, RangedStats, TwelveStats},
    GameState, Play, PlayAndState,
};

macro_rules! impl_deserialize {
    ($enum_name:ident { $( $variant:ident ( $type:ty ) ),+ }) => {
        impl<'de> Deserialize<'de> for $enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>
            {
                #[derive(Deserialize)]
                #[serde(untagged)]
                enum Tagged {
                   $( $variant ($type) ),+
                }

                let tagged = Tagged::deserialize(deserializer)?;

                Ok(match tagged {
                   $(Tagged::$variant(v) => $enum_name::$variant(v)),+
                })
           }
        }
    };
}

#[macro_export]
macro_rules! validate_field {
    ($field:expr, $name:expr ) => {
        if $field.is_none() {
            return Err(format!("{} not set", $name));
        }

    };
}

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

pub trait PlayImpl: Send {
    fn validate(&self) -> Result<(), String>;
    fn set_offense_call(&mut self, call: OffenseCall) -> Result<(), String>;
    fn set_defense_call(&mut self, call: DefenseCall) -> Result<(), String>;
    fn set_offense_lineup(
        &mut self,
        lineup: &OffenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String>;
    fn set_defense_lineup(
        &mut self,
        lineup: &DefenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String>;
    fn run_play<'a>(
        &'a self,
        game_state: &'a GameState,
        card_streamer: &'a mut CardStreamer<'a>,
    ) -> PlayResult;
    fn get_play(&self) -> Play;
    fn get_type(&self) -> PlayType;
}

pub type Yard = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDOffensivePlay {
    pub play_code: String,
    pub strategy: String,
    pub target_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, PartialEq, Copy)]
pub enum PlayType {
    Kickoff,
    Punt,
    ExtraPoint,
    FieldGoal,
    Standard,
    None,
}

impl PlayType {
    pub fn create_impl(&self) -> Box<dyn PlayImpl + Send> {
        match self {
            PlayType::Kickoff => return Box::new(KickoffPlay::new()),
            PlayType::Standard => return Box::new(StandardPlay::new()),
            _ => {
                return Box::new(KickoffPlay {
                    ..KickoffPlay::default()
                })
            }
        }
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

pub fn run_play(
    game_state: &GameState,
    fac_deck: &mut FacManager,
    play: &Box<dyn PlayImpl + Send>,
) -> Result<PlayAndState, String> {
    play.validate()?;

    let mut card_streamer = CardStreamer::new(fac_deck);

    let result = play.run_play(game_state, &mut card_streamer);

    if result.cards.had_z {
        StandardPlay::handle_z(&result);
    }

    let new_state = calculate_play_result(game_state, &result);

    return Ok(PlayAndState {
        play: play.get_play(),
        result,
        new_state,
    });
}

#[derive(Debug, Clone, EnumAsInner, Serialize)]
#[serde(untagged)]
pub enum OffenseIDLineup {
    KickoffIDOffenseLineup(KickoffIDOffenseLineup),
    StandardIDOffenseLineup(StandardIDOffenseLineup),
}

impl_deserialize!(OffenseIDLineup {
    KickoffIDOffenseLineup(KickoffIDOffenseLineup),
    StandardIDOffenseLineup(StandardIDOffenseLineup)
});

#[derive(Debug, Clone, EnumAsInner, Serialize)]
#[serde(untagged)]
pub enum DefenseIDLineup {
    KickoffIDDefenseLineup(KickoffIDDefenseLineup),
    StandardIDDefenseLineup(StandardIDDefenseLineup),
}

impl_deserialize!(DefenseIDLineup {
    KickoffIDDefenseLineup(KickoffIDDefenseLineup),
    StandardIDDefenseLineup(StandardIDDefenseLineup)
});

#[derive(Debug, Clone, EnumAsInner)]
pub enum DefenseCall {
    StandardDefenseCall(StandardDefenseCall),
    KickoffDefenseCall(KickoffDefenseCall),
    PuntDefenseCall(PuntDefenseCall),
}

impl_deserialize!(DefenseCall {
    StandardDefenseCall(StandardDefenseCall),
    KickoffDefenseCall(KickoffDefenseCall),
    PuntDefenseCall(PuntDefenseCall)
});

#[derive(Debug, Clone, EnumAsInner)]
pub enum OffenseCall {
    StandardOffenseCall(StandardOffenseCall),
    KickoffOffenseCall(KickoffOffenseCall),
    PuntOffenseCall(PuntOffenseCall),
}

impl_deserialize!(OffenseCall {
    StandardOffenseCall(StandardOffenseCall),
    KickoffOffenseCall(KickoffOffenseCall),
    PuntOffenseCall(PuntOffenseCall)
});

// impl<'de> Deserialize<'de> for OffenseCall {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         #[derive(Deserialize)]
//         #[serde(untagged)]
//         enum TaggedOffenseCall {
//             KickoffOffenseCall(KickoffOffenseCall),
//             PuntOffenseCall(PuntOffenseCall),
//         }

//         let tagged_call = TaggedOffenseCall::deserialize(deserializer)?;

//         Ok(match tagged_call {
//             TaggedOffenseCall::KickoffOffenseCall(kc) => OffenseCall::KickoffOffenseCall(kc),
//             TaggedOffenseCall::PuntOffenseCall(pc) => OffenseCall::PuntOffenseCall(pc),
//         })
//     }
// }

#[derive(Debug, Clone, Deserialize)]
pub struct KickoffDefenseCall {}

#[derive(Debug, Clone, Deserialize)]
pub struct KickoffOffenseCall {
    pub onside: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PuntDefenseCall {
    pub attempt_block: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PuntOffenseCall {
    pub coffin_corner: i32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct KickoffPlay {
    pub onside: Option<bool>,
    pub kr: Option<KRStats>,
    pub k: Option<KStats>,
}

impl KickoffPlay {
    pub fn new() -> Self {
        return Self {
            ..Default::default()
        };
    }
}

impl PlayImpl for KickoffPlay {
    fn validate(&self) -> Result<(), String> {
        validate_field!(self.onside, "Offense Call");
        validate_field!(self.kr, "Defense Lineup");
        validate_field!(self.k, "Offense Lineup");
        Ok(())
    }

    fn set_offense_call(&mut self, call: OffenseCall) -> Result<(), String> {
        let c = call
            .as_kickoff_offense_call()
            .ok_or("Bad type".to_string())?;
        self.onside = Some(c.onside);
        Ok(())
    }

    fn set_defense_call(&mut self, call: DefenseCall) -> Result<(), String> {
        Ok(())
    }

    fn set_offense_lineup(
        &mut self,
        lineup: &OffenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String> {
        let l = lineup
            .as_kickoff_id_offense_lineup()
            .ok_or("Bad type".to_string())?;

        self.k = Player::is_k(
            roster
                .get_player(&l.k)
                .ok_or(format!("Unknown player: {}", l.k))?
                .get_full_player(),
        );

        if self.k.is_none() {
            return Err("Player is not a K".to_string());
        }

        return Ok(());
    }

    fn set_defense_lineup(
        &mut self,
        lineup: &DefenseIDLineup,
        roster: &Roster,
    ) -> Result<(), String> {
        let l = lineup
            .as_kickoff_id_defense_lineup()
            .ok_or("Bad type".to_string())?;

        self.kr = Player::is_kr(
            roster
                .get_player(&l.kr)
                .ok_or(format!("Unknown player: {}", l.kr))?
                .get_full_player(),
        );

        if self.kr.is_none() {
            return Err("Player is not a KR".to_string());
        }

        return Ok(());
    }

    fn run_play<'a>(
        &'a self,
        game_state: &'a GameState,
        card_streamer: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        return KickPlayImpl::run_play(game_state, self, card_streamer);
    }

    fn get_play(&self) -> Play {
        return Play::Kickoff(self.clone());
    }

    fn get_type(&self) -> PlayType {
        PlayType::Kickoff
    }
}
