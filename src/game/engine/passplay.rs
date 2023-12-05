use std::cmp::min;

use crate::game::{
    engine::{PassRushResult, defs::{TIMES, INTERCEPTION_TABLE, INTERCEPTION_RETURN_TABLE, PASS_DEFENDERS}},
    fac::{FacData, PassTarget},
    lineup::{DefensiveBox, OffensiveBox},
    players::{BasePlayer, Player, PlayerUtils, Position, QBStats},
    stats::{NumStat, RangedStats},
    GameState,
};

use super::{
    CardStreamer, OffensivePlayInfo, PassMetaData, PassResult, PlayResult, PlaySetup, ResultType,
    
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

pub struct PassUtils {}
impl PassUtils {
    pub fn handle_pass_play<'a>(
        state: &'a GameState,
        play: PlaySetup<'a>,
        cards: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        let mut data = PassPlayData::new(play.offense_metadata);

        let mut context = PassContext {
            state,
            play,
            cards,
            data,
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
    details: Vec<String>,
    mechanic: Vec<String>,
    target: OffensiveBox,

    md: PassMetaData,
    result: Option<PlayResult>,
}

impl PassPlayData {
    fn new(playinfo: &OffensivePlayInfo) -> Self {
        return Self {
            details: vec![],
            mechanic: vec![],
            target: OffensiveBox::QB,
            md: playinfo.play_type.as_pass().unwrap().clone(),
            result: None,
        };
    }
}

struct PassContext<'a> {
    state: &'a GameState,
    play: PlaySetup<'a>,
    cards: &'a mut CardStreamer<'a>,
    data: PassPlayData,
}
impl<'a> PassContext<'a> {
    fn start_pass(&mut self) -> PlayResult {
        self.data.target = self.play.offense_call.target;

        let card = self.get_fac();
        let target = (self.data.md.target)(&card);

        mechanic!(self, "Target card returned: {:?}", target);

        match target {
            PassTarget::PassRush => return self.handle_pass_rush(),
            PassTarget::Orig => {
                self.data.target = self.play.offense_call.target;
                detail!(
                    self,
                    format!("The pass is thrown towards the {:?}", self.data.target)
                );
            }
            PassTarget::Actual(target) => {
                self.data.target = *target;
                detail!(
                    self,
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
                    mechanic!(self, "{:?} is empty", self.data.target);
                    detail!(self, "But no one is there");
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
        mechanic!(self, "Completion Range: {:?}", range);
        let res = range.get_category(self.get_pass_num(), shift);
        mechanic!(self, "Pass Result: {:?} ", res);

        match res {
            PassResult::Complete => self.complete_pass(),
            PassResult::Incomplete => self.incomplete_pass(),
            PassResult::Interception => self.qb_interception(),
        }
    }

    fn handle_pass_rush(&mut self) -> PlayResult {
        detail!(self, "The pass rush gets in");

        let off_block = self.get_offensive_block();
        let def_rush = self.get_defensive_rush();

        let sack_range_impact = (def_rush - off_block) * 2;
        mechanic!(self, "Sack range impact of {}", sack_range_impact);

        let qb = PassContext::get_qb_stats(&self.play);
        let res = qb
            .pass_rush
            .get_category(self.get_pass_num(), sack_range_impact);
        mechanic!(self, "Pass Rush Result: {:?}", res);
        match res {
            PassRushResult::Sack => self.sack(),
            PassRushResult::Runs => self.qb_run(),
            PassRushResult::Complete => self.complete_pass(),
            PassRushResult::Incomplete => self.incomplete_pass(),
        }
    }

    fn sack(&mut self) -> PlayResult {
        let yds = self.get_pass_num() / 3;
        detail!(self, format!("The QB is sacked for {} yards", yds));

        self.create_result(-yds, ResultType::Regular, TIMES.run_play)
    }

    fn qb_run(&mut self) -> PlayResult {
        let qb = PassContext::get_qb_stats(&self.play);
        let val = qb.rushing.get_stat(self.get_run_num() as usize);
        let yds = match val {
            NumStat::Sg => 0,
            NumStat::Lg => 0,
            NumStat::Val(v) => *v,
        };
        detail!(self, format!("The QB runs for it for {} yards", yds));

        self.create_result(yds, ResultType::Regular, TIMES.run_play)
    }

    fn complete_pass(&mut self) -> PlayResult {
        detail!(self, "Pass Complete");

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
        detail!(self, "The pass falls incomplete");

        self.create_result(0, ResultType::Regular, TIMES.pass_play_incomplete)
    }

    fn short_gain(&mut self) -> PlayResult {
        self.finalize_pass(15)
    }

    fn long_gain(&mut self) -> PlayResult {
        detail!(self, "It's a long gain");
        let yards = min(30, self.get_run_num() * 4);
        self.finalize_pass(yards)
    }

    fn finalize_pass(&mut self, yards: i32) -> PlayResult {
        detail!(self, format!("Pass complete for {} yards", yards));

        self.create_result(yards, ResultType::Regular, TIMES.pass_play_complete)
    }

    fn qb_interception(&mut self) -> PlayResult {
        let def_box = INTERCEPTION_TABLE
            .get_stat(self.get_run_num() as usize)
            .get_val(self.play.offense_metadata.code.to_string())
            .unwrap();

        let int_point = self.get_interception_point();
        detail!(
            self,
            format!(
                "The QB throws it towards the defense {:?}, {} yards downfield",
                def_box, int_point
            )
        );

        let players = self.play.defense.get_players_in_pos(def_box);
        if players.is_empty() {
            self.data
                .details
                .push("But there is no one there".to_string());
            return self.incomplete_pass();
        }

        let ret_yards = self.get_return_yardage(players[0]);

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

    fn get_pass_num(&mut self) -> i32 {
        let card = self.get_fac();
        let pass_num = card.pass_num;
        mechanic!(self, "Pass Num: {}", pass_num);
        pass_num
    }

    fn get_run_num(&mut self) -> i32 {
        let card = self.get_fac();
        let run_num = card.run_num.num;
        mechanic!(self, "Run Num: {}", run_num);
        run_num
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
        let rn = self.get_run_num();
        match self.play.offense_call.play_type {
            super::OffensivePlayType::QK => rn,
            super::OffensivePlayType::SH => rn * 2,
            super::OffensivePlayType::LG => rn * 4,
            super::OffensivePlayType::SC => rn - 6,
            _ => 0,
        }
    }

    fn get_return_yardage(&mut self, player: &dyn BasePlayer) -> i32 {
        let ret_yards = INTERCEPTION_RETURN_TABLE
            .get_stat(self.get_run_num() as usize)
            .get_val(player.get_pos().to_string())
            .unwrap_or(&0);
        detail!(self, format!("It's returned for {} yard", ret_yards));

        return *ret_yards;
    }

    fn get_pass_gain(&mut self) -> Option<NumStat> {
        println!("Target is {:?}", self.data.target);
        let pass_gain =
            PlayerUtils::get_pass_gain(self.play.offense.get_player_in_pos(&self.data.target))
                .unwrap();
        // mechanic!(self, "Full Pass Chart {:?}", pass_gain);

        let run_num = self.get_run_num();

        let gain = pass_gain
            .get_stat(run_num as usize)
            .get_val(self.data.md.pass_gain.clone());
        mechanic!(self, "Assigned gain: {:?}", gain);

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
        mechanic!(self, "Defensive Impact: {}", m);

        m
    }

    fn get_pass_defender_impact(&mut self) -> i32 {
        let def_box = PASS_DEFENDERS.get(&self.data.target).unwrap();
        detail!(self, format!("Pass defended by {:?}", def_box));
        let players = self.play.defense.get_players_in_pos(def_box);
        if players.is_empty() {
            self.data
                .details
                .push("But there is no one there".to_string());
            mechanic!(self, "No player impact: {}", 5);
            return 5;
        }
        let player_imp = players
            .iter()
            .fold(0, |acc, p| acc + PlayerUtils::get_pass_defense(*p));
        mechanic!(self, "Total player impact: {}", player_imp);
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

        mechanic!(self, "Blocking value of {}", val);
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
        mechanic!(self, "Rushing value of {}", val);
        val
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
            final_line: result+self.state.yardline,
            time,
            details: self.data.details.clone(),
            mechanic: self.data.mechanic.clone(),
            extra: None,
            cards: self.cards.get_results(),
        };
    }
}
