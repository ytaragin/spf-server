use crate::game::{
    fac::{FacData, RunNum},
    lineup::{DefensiveBox, OffensiveBox},
    standard_play::DefensivePlay,
    GameState,
};

use super::{defs::RunPlayDefenseImpact, CardResults, CardStreamer, PlayResult};

// use macro_rules! <name of macro> {<Body>}
#[macro_export]
macro_rules! mechanic {
    // match like arm for macro
    ($ctxt:expr, $msg:expr, $val:expr) => {
        // macro expands to this code
        // $msg and $val will be templated using the value/variable provided to macro
        $ctxt.mechanic(format!($msg, $val));
    };
}

#[macro_export]
macro_rules! mechanic2 {
    // match like arm for macro
    ($ctxt:expr, $msg:expr, $val1:expr, $val2:expr) => {
        // macro expands to this code
        // $msg and $val will be templated using the value/variable provided to macro
        $ctxt.mechanic(format!($msg, $val1, $val2));
    };
}

#[macro_export]
macro_rules! detail {
    // match like arm for macro
    ($ctxt:expr, $msg:expr) => {
        // macro expands to this code
        // $msg and $val will be templated using the value/variable provided to macro
        $ctxt.detail($msg.to_string());
    };
}

#[macro_export]
macro_rules! detailf {
    // match like arm for macro
    ($ctxt:expr, $msg:expr, $val:expr) => {
        // macro expands to this code
        // $msg and $val will be templated using the value/variable provided to macro
        $ctxt.detail(format!($msg, $val));
    };
}

pub struct PlayUtils<'a> {
    details: Vec<String>,
    mechanics: Vec<String>,
    cards: &'a mut CardStreamer<'a>,
    state: &'a GameState,
}

impl<'a> PlayUtils<'a> {
    pub fn new(state: &'a GameState, cards: &'a mut CardStreamer<'a>) -> Self {
        Self {
            details: vec![],
            mechanics: vec![],
            cards,
            state,
        }
    }

    pub fn get_fac(&mut self) -> FacData {
        let card = self.cards.get_fac();
        self.mechanic(format!("Card Flipped: {}", (card.id)));
        card
    }

    pub fn get_pass_num(&mut self) -> i32 {
        let card = self.get_fac();
        let pass_num = card.pass_num;
        mechanic!(self, "Pass Num: {}", pass_num);
        pass_num
    }

    pub fn get_run_num(&mut self) -> i32 {
        self.get_full_run_num().num
    }

    pub fn get_full_run_num(&mut self) -> RunNum {
        let card = self.get_fac();
        let run_num = card.run_num;
        mechanic!(self, "Run Num: {:?}", run_num);
        run_num
    }

    pub fn mechanic(&mut self, msg: String) {
        self.mechanics.push(msg);
    }

    pub fn detail(&mut self, msg: String) {
        self.details.push(msg);
    }

    pub fn result(&self) -> PlayResult {
        PlayResult {
            details: self.details.clone(),
            mechanic: self.mechanics.clone(),
            extra: None,
            cards: self.cards.get_results(),

            result_type: super::ResultType::Regular,
            result: 0,
            final_line: 0,
            time: 0,
        }
    }

}
