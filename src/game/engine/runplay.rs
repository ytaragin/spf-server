use crate::game::{
    fac::{FacData, RunResult, RunResultActual},
    players::{BasePlayer, Player, PlayerUtils},
    stats, GameState,
};
use itertools::fold;

use super::{OffensivePlayType, PlayLogicState, PlayResult, PlaySetup};

#[derive(Clone)]
pub struct RunPlayData {
    details: Vec<String>,
    modifier: i32,
    yardage: i32,
    result: Option<PlayResult>,
    ob: bool,
}

impl RunPlayData {
    fn new() -> Self {
        return Self {
            details: vec![],
            result: None,
            modifier: 0,
            yardage: 0,
            ob: false,
        };
    }

    fn get_fac_result<'a>(play_type: &OffensivePlayType, card: &'a FacData) -> &'a RunResult {
        match play_type {
            OffensivePlayType::SL => &card.sl,
            OffensivePlayType::SR => &card.sr,
            OffensivePlayType::IL => &card.il,
            OffensivePlayType::IR => &card.ir,
            _ => &RunResult::Break,
        }
    }
}

pub struct RunUtils {}
impl RunUtils {
    // fn create_run_play<'a>(setup: &'a PlaySetup) ->  Box<dyn PlayRunner2+'a> {
    pub fn create_run_play() -> Box<dyn PlayLogicState> {
        let data = RunPlayData::new();
        // return Box::new(p);
        return Box::new(RunStateStart { data });
    }

    fn calculate_run_yardage_modifier(
        result: &RunResultActual,
        play: &PlaySetup,
    ) -> (i32, Vec<String>) {
        let mut logs: Vec<String> = Vec::new();
        logs.push(format!(
            "It's {:?} against {:?}",
            result.offensive_boxes, result.defensive_boxes
        ));

        let tackles: i32 = result
            .defensive_boxes
            .iter()
            .flat_map(|s| play.defense.get_players_in_pos(s))
            // .flatten()
            .fold(0, |acc, ele| acc + PlayerUtils::get_tackles(ele));

        let blocks = result
            .offensive_boxes
            .iter()
            .map(|s| play.offense.get_player_in_pos(s))
            // .flatten()
            .fold(0, |acc, ele| acc + PlayerUtils::get_blocks(ele));

        let modifier = match tackles.cmp(&blocks) {
            std::cmp::Ordering::Less => {
                logs.push(format!(
                    "Block spring the runner for an extra {} yards",
                    blocks
                ));
                blocks
            }
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => {
                logs.push(format!("Big tackle to save {} yards", tackles));
                -tackles
            }
        };

        return (modifier, logs);
    }

    fn handle_bad_play(mut data: RunPlayData, error: String) -> Box<dyn PlayLogicState> {
        data.details.push(error);
        data.result = Some(PlayResult {
            result: 0,
            time: 10,
            details: data.details.clone(),
            extra: None,
        });
        return Box::new(RunStateEnd { data: data });
    }

    fn finalize_yardage(mut data: RunPlayData) -> Box<dyn PlayLogicState> {
        data.yardage += data.modifier;
        data.details.push(format!("Gain of {} yards", data.yardage));
        data.result = Some(PlayResult {
            result: data.yardage,
            time: 10,
            details: data.details.clone(),
            extra: None,
        });

        return Box::new(RunStateEnd { data: data });
    }

    fn get_lg_yardage(c: char) -> i32 {
        match c {
            'A' => 100,
            'B' => 95,
            'C' => 90,
            'D' => 85,
            'E' => 80,
            'F' => 75,
            'G' => 70,
            'H' => 65,
            'I' => 60,
            'J' => 55,
            'K' => 50,
            'L' => 45,
            'M' => 40,
            'N' => 35,
            'O' => 30,
            'P' => 25,
            'Q' => 20,
            'R' => 15,
            // Use an underscore to handle any other char values and return a default value
            _ => 0,
        }
    }
}

struct RunStateStart {
    data: RunPlayData,
}

impl RunStateStart {
    fn get_run_block(
        mut data: RunPlayData,
        play: &PlaySetup,
        player: &dyn BasePlayer,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        // println!("RunStateStart::get_run_block");

        data.details
            .push(format!("Handoff to {}", player.get_name()));

        let res = RunPlayData::get_fac_result(&play.offense_call.play_type, card);
        match res {
            RunResult::Actual(actual) => {
                let (modifier, mut logs) = RunUtils::calculate_run_yardage_modifier(&actual, play);
                data.modifier = modifier;
                data.details.append(&mut logs);
                return Box::new(RunStateYardage { data });
            }
            RunResult::Break => {
                data.details.push("It's a breakaway".to_string());
                return Box::new(RunStateBreakawayBase { data });
            }
        };
    }
}

impl PlayLogicState for RunStateStart {
    fn handle_card(
        &self,
        _state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        println!("RunStateStart::handle_card");

        let mut data = self.data.clone();
        let player = play.offense.get_player_in_pos(&play.offense_call.target);
        match player {
            Some(p) => {
                return Self::get_run_block(data, play, p, card);
            }
            None => {
                // this should really never happen
                return RunUtils::handle_bad_play(
                    data,
                    format!(
                        "Handoff to nobody in position {:?}",
                        play.offense_call.target
                    ),
                );
            }
        };
    }
    fn get_result(&self) -> Option<PlayResult> {
        println!("RunStateStart::get_result");
        self.data.result.clone()
    }
}

#[derive(Clone)]
struct RunStateYardage {
    data: RunPlayData,
}

impl PlayLogicState for RunStateYardage {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        println!("RunStateYardage::handle_card");

        let player = play
            .offense
            .get_player_in_pos(&play.offense_call.target)
            .unwrap()
            .get_full_player();
        let rb = Player::is_rb(player).unwrap();
        let mut data = self.data.clone();

        //  { return RunUtils::handle_bad_play(self.data.clone(), "Player not a running back".to_string());}
        let stat = rb
            .rushing
            .get_stat(card.run_num.num.try_into().unwrap())
            .get_val("N".to_string())
            .unwrap();
        match stat {
            stats::NumStat::Sg | stats::NumStat::Lg => {
                data.details
                    .push(format!("{} gets out for short gain", rb.get_name()));
                return Box::new(RunStateSGYardage { data });
            }
            stats::NumStat::Val(num) => {
                data.yardage = *num;
                data.ob = card.run_num.ob;
                return RunUtils::finalize_yardage(data);
            }
        }
    }
}

#[derive(Clone)]
struct RunStateBreakawayBase {
    data: RunPlayData,
}

impl PlayLogicState for RunStateBreakawayBase {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        println!("RunStateBreakawayBase::handle_card");

        let player = play
            .offense
            .get_player_in_pos(&play.offense_call.target)
            .unwrap()
            .get_full_player();
        let rb = Player::is_rb(player).unwrap();
        let yardage = RunUtils::get_lg_yardage(rb.lg);

        let mut data = self.data.clone();
        return Box::new(RunStateEnd {
            data: self.data.clone(),
        });
    }
}

#[derive(Clone)]
struct RunStateBreakawayYardage {
    data: RunPlayData,
}

impl PlayLogicState for RunStateBreakawayYardage {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        println!("RunStateBreakawayYardage::handle_card");

        let mut data = self.data.clone();

        data.ob = card.run_num.ob;

        data.yardage = data.yardage - (5 * card.run_num.num);
        return RunUtils::finalize_yardage(data);
    }
}

#[derive(Clone)]
struct RunStateSGYardage {
    data: RunPlayData,
}

impl PlayLogicState for RunStateSGYardage {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        println!("RunStateSGYardage::handle_card");

        let mut data = self.data.clone();
        let rn = card.run_num.num;
        data.yardage = rn + 5;
        data.ob = card.run_num.ob;

        return RunUtils::finalize_yardage(data);
    }
}

#[derive(Clone)]
struct RunStateEnd {
    data: RunPlayData,
}

impl PlayLogicState for RunStateEnd {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        println!("RunStateEnd::handle_card");

        return Box::new(RunStateEnd {
            data: self.data.clone(),
        });
    }
    fn get_result(&self) -> Option<PlayResult> {
        return self.data.result.clone();
    }
}
