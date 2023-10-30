use std::cmp::min;

use crate::game::{
    fac::{FacData, PassTarget},
    lineup::OffensiveBox,
    players::{BasePlayer, Player, PlayerUtils, Position, QBStats},
    stats::{NumStat, RangedStats},
    GameState,
};

use super::{
    CardStreamer, OffensivePlayInfo, PassMetaData, PassResult, PlayResult, PlaySetup, ResultType,
    INTERCEPTION_RETURN_TABLE, INTERCEPTION_TABLE, PASS_DEFENDERS, TIMES,
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

pub struct PassUtils {}
impl PassUtils {
    pub fn handle_pass_play<'a>(
        state: &'a GameState,
        play: &'a PlaySetup<'a>,
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
    play: &'a PlaySetup<'a>,
    cards: &'a mut CardStreamer<'a>,
    data: PassPlayData,
}
impl<'a> PassContext<'a> {
    fn start_pass(&mut self) -> PlayResult {
        let card = self.get_fac();
        let target = (self.data.md.target)(&card);

        mechanic!(self, "Target card returned: {:?}", target);

        match target {
            PassTarget::PassRush => return self.handle_pass_rush(),
            PassTarget::Orig => {
                self.data.target = self.play.offense_call.target;
                self.data.details.push(format!(
                    "The pass is thrown towards the {:?}",
                    self.data.target
                ));
            }
            PassTarget::Actual(target) => {
                self.data.target = *target;
                self.data.details.push(format!(
                    "The QB adjusts and throws it towards the {:?} ",
                    self.data.target
                ));
                if self
                    .play
                    .offense
                    .get_player_in_pos(&self.data.target)
                    .is_none()
                {
                    mechanic!(self, "{:?} is empty", self.data.target);
                    self.data.details.push("But no one is there".to_string());
                    return self.incomplete_pass();
                }
            }
        }

        return self.handle_check_result();
    }

    fn handle_check_result(&mut self) -> PlayResult {
        let pass_num = self.get_pass_num();

        let qb = PassContext::get_qb_stats(self.play);

        let shift = self.calculate_pass_shift();

        let range = (self.data.md.completion_range)(&qb);
        mechanic!(self, "Completion Range: {:?}", range);
        let res = range.get_category(pass_num, shift);
        mechanic!(self, "Pass Result: {:?} ", res);

        match res {
            PassResult::Complete => self.complete_pass(),
            PassResult::Incomplete => self.incomplete_pass(),
            PassResult::Interception => self.qb_interception(),
        }
    }

    fn handle_pass_rush(&mut self) -> PlayResult {
        self.data.details.push("The pass rush gets in".to_string());
        self.incomplete_pass()
    }

    fn complete_pass(&mut self) -> PlayResult {
        self.data.details.push("Pass Complete".to_string());

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
        self.data
            .details
            .push("The pass falls incomplete".to_string());

        return PlayResult {
            result_type: ResultType::Regular,
            result: 0,
            time: TIMES.pass_play_incomplete,
            details: self.data.details.clone(),
            mechanic: self.data.mechanic.clone(),
            extra: None,
            cards: self.cards.get_results(),
        };
    }

    fn short_gain(&mut self) -> PlayResult {
        self.finalize_pass(15)
    }

    fn long_gain(&mut self) -> PlayResult {
        self.data.details.push("It's a long gain".to_string());
        let yards = min(30, self.get_run_num() * 4);
        self.finalize_pass(yards)
    }

    fn finalize_pass(&mut self, yards: i32) -> PlayResult {
        self.data
            .details
            .push(format!("Pass complete for {} yards", yards));

        return PlayResult {
            result_type: ResultType::Regular,
            result: yards,
            time: TIMES.pass_play_complete,
            details: self.data.details.clone(),
            mechanic: self.data.mechanic.clone(),
            extra: None,
            cards: self.cards.get_results(),
        };
    }

    fn qb_interception(&mut self) -> PlayResult {
        let def_box = INTERCEPTION_TABLE
            .get_stat(self.get_run_num() as usize)
            .get_val(self.play.offense_metadata.code.to_string())
            .unwrap();

        let int_point = self.get_interception_point();
        self.data.details.push(format!(
            "The QB throws it towards the defense {:?}, {} yards downfield",
            def_box, int_point
        ));

        let players = self.play.defense.get_players_in_pos(def_box);
        if players.is_empty() {
            self.data
                .details
                .push("But there is no one there".to_string());
            return self.incomplete_pass();
        }

        let ret_yards = self.get_return_yardage(players[0]);

        return PlayResult {
            result_type: ResultType::TurnOver,
            result: int_point - ret_yards,
            time: TIMES.pass_play_complete,
            details: self.data.details.clone(),
            mechanic: self.data.mechanic.clone(),
            extra: None,
            cards: self.cards.get_results(),
        };
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
        self.data
            .details
            .push(format!("It's returned for {} yard", ret_yards));

        return *ret_yards;
    }

    fn get_pass_gain(&mut self) -> Option<NumStat> {
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
        self.data
            .details
            .push(format!("Pass defended by {:?}", def_box));
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

    fn get_fac(&mut self) -> FacData {
        let card = self.cards.get_fac();
        self.data
            .mechanic
            .push(format!("Card Flipped: {}", (card.id)));
        card
    }
}
