use crate::game::{
    fac::{FacData, PassTarget},
    lineup::OffensiveBox,
    players::{Player, QBStats},
    stats::RangedStats,
    GameState,
};

use super::{
    CardStreamer, OffensivePlayInfo, PassMetaData, PassResult, PlayLogicState, PlayResult,
    PlaySetup, TIMES,
};

pub struct PassUtils {}
impl PassUtils {
    // fn create_run_play<'a>(setup: &'a PlaySetup) ->  Box<dyn PlayRunner2+'a> {
    pub fn create_pass_play(playinfo: &OffensivePlayInfo) -> Box<dyn PlayLogicState> {
        let data = PassPlayData::new(playinfo);
        // return Box::new(p);
        return Box::new(PassStateDetermineTarget { data });
    }

    // pub fn handle_pass_play<'a>(
    //     state: &GameState,
    //     play: &PlaySetup,
    //     cards: &mut CardStreamer,
    pub fn handle_pass_play<'a>(
        state: &'a GameState,
        play: &'a PlaySetup<'a>,
        cards: &'a mut CardStreamer<'a>,
    ) -> PlayResult {
        let mut data = PassPlayData::new(play.offense_metadata);
        // let context = RunContext {
        //     state,
        //     play,
        //     cards,
        //     data: &mut data,
        // };
        // return start_run(&context);
        return 
            PlayResult {
                result: 0,
                time: 0,
                details: vec![],
                extra: None,
                cards: cards.get_results(),
            };
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
    target: OffensiveBox,
    md: PassMetaData,
    result: Option<PlayResult>,
}

impl PassPlayData {
    fn new(playinfo: &OffensivePlayInfo) -> Self {
        return Self {
            details: vec![],
            target: OffensiveBox::QB,
            md: playinfo.play_type.as_pass().unwrap().clone(),
            result: None,
        };
    }
}

fn incomplete_pass(mut data: PassPlayData) -> Box<dyn PlayLogicState> {
    data.details.push("The pass falls incomplete".to_string());
    // data.result = Some(PlayResult {
    //     result: 0,
    //     time: TIMES.pass_play_incomplete,
    //     details: data.details.clone(),
    //     extra: None,

    // });

    return Box::new(PassStateEnd { data });
}

struct PassStateDetermineTarget {
    data: PassPlayData,
}

impl PlayLogicState for PassStateDetermineTarget {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        let mut data = self.data.clone();

        let target = (self.data.md.target)(card);

        match target {
            PassTarget::Orig => {
                data.target = play.offense_call.target;
            }
            PassTarget::PassRush => return Box::new(PassRushState { data }),
            PassTarget::Actual(target) => {
                data.target = *target;
                data.details.push(format!(
                    "The QB adjusts and throws it towards the {:?}",
                    target
                ));
                if play.offense.get_player_in_pos(&data.target).is_none() {
                    data.details.push("But no one is there".to_string());
                    return incomplete_pass(data);
                }
            }
        }

        return Box::new(PassCheckResultState { data });
    }
    fn get_name(&self) -> &str {
        return "PassStateDetermineTarget";
    }
}

#[derive(Clone)]
struct PassRushState {
    data: PassPlayData,
}

impl PlayLogicState for PassRushState {
    fn get_name(&self) -> &str {
        return "PassRushState";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        return Box::new(PassStateEnd {
            data: self.data.clone(),
        });
    }
    fn get_result(&self) -> Option<PlayResult> {
        return self.data.result.clone();
    }
}

#[derive(Clone)]
struct PassCheckResultState {
    data: PassPlayData,
}

impl PlayLogicState for PassCheckResultState {
    fn get_name(&self) -> &str {
        return "PassCheckResultState";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        let pass_num = card.pass_num;
        let qb = Player::is_qb(
            play.offense
                .get_player_in_pos(&OffensiveBox::QB)
                .unwrap()
                .get_full_player(),
        )
        .unwrap();

        let range = (self.data.md.complete)(&qb);
        let res = range.get_category(pass_num);

        return Box::new(PassStateEnd {
            data: self.data.clone(),
        });
    }
    fn get_result(&self) -> Option<PlayResult> {
        return self.data.result.clone();
    }
}

#[derive(Clone)]
struct PassStateEnd {
    data: PassPlayData,
}

impl PlayLogicState for PassStateEnd {
    fn get_name(&self) -> &str {
        return "PassStateEnd";
    }

    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        return Box::new(PassStateEnd {
            data: self.data.clone(),
        });
    }
    fn get_result(&self) -> Option<PlayResult> {
        return self.data.result.clone();
    }
}
