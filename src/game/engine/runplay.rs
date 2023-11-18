use std::cmp::{max, min};

use crate::game::{
    engine::{GAMECONSTANTS, TIMES},
    fac::{FacData, RunDirection, RunDirectionActual, RunNum},
    lineup::OffensiveBox,
    players::{BasePlayer, Player, PlayerUtils, RBStats},
    stats::{self, NumStat},
    Game, GameState,
};
use itertools::fold;
use option_ext::OptionExt;

use super::{
    CardStreamer, DefensivePlay, OffenseCall, OffensivePlayInfo, OffensivePlayType, PlayLogicState,
    PlayResult, PlaySetup, ResultType, RunMetaData,
};

// use macro_rules! <name of macro> {<Body>}
macro_rules! mechanic {
    // match like arm for macro
    ($ctxt:expr, $msg:expr, $val:expr) => {
        // macro expands to this code
        // $msg and $val will be templated using the value/variable provided to macro
        $ctxt.data.mechanic.push(format!($msg, $val));
    };
}

macro_rules! detail {
    // match like arm for macro
    ($ctxt:expr, $msg:expr) => {
        // macro expands to this code
        // $msg and $val will be templated using the value/variable provided to macro
        $ctxt.data.details.push($msg.to_string());
    };
}

pub struct RunUtils {}
impl RunUtils {
    pub fn handle_run_play<'a>(
        state: &'a GameState,
        play: &'a PlaySetup<'a>,
        cards: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        let mut data = RunPlayData::new(play.offense_metadata);
        let mut context = RunContext {
            state,
            play,
            cards,
            data,
        };
        return context.start_run();
    }

    pub fn get_sl_fac_result<'a>(card: &'a FacData) -> &'a RunDirection {
        &card.sl
    }
    pub fn get_sr_fac_result<'a>(card: &'a FacData) -> &'a RunDirection {
        &card.sr
    }
    pub fn get_il_fac_result<'a>(card: &'a FacData) -> &'a RunDirection {
        &card.il
    }
    pub fn get_ir_fac_result<'a>(card: &'a FacData) -> &'a RunDirection {
        &card.ir
    }
}

#[derive(Clone)]
pub struct RunPlayData {
    details: Vec<String>,
    mechanic: Vec<String>,
    yardage: i32,
    // result: Option<PlayResult>,
    ob: bool,
    md: RunMetaData,
}

impl RunPlayData {
    fn new(playinfo: &OffensivePlayInfo) -> Self {
        return Self {
            details: vec![],
            mechanic: vec![],
            yardage: 0,
            ob: false,
            md: playinfo.play_type.as_run().unwrap().clone(),
        };
    }
}

struct RunContext<'a> {
    state: &'a GameState,
    play: &'a PlaySetup<'a>,
    cards: &'a mut CardStreamer<'a>,
    data: RunPlayData,
}

impl<'a> RunContext<'a> {
    fn start_run(&mut self) -> PlayResult {
        let player = get_rb_stats(self.play);

        detail!(self, format!("Handoff to {}", player.get_name()));

        let dir = self.get_run_direction();
        match dir {
            RunDirection::Actual(actual) => self.handle_actual_run(&actual),
            RunDirection::Break => self.handle_breakaway(),
        }
    }

    fn handle_breakaway(&mut self) -> PlayResult {
        detail!(self, "It's a breakaway");
        let rb = get_rb_stats(self.play);
        self.data.yardage = get_lg_yardage(rb.lg);
        return self.finalize_yardage();
    }

    fn get_run_direction(&mut self) -> RunDirection {
        let card = &self.cards.get_fac();
        // let res = RunPlayData::get_fac_result(&play.offense_call.play_type, card);
        let res = (self.data.md.card_val)(card);
        mechanic!(self, "Run Result {:?}", res);
        return res.clone();
    }

    fn handle_actual_run(&mut self, actual: &RunDirectionActual) -> PlayResult {
        let rb = get_rb_stats(self.play);

        let run_num_modifier = self.get_run_modifier();
        let run_num_full = self.get_run_num();
        let run_num = min(run_num_full.num + run_num_modifier, 12);

        let stat = get_rush_stat(&rb, run_num);
        match stat {
            stats::NumStat::Sg | stats::NumStat::Lg => {
                (self.data.yardage, self.data.ob) = self.calculate_sg_yardage();
            }
            stats::NumStat::Val(num) => {
                self.data.yardage = *num;
                self.data.ob = run_num_full.ob;
            }
        }

        self.data.yardage += self.calculate_run_yardage_modifier(actual);
        return self.finalize_yardage();
    }

    fn calculate_sg_yardage(&mut self) -> (i32, bool) {
        detail!(self, "He gets out for short gain");
        let rn = self.get_run_num();
        (rn.num + 5, rn.ob)
    }

    fn calculate_run_yardage_modifier(&mut self, result: &RunDirectionActual) -> i32 {
        detail!(
            self,
            format!(
                "It's {:?} against {:?}",
                result.offensive_boxes, result.defensive_boxes
            )
        );

        let tackles: i32 = result
            .defensive_boxes
            .iter()
            .flat_map(|s| self.play.defense.get_players_in_pos(s))
            // .flatten()
            .fold(0, |acc, ele| acc + PlayerUtils::get_tackles(ele));

        let blocks = result
            .offensive_boxes
            .iter()
            .map(|s| self.play.offense.get_player_in_pos(s))
            // .flatten()
            .fold(0, |acc, ele| acc + PlayerUtils::get_blocks(ele));

        let modifier = match tackles.cmp(&blocks) {
            std::cmp::Ordering::Less => {
                detail!(
                    self,
                    format!("Block spring the runner for an extra {} yards", blocks)
                );
                blocks
            }
            std::cmp::Ordering::Equal => {
                detail!(self, "Runner gets by blocks and tackles");
                0
            }
            std::cmp::Ordering::Greater => {
                detail!(self, format!("Big tackle to save {} yards", tackles));
                -tackles
            }
        };

        return modifier;
    }

    fn get_run_modifier(&mut self) -> i32 {
        if self.play.defense_call.defense_type != DefensivePlay::RunDefense {
            mechanic!(self, "Run modifier {}", 0);
            return 0;
        }

        detail!(self, "The run defense focuses on the run");
        let mut modifier = 2;
        if let Some(pos) = &self.play.defense_call.key {
            if *pos == self.play.offense_call.target {
                detail!(self, " and they key on the right back");
                modifier += 2;
            } else {
                detail!(self, "But they focus on the wrong back");
                modifier -= 2;
            }
        }
        mechanic!(self, "Run modifier {}", modifier);
        return modifier;
    }

    fn handle_bad_play(&mut self) -> PlayResult {
        return PlayResult {
            result_type: ResultType::Regular,
            result: 0,
            time: 10,
            details: self.data.details.clone(),
            mechanic: vec![],
            extra: None,
            cards: self.cards.get_results(),
        };
        // return Box::new(RunStateEnd { data: data });
    }

    fn finalize_yardage(&mut self) -> PlayResult {
        let result = max(self.data.yardage, self.data.md.max_loss);

        let mut time = TIMES.run_play;

        if self.data.ob && self.data.md.can_go_ob {
            detail!(self, "Play ends out of bounds");
            time = TIMES.run_play_ob;
        }

        detail!(self, format!("Gain of {} yards", self.data.yardage));
        return PlayResult {
            result_type: ResultType::Regular,
            result,
            time,
            details: self.data.details.clone(),
            mechanic: self.data.mechanic.clone(),
            extra: None,
            cards: self.cards.get_results(),
        };
    }

    fn get_pass_num(&mut self) -> i32 {
        let card = self.get_fac();
        let pass_num = card.pass_num;
        mechanic!(self, "Pass Num: {}", pass_num);
        pass_num
    }

    fn get_run_num(&mut self) -> RunNum {
        let card = self.get_fac();
        let run_num = card.run_num;
        mechanic!(self, "Run Num: {:?}", run_num);
        run_num
    }

    fn get_fac(&mut self) -> FacData {
        let card = self.cards.get_fac();
        self.data
            .mechanic
            .push(format!("Card Flipped: {}", (card.id)));
        card
    }

    fn create_result(&mut self, result: i32, result_type: ResultType, time: i32) -> PlayResult {
        return PlayResult {
            result_type,
            result,
            time,
            details: self.data.details.clone(),
            mechanic: self.data.mechanic.clone(),
            extra: None,
            cards: self.cards.get_results(),
        };
    }
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

fn get_rb_stats(play: &PlaySetup) -> RBStats {
    let player = play
        .offense
        .get_player_in_pos(&play.offense_call.target)
        .unwrap()
        .get_full_player();
    Player::is_rb(player).unwrap()
}

fn get_rush_stat<'a>(rb: &'a RBStats, run_num: i32) -> &'a NumStat {
    rb.rushing
        .get_stat(run_num.try_into().unwrap())
        .get_val("N".to_string())
        .unwrap()
}
