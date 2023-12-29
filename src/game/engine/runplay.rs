use std::cmp::{max, min};

use crate::{
    detail, detailf,
    game::{
        engine::{
            defs::{DRAW_IMPACT, TIMES},
            OffensiveStrategy,
        },
        fac::{FacData, RunDirection, RunDirectionActual, RunNum},
        lineup::{DefensiveBox, OffensiveBox},
        players::{BasePlayer, Player, PlayerUtils, RBStats},
        stats::{self, NumStat},
        GameState,
    },
    mechanic, mechanic2,
};

use super::{
    playutils::PlayUtils, CardStreamer, DefensivePlay, OffensivePlayInfo, OffensivePlayType,
    PlayResult, PlaySetup, ResultType, RunMetaData, StandardOffenseCall,
};

pub struct RunUtils {}
impl RunUtils {
    pub fn handle_run_play<'a>(
        state: &'a GameState,
        play: PlaySetup<'a>,
        cards: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        let mut data = RunPlayData::new(play.offense_metadata);
        let mut context = RunContext {
            state,
            play,
            data,
            utils: PlayUtils::new(state, cards),
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

// #[derive(Clone)]
pub struct RunPlayData {
    yardage: i32,

    ob: bool,
    md: RunMetaData,
}

impl RunPlayData {
    fn new(playinfo: &OffensivePlayInfo) -> Self {
        return Self {
            yardage: 0,
            ob: false,
            md: playinfo.play_type.as_run().unwrap().clone(),
        };
    }
}

struct RunContext<'a> {
    state: &'a GameState,
    play: PlaySetup<'a>,
    // cards: &'a mut CardStreamer<'a>,
    data: RunPlayData,
    utils: PlayUtils<'a>,
}

impl<'a> RunContext<'a> {
    fn start_run(&mut self) -> PlayResult {
        let player = get_rb_stats(&self.play);

        detail!(self.utils, format!("Handoff to {}", player.get_name()));

        let dir = self.get_run_direction();
        match dir {
            RunDirection::Actual(actual) => self.handle_actual_run(&actual),
            RunDirection::Break => self.handle_breakaway(),
        }
    }

    fn handle_breakaway(&mut self) -> PlayResult {
        detail!(self.utils, "It's a breakaway");
        let rb = get_rb_stats(&self.play);
        self.data.yardage = get_lg_yardage(rb.lg);
        return self.finalize_yardage();
    }

    fn get_run_direction(&mut self) -> RunDirection {
        let card = &self.utils.get_fac();
        // let res = RunPlayData::get_fac_result(&play.offense_call.play_type, card);
        let res = (self.data.md.card_val)(card);
        mechanic!(self.utils, "Run Result {:?}", res);
        return res.clone();
    }

    fn handle_actual_run(&mut self, actual: &RunDirectionActual) -> PlayResult {
        let rb = get_rb_stats(&self.play);

        let run_num_modifier = self.get_run_modifier();
        let run_num_full = self.utils.get_full_run_num();
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
        mechanic!(self.utils, "Base yardage gain: {}", self.data.yardage);

        self.data.yardage += self.calculate_run_yardage_modifier(actual);
        return self.finalize_yardage();
    }

    fn calculate_sg_yardage(&mut self) -> (i32, bool) {
        detail!(self.utils, "He gets out for short gain");
        let rn = self.utils.get_full_run_num();
        (rn.num + 5, rn.ob)
    }

    fn calculate_run_yardage_modifier(&mut self, result: &RunDirectionActual) -> i32 {
        detail!(
            self.utils,
            format!(
                "It's {:?} against {:?}",
                result.offensive_boxes, result.defensive_boxes
            )
        );

        if result.offensive_boxes.len() > 0 && result.defensive_boxes.len() > 0 {
            self.off_vs_def(result)
        } else if result.offensive_boxes.len() > 0 {
            self.off_block(result)
        } else {
            self.def_tackle(result)
        }
    }

    fn off_block(&mut self, result: &RunDirectionActual) -> i32 {
        let v = result
            .offensive_boxes
            .iter()
            .map(|b| self.get_block_value(b))
            .sum();
        detailf!(self.utils, "Block springs for an extra {} yards", v);
        v
    }

    fn def_tackle(&mut self, result: &RunDirectionActual) -> i32 {
        let vals: Vec<Option<i32>> = result
            .defensive_boxes
            .iter()
            .map(|db| self.get_tackle_value(db))
            .collect();

        if vals.iter().all(|d| d.is_none()) {
            detail!(self.utils, "All def boxes were empty");
            2
        } else {
            let v = vals.iter().filter_map(|x| *x).sum();
            detailf!(self.utils, "Defense tackles for a {} yards", v);
            v
        }
    }

    fn off_vs_def(&mut self, result: &RunDirectionActual) -> i32 {
        if result.offensive_boxes.len() > 1 || result.defensive_boxes.len() > 1 {
            mechanic!(self.utils, "Unexpected run block result {:?}", result);
        }

        let b = self.get_block_value(&result.offensive_boxes[0]);

        let t_opt = self.get_tackle_value(&result.defensive_boxes[0]);
        if t_opt.is_none() {
            detailf!(
                self.utils,
                "No defense player so block gains extra {} yards",
                b
            );
            return b;
        }
        let t = t_opt.unwrap();

        let check = b + t;
        match check.cmp(&0) {
            std::cmp::Ordering::Less => {
                detailf!(
                    self.utils,
                    "Offense wins the blocking battle for an extra {} yards",
                    b
                );
                b
            }
            std::cmp::Ordering::Equal => {
                detail!(self.utils, "The blocker and tackler match up well");
                0
            }
            std::cmp::Ordering::Greater => {
                detailf!(
                    self.utils,
                    "Defense wins the tackling battle and takes away {} yards",
                    t
                );
                t
            }
        }
    }

    fn get_block_value(&mut self, o_box: &OffensiveBox) -> i32 {
        let b = PlayerUtils::get_blocks(self.play.offense.get_player_in_pos(o_box));
        mechanic2!(self.utils, "Box {:?} blocks for {}", o_box, b);
        b
    }

    fn get_tackle_value(&mut self, d_box: &DefensiveBox) -> Option<i32> {
        let ps = self.play.defense.get_players_in_pos(d_box);
        match ps.len() {
            0 => {
                mechanic!(self.utils, "Box {:?} empty.", d_box);
                None
            }
            1 => {
                let t = PlayerUtils::get_tackles(ps[0]);
                mechanic2!(self.utils, "Box {:?} tackles for {}", d_box, t);
                Some(t)
            }
            _ => {
                mechanic!(self.utils, "Box {:?} with 2 players. TV: -4", d_box);
                Some(-4)
            }
        }
    }

    fn calculate_run_yardage_modifier2(&mut self, result: &RunDirectionActual) -> i32 {
        detail!(
            self.utils,
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
                    self.utils,
                    format!("Block spring the runner for an extra {} yards", blocks)
                );
                blocks
            }
            std::cmp::Ordering::Equal => {
                detail!(self.utils, "Runner gets by blocks and tackles");
                0
            }
            std::cmp::Ordering::Greater => {
                detail!(self.utils, format!("Big tackle to save {} yards", tackles));
                -tackles
            }
        };

        return modifier;
    }

    fn get_run_modifier(&mut self) -> i32 {
        let mut modifier = self.get_drawplay_impact();

        if self.play.defense_call.defense_type == DefensivePlay::RunDefense {
            detail!(self.utils, "The run defense focuses on the run");
            modifier += 2;
            if let Some(pos) = &self.play.defense_call.key {
                if *pos == self.play.offense_call.target {
                    detail!(self.utils, " and they key on the right back");
                    modifier += 2;
                } else {
                    detail!(self.utils, "But they focus on the wrong back");
                    modifier -= 2;
                }
            }
        }

        mechanic!(self.utils, "Run modifier {}", modifier);
        return modifier;
    }

    fn get_drawplay_impact(&mut self) -> i32 {
        if self.play.offense_call.strategy == OffensiveStrategy::Draw
            && (self.play.offense_call.play_type == OffensivePlayType::IL
                || self.play.offense_call.play_type == OffensivePlayType::IR)
        {
            let val = match self.play.defense_call.defense_type {
                DefensivePlay::RunDefense => DRAW_IMPACT.run_defense,
                DefensivePlay::PassDefense => DRAW_IMPACT.pass_defense,
                DefensivePlay::PreventDefense => DRAW_IMPACT.prevent_defense,
                DefensivePlay::Blitz => DRAW_IMPACT.blitz,
            };

            if val < 0 {
                detail!(self.utils, "The draw play fools the defense");
            } else {
                detail!(self.utils, "The draw crashes into the run defense");
            }
            mechanic!(self.utils, "Draw modifier {}", val);

            return val;
        }

        return 0;
    }

    fn handle_bad_play(&mut self) -> PlayResult {
        return self.create_result(0, ResultType::Regular, 10);
    }

    fn finalize_yardage(&mut self) -> PlayResult {
        let result = max(self.data.yardage, self.data.md.max_loss);

        let mut time = TIMES.run_play;

        if self.data.ob && self.data.md.can_go_ob {
            detail!(self.utils, "Play ends out of bounds");
            time = TIMES.run_play_ob;
        }

        detail!(self.utils, format!("Gain of {} yards", result));
        return self.create_result(result, ResultType::Regular, time);
    }

    fn create_result(&mut self, result: i32, result_type: ResultType, time: i32) -> PlayResult {
        return PlayResult {
            result_type,
            result,
            final_line: result + self.state.yardline,
            time,

            ..self.utils.result()
        };
    }
}

fn get_lg_yardage(c: char) -> i32 {
    // match c {
    //     'A' => 100,
    //     'B' => 95,
    //     'C' => 90,
    //     'D' => 85,
    //     'E' => 80,
    //     'F' => 75,
    //     'G' => 70,
    //     'H' => 65,
    //     'I' => 60,
    //     'J' => 55,
    //     'K' => 50,
    //     'L' => 45,
    //     'M' => 40,
    //     'N' => 35,
    //     'O' => 30,
    //     'P' => 25,
    //     'Q' => 20,
    //     'R' => 15,
    //     // Use an underscore to handle any other char values and return a default value
    //     _ => 0,
    // }

    // let ascii_value = c as i32;
    100 - (c as i32 - 65) * 5
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
