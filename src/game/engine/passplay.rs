use std::cmp::min;

use crate::game::{
    fac::{FacData, PassTarget},
    lineup::OffensiveBox,
    players::{Player, PlayerUtils, QBStats},
    stats::{NumStat, RangedStats, TripleStat, TwelveStats},
    GameState,
};

use super::{
    CardStreamer, OffensivePlayInfo, PassMetaData, PassResult, PlayLogicState, PlayResult,
    PlaySetup, TIMES,
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
            PassResult::Interception => self.interception(),
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
            result: 0,
            time: TIMES.pass_play_incomplete,
            details: self.data.details.clone(),
            mechanic: vec![],
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
            result: yards,
            time: 10,
            details: self.data.details.clone(),
            mechanic: self.data.mechanic.clone(),
            extra: None,
            cards: self.cards.get_results(),
        };
    }

    fn interception(&mut self) -> PlayResult {
        self.data.details.push("It's intercepted".to_string());
        self.incomplete_pass()
    }

    fn calculate_pass_shift(&mut self) -> i32 {
        0
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

    fn get_fac(&mut self) -> FacData {
        let card = self.cards.get_fac();
        self.data
            .mechanic
            .push(format!("Card Flipped: {}", (card.id)));
        card
    }
}
