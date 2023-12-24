use std::cmp::min;

use crate::{
    detail,
    game::{
        engine::{
            defs::{INTERCEPTION_RETURN_TABLE, INTERCEPTION_TABLE, PASS_DEFENDERS, TIMES},
            DefensivePlay, PassRushResult,
        },
        fac::{FacData, PassTarget},
        lineup::{DefensiveBox, OffensiveBox, StandardDefensiveLineup},
        players::{BasePlayer, Player, PlayerUtils, Position, QBStats},
        stats::{NumStat, RangedStats},
        GameState,
    },
    mechanic,
};

use super::{
    playutils::PlayUtils, CardStreamer, DefenseIDLineup, OffensivePlayInfo, PassMetaData,
    PassResult, PlayResult, PlaySetup, ResultType,
};

// // use macro_rules! <name of macro> {<Body>}
// macro_rules! mechanic {
//     // match like arm for macro
//     ($ctxt:expr, $msg:expr, $val:expr) => {
//         // macro expands to this code
//         // $msg and $val will be templated using the value/variable provided to macro
//         $ctxt.data.mechanic.push(format!($msg, $val));
//     };
// }

// macro_rules! detail {
//     // match like arm for macro
//     ($ctxt:expr, $msg:expr) => {
//         // macro expands to this code
//         // $msg and $val will be templated using the value/variable provided to macro
//         $ctxt.data.details.push($msg.to_string());
//     };
// }

pub struct PassUtils {}
impl PassUtils {
    pub fn handle_pass_play<'a>(
        state: &'a GameState,
        play: PlaySetup<'a>,
        cards: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        let data = PassPlayData::new(play.offense_metadata);
        // let p_filt = play.defense.filter_players(&play.defense_call.def_players);

        // let p = if play.defense_call.defense_type == DefensivePlay::Blitz {
        //     PlaySetup {
        //         defense: &p_filt,
        //         ..play

        //     }
        // } else {
        //     play
        // };



        let mut context = PassContext {
            state,
            play,
            data,
            utils: PlayUtils::new(state, cards),
        };
        return context.start_pass();
    }

    pub fn get_qk_fac_target<'a>(card: &'a FacData) -> &'a PassTarget {
        &card.qk
    }
    pub fn get_sh_fac_target<'a>(card: &'a FacData) -> &'a PassTarget {
        &card.sh
    }
    pub fn get_lg_fac_target<'a>(card: &'a FacData) -> &'a PassTarget {
        &card.lg
    }

    pub fn get_qk_qb_range<'a>(qb: &'a QBStats) -> &'a RangedStats<PassResult> {
        &qb.quick
    }
    pub fn get_sh_qb_range<'a>(qb: &'a QBStats) -> &'a RangedStats<PassResult> {
        &qb.short
    }
    pub fn get_lg_qb_range<'a>(qb: &'a QBStats) -> &'a RangedStats<PassResult> {
        &qb.long
    }
}

#[derive(Clone)]
pub struct PassPlayData {
    // details: Vec<String>,
    // mechanic: Vec<String>,
    target: OffensiveBox,

    md: PassMetaData,
}

impl PassPlayData {
    fn new(playinfo: &OffensivePlayInfo) -> Self {
        return Self {
            // details: vec![],
            // mechanic: vec![],
            target: OffensiveBox::QB,
            md: playinfo.play_type.as_pass().unwrap().clone(),
        };
    }
}

struct PassContext<'a> {
    state: &'a GameState,
    play: PlaySetup<'a>,
    // cards: &'a mut CardStreamer<'a>,
    data: PassPlayData,
    utils: PlayUtils<'a>,
}
impl<'a> PassContext<'a> {
    fn start_pass(&mut self) -> PlayResult {
        self.data.target = self.play.offense_call.target;

        if self.play.defense_call.defense_type == super::DefensivePlay::Blitz
            && (self.play.offense_call.play_type == super::OffensivePlayType::SH
                || self.play.offense_call.play_type == super::OffensivePlayType::LG)
        {
            detail!(self.utils, "There is a blitz");
            return self.handle_pass_rush();
        }

        let card = self.utils.get_fac();
        let target = (self.data.md.target)(&card);

        mechanic!(self.utils, "Target card returned: {:?}", target);

        match target {
            PassTarget::PassRush => {
                detail!(self.utils, "The pass rush gets in");
                return self.handle_pass_rush();
            }
            PassTarget::Orig => {
                self.data.target = self.play.offense_call.target;
                detail!(
                    self.utils,
                    format!("The pass is thrown towards the {:?}", self.data.target)
                );
            }
            PassTarget::Actual(target) => {
                self.data.target = *target;
                detail!(
                    self.utils,
                    format!(
                        "The QB adjusts and throws it towards the {:?} ",
                        self.data.target
                    )
                );
                if self
                    .play
                    .offense
                    .get_player_in_pos(&self.data.target)
                    .is_none()
                {
                    mechanic!(self.utils, "{:?} is empty", self.data.target);
                    detail!(self.utils, "But no one is there");
                    return self.incomplete_pass();
                }
            }
        }

        return self.handle_check_result();
    }

    fn handle_check_result(&mut self) -> PlayResult {
        let qb = PassContext::get_qb_stats(&self.play);

        let shift = self.calculate_pass_shift();

        let range = (self.data.md.completion_range)(&qb);
        mechanic!(self.utils, "Completion Range: {:?}", range);
        let res = range.get_category(self.utils.get_pass_num(), shift);
        mechanic!(self.utils, "Pass Result: {:?} ", res);

        match res {
            PassResult::Complete => self.complete_pass(),
            PassResult::Incomplete => self.incomplete_pass(),
            PassResult::Interception => self.qb_interception(),
        }
    }

    fn handle_pass_rush(&mut self) -> PlayResult {
        let off_block = self.get_offensive_block();
        let def_rush = self.get_defensive_rush();

        let sack_range_impact = (def_rush - off_block) * 2;
        mechanic!(self.utils, "Sack range impact of {}", sack_range_impact);

        let qb = PassContext::get_qb_stats(&self.play);
        let res = qb
            .pass_rush
            .get_category(self.utils.get_pass_num(), sack_range_impact);
        mechanic!(self.utils, "Pass Rush Result: {:?}", res);
        match res {
            PassRushResult::Sack => self.sack(),
            PassRushResult::Runs => self.qb_run(),
            PassRushResult::Complete => self.complete_pass(),
            PassRushResult::Incomplete => self.incomplete_pass(),
        }
    }

    fn sack(&mut self) -> PlayResult {
        let yds = self.utils.get_pass_num() / 3;
        detail!(self.utils, format!("The QB is sacked for {} yards", yds));

        self.create_result(-yds, ResultType::Regular, TIMES.run_play)
    }

    fn qb_run(&mut self) -> PlayResult {
        let qb = PassContext::get_qb_stats(&self.play);
        let val = qb.rushing.get_stat(self.utils.get_run_num() as usize);
        let yds = match val {
            NumStat::Sg => 0,
            NumStat::Lg => 0,
            NumStat::Val(v) => *v,
        };
        detail!(self.utils, format!("The QB runs for it for {} yards", yds));

        self.create_result(yds, ResultType::Regular, TIMES.run_play)
    }

    fn complete_pass(&mut self) -> PlayResult {
        detail!(self.utils, "Pass Complete");

        let gain = self.get_pass_gain();
        match gain {
            None => self.incomplete_pass(),
            Some(ns) => match ns {
                NumStat::Sg => self.short_gain(),
                NumStat::Lg => self.long_gain(),
                NumStat::Val(v) => self.finalize_pass(v),
            },
        }
    }

    fn incomplete_pass(&mut self) -> PlayResult {
        detail!(self.utils, "The pass falls incomplete");

        self.create_result(0, ResultType::Regular, TIMES.pass_play_incomplete)
    }

    fn short_gain(&mut self) -> PlayResult {
        self.finalize_pass(15)
    }

    fn long_gain(&mut self) -> PlayResult {
        detail!(self.utils, "It's a long gain");
        let yards = min(30, self.utils.get_run_num() * 4);
        self.finalize_pass(yards)
    }

    fn finalize_pass(&mut self, yards: i32) -> PlayResult {
        detail!(self.utils, format!("Pass complete for {} yards", yards));

        self.create_result(yards, ResultType::Regular, TIMES.pass_play_complete)
    }

    fn qb_interception(&mut self) -> PlayResult {
        let def_box = INTERCEPTION_TABLE
            .get_stat(self.utils.get_run_num() as usize)
            .get_val(self.play.offense_metadata.code.to_string())
            .unwrap();

        let int_point = self.get_interception_point();
        detail!(
            self.utils,
            format!(
                "The QB throws it towards the defense {:?}, {} yards downfield",
                def_box, int_point
            )
        );

        let players = self.play.defense.get_players_in_pos(def_box);
        if players.is_empty() {
            detail!(self.utils, "But there is no one there");
            return self.incomplete_pass();
        }

        let ret_yards = self.get_return_yardage(players[0].get_pos().to_string());

        self.create_result(
            int_point - ret_yards,
            ResultType::TurnOver,
            TIMES.pass_play_complete,
        )
    }

    fn calculate_pass_shift(&mut self) -> i32 {
        let def_impact = self.get_def_impact();

        let player_impact = self.get_pass_defender_impact();

        def_impact + player_impact
    }

    fn get_qb_stats(play: &PlaySetup) -> QBStats {
        Player::is_qb(
            play.offense
                .get_player_in_pos(&OffensiveBox::QB)
                .unwrap()
                .get_full_player(),
        )
        .unwrap()
    }

    fn get_interception_point(&mut self) -> i32 {
        let rn = self.utils.get_run_num();
        match self.play.offense_call.play_type {
            super::OffensivePlayType::QK => rn,
            super::OffensivePlayType::SH => rn * 2,
            super::OffensivePlayType::LG => rn * 4,
            super::OffensivePlayType::SC => rn - 6,
            _ => 0,
        }
    }

    fn  get_return_yardage(&mut self, pos: String) -> i32 {
        let ret_yards = INTERCEPTION_RETURN_TABLE
            .get_stat(self.utils.get_run_num() as usize)
            .get_val(pos)
            .unwrap_or(&0);
        detail!(self.utils, format!("It's returned for {} yard", ret_yards));

        return *ret_yards;
    }

    fn get_pass_gain(&mut self) -> Option<NumStat> {
        println!("Target is {:?}", self.data.target);
        let pass_gain =
            PlayerUtils::get_pass_gain(self.play.offense.get_player_in_pos(&self.data.target))
                .unwrap();
        // mechanic!(self.utils, "Full Pass Chart {:?}", pass_gain);

        let run_num = self.utils.get_run_num();

        let gain = pass_gain
            .get_stat(run_num as usize)
            .get_val(self.data.md.pass_gain.clone());
        mechanic!(self.utils, "Assigned gain: {:?}", gain);

        match gain {
            Some(n) => Some(*n),
            None => None,
        }
    }

    fn get_def_impact(&mut self) -> i32 {
        let m = match self.play.defense_call.defense_type {
            super::DefensivePlay::RunDefense => match self.play.offense_call.play_type {
                super::OffensivePlayType::QK => 0,
                super::OffensivePlayType::SH => 5,
                super::OffensivePlayType::LG => 7,
                _ => 0,
            },
            super::DefensivePlay::PassDefense => match self.play.offense_call.play_type {
                super::OffensivePlayType::QK => -10,
                super::OffensivePlayType::SH => -5,
                super::OffensivePlayType::LG => 0,
                _ => 0,
            },
            super::DefensivePlay::PreventDefense => match self.play.offense_call.play_type {
                super::OffensivePlayType::QK => 0,
                super::OffensivePlayType::SH => -5,
                super::OffensivePlayType::LG => -7,
                _ => 0,
            },
            super::DefensivePlay::Blitz => 0,
        };
        mechanic!(self.utils, "Defensive Impact: {}", m);

        m
    }

    fn get_pass_defender_impact(&mut self) -> i32 {
        let def_box = PASS_DEFENDERS.get(&self.data.target).unwrap();
        detail!(self.utils, format!("Pass defended by {:?}", def_box));
        let players = self.play.defense.get_players_in_pos(def_box);
        if players.is_empty() {
            detail!(self.utils, "But there is no one there");
            mechanic!(self.utils, "No player impact: {}", 5);
            return 5;
        }
        let player_imp = players
            .iter()
            .fold(0, |acc, p| acc + PlayerUtils::get_pass_defense(*p));
        mechanic!(self.utils, "Total player impact: {}", player_imp);
        player_imp
    }

    fn get_offensive_block(&mut self) -> i32 {
        let blockers = vec![
            OffensiveBox::LT,
            OffensiveBox::LG,
            OffensiveBox::C,
            OffensiveBox::RG,
            OffensiveBox::RT,
        ];
        let val = blockers
            .iter()
            .map(|spot| self.play.offense.get_player_in_pos(spot).unwrap())
            .fold(0, |acc, p| acc + PlayerUtils::get_pass_block(p));

        mechanic!(self.utils, "Blocking value of {}", val);
        val
    }

    fn get_defensive_rush(&mut self) -> i32 {
        let rushers = vec![
            DefensiveBox::BoxA,
            DefensiveBox::BoxB,
            DefensiveBox::BoxC,
            DefensiveBox::BoxD,
            DefensiveBox::BoxE,
        ];
        let val = rushers
            .iter()
            .flat_map(|b| {
                self.play
                    .defense
                    .get_players_in_pos(b)
                    .iter()
                    .map(|p| PlayerUtils::get_pass_rush(*p))
                    .collect::<Vec<_>>()
            })
            .sum();
        // .fold(0, |acc, v| acc + v);

        let blitzing_val: i32;
        if self.play.defense_call.defense_type == DefensivePlay::Blitz {
            blitzing_val = (self.play.defense_call.def_players.len() as i32) * 2;
            mechanic!(self.utils, "Blitzing Val {}", blitzing_val);
        } else {
            blitzing_val = 0;
        }

        mechanic!(self.utils, "Total rushing value of {}", val + blitzing_val);
        val
    }

    fn create_result(&mut self, result: i32, result_type: ResultType, time: i32) -> PlayResult {
        return PlayResult {
            result_type,
            result,
            final_line: result + self.state.yardline,
            time,
            // details: self.data.details.clone(),
            // mechanic: self.data.mechanic.clone(),
            // extra: None,
            // cards: self.cards.get_results(),
            ..self.utils.result()
        };
    }



    fn is_non_blitzer(&mut self, player: String) -> bool {
        self.play.defense_call.defense_type == DefensivePlay::Blitz
            && self.play.defense_call.def_players.contains(&player)
    }
}
