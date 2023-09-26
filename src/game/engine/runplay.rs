use std::cmp::min;

use crate::game::{
    fac::{FacData, RunResult, RunResultActual},
    players::{BasePlayer, Player, PlayerUtils},
    stats, GameState,
};
use itertools::fold;
use option_ext::OptionExt;

use super::{DefensivePlay, OffensivePlayType, PlayLogicState, PlayResult, PlaySetup};

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

fn get_run_modifier(play: &PlaySetup) -> (i32, Vec<String>) {
    let mut modifier = 0;
    let mut logs: Vec<String> = Vec::new();
    if play.defense_call.defense_type == DefensivePlay::RunDefense {
        logs.push("The run defense focuses on the run".to_string());
        modifier = 2;
        if let Some(pos) = &play.defense_call.key {
            if *pos == play.offense_call.target {
                logs.push("They key on the right back".to_string());
                modifier += 2;
            } else {
                logs.push("But they focus on the wrong back".to_string());
                modifier -= 2;
            }
        }
    }
    return (modifier, logs);
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
                let (modifier, mut logs) = calculate_run_yardage_modifier(&actual, play);
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
    fn get_name(&self) -> &str {
        return "RunStateStart";
    }

    fn handle_card(
        &self,
        _state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        let mut data = self.data.clone();
        let player = play.offense.get_player_in_pos(&play.offense_call.target);
        match player {
            Some(p) => {
                return Self::get_run_block(data, play, p, card);
            }
            None => {
                // this should really never happen
                return handle_bad_play(
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
    fn get_name(&self) -> &str {
        return "RunStateYardage";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        let player = play
            .offense
            .get_player_in_pos(&play.offense_call.target)
            .unwrap()
            .get_full_player();
        let rb = Player::is_rb(player).unwrap();
        let mut data = self.data.clone();

        let (modifier, mut logs) = get_run_modifier(play);
        data.details.append(&mut logs);
        let run_num = min(card.run_num.num + modifier, 12);

        //  { return RunUtils::handle_bad_play(self.data.clone(), "Player not a running back".to_string());}
        let stat = rb
            .rushing
            .get_stat(run_num.try_into().unwrap())
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
                return finalize_yardage(data);
            }
        }
    }
}

#[derive(Clone)]
struct RunStateBreakawayBase {
    data: RunPlayData,
}

impl PlayLogicState for RunStateBreakawayBase {
    fn get_name(&self) -> &str {
        return "RunStateBreakawayBase";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        let player = play
            .offense
            .get_player_in_pos(&play.offense_call.target)
            .unwrap()
            .get_full_player();
        let rb = Player::is_rb(player).unwrap();
        let yardage = get_lg_yardage(rb.lg);

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
    fn get_name(&self) -> &str {
        return "RunStateBreakawayYardage";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        let mut data = self.data.clone();

        data.ob = card.run_num.ob;

        data.yardage = data.yardage - (5 * card.run_num.num);
        return finalize_yardage(data);
    }
}

#[derive(Clone)]
struct RunStateSGYardage {
    data: RunPlayData,
}

impl PlayLogicState for RunStateSGYardage {
    fn get_name(&self) -> &str {
        return "RunStateSGYardage";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        let mut data = self.data.clone();
        let rn = card.run_num.num;
        data.yardage = rn + 5;
        data.ob = card.run_num.ob;

        return finalize_yardage(data);
    }
}

#[derive(Clone)]
struct RunStateEnd {
    data: RunPlayData,
}

impl PlayLogicState for RunStateEnd {
    fn get_name(&self) -> &str {
        return "RunStateEnd";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        return Box::new(RunStateEnd {
            data: self.data.clone(),
        });
    }
    fn get_result(&self) -> Option<PlayResult> {
        return self.data.result.clone();
    }
}
