use crate::{
    detail,
    game::{
        engine::defs::{GAMECONSTANTS, KICKOFFRESULTSB},
        players::Returner,
        GameState,
    },
    mechanic,
};

use super::{
    defs::{KickoffResult, KICKOFFRESULTSA},
    playutils::PlayUtils,
    CardStreamer, KickoffPlay, PlayResult, ResultType, Yard,
};

pub struct KickPlayContext<'a> {
    utils: PlayUtils<'a>,
    play: KickoffPlay,
}

impl<'a> KickPlayContext<'a> {
    pub fn run_play<'b>(
        state: &'b GameState,
        play: KickoffPlay,
        cards: &'b mut CardStreamer<'b>,
    ) -> PlayResult {
        let mut kpc = KickPlayContext {
            utils: PlayUtils::new(state, cards),
            play,
        };

        kpc.run_kickoff()
    }

    fn run_kickoff(&mut self) -> PlayResult {
        if self.play.onside {
            self.run_onside_kick()
        } else {
            let num = self.utils.get_run_num();
            self.run_result(KICKOFFRESULTSA.get(&num).unwrap())
        }
    }

    fn run_onside_kick(&mut self) -> PlayResult {
        detail!(self.utils, "An onside kick is tried");
        let result_type = match self.utils.get_pass_num() {
            1..=11 => {
                detail!(self.utils, "Recovered by the kicking team");
                ResultType::TurnOver
            }
            _ => {
                detail!(self.utils, "Recovered by the receiving team");
                ResultType::Regular
            }
        };

        self.create_result(result_type, GAMECONSTANTS.onside_kick_line, 0)
    }

    fn run_result(&mut self, result: &KickoffResult) -> PlayResult {
        match result {
            KickoffResult::Touchback => {
                detail!(self.utils, "Touchback");
                mechanic!(
                    self.utils,
                    "Setting ball at {}",
                    GAMECONSTANTS.touchback_line
                );
                self.create_result(ResultType::Regular, GAMECONSTANTS.touchback_line, 0)
            }
            KickoffResult::ColumnB => {
                mechanic!(self.utils, "Going to Column  {}", 'B');
                let num = self.utils.get_run_num();
                self.run_result(KICKOFFRESULTSB.get(&num).unwrap())
            }
            KickoffResult::Return { recipient, line } => {
                let returner = self.play.kr.returners[*recipient as usize].clone();
                self.run_return(&returner, *line)
            }
        }
    }

    fn run_return(&mut self, returner: &Returner, line: Yard) -> PlayResult {
        match returner {
            Returner::SameAs(s) => {
                let real_returner = self.play.kr.returners[*s as usize].clone();
                self.run_return(&real_returner, line)
            },
            Returner::Actual {
                name,
                return_stats,
                asterisk_val,
            } => {
                detail!(
                    self.utils,
                    format!("Kick taken by {} at the {}", name, line)
                );

                let stats = return_stats.get_stat(self.utils.get_run_num() as usize);
                let ret_val = self.get_return_val(stats.asterisk, stats.yards, *asterisk_val);
                detail!(self.utils, format!("It's a {} yard return", ret_val));

                self.create_result(ResultType::Regular, line, ret_val)
            }
        }
    }

    fn get_return_val(&mut self, is_ast: bool, card_val: Yard, ast_val: Yard) -> i32 {
        if !is_ast {
            return card_val;
        }

        match self.utils.get_run_num() {
            1 | 2 => {
                detail!(self.utils, "He breaks away");
                ast_val
            }
            _ => card_val,
        }
    }

    fn create_result(&mut self, result_type: ResultType, line: Yard, result: Yard) -> PlayResult {
        PlayResult {
            result_type: ResultType::Regular,
            result: result,
            final_line: line + result,
            time: 10,
            ..self.utils.result()
        }
    }
}
