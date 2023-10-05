use crate::game::{fac::FacData, GameState};

use super::{PlayLogicState, PlaySetup, OffensivePlayInfo};

pub struct PassUtils {}
impl PassUtils {
    // fn create_run_play<'a>(setup: &'a PlaySetup) ->  Box<dyn PlayRunner2+'a> {
    pub fn create_pass_play(playinfo: &OffensivePlayInfo) -> Box<dyn PlayLogicState> {
        let data = PassPlayData::new();
        // return Box::new(p);
        return Box::new(PassStateStart { data });
    }
}

#[derive(Clone)]
pub struct PassPlayData {
    details: Vec<String>,
}

impl PassPlayData {
    fn new() -> Self {
        return Self {
            details: vec![]
            
        };
    }

}



struct PassStateStart {
    data: PassPlayData,
}

impl PlayLogicState for PassStateStart {
    fn handle_card(
        &self,
        state: &GameState,
        play: &PlaySetup,
        card: &FacData,
    ) -> Box<dyn PlayLogicState> {
        return Box::new(PassStateStart {
            data: self.data.clone(),
        });
    }
    fn get_name(&self) -> &str {
        return "PassStateStart";
    }
}
