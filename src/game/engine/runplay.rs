use std::cmp::{max, min};

use crate::game::{
    fac::{FacData, RunResult, RunResultActual},
    players::{BasePlayer, Player, PlayerUtils, RBStats},
    stats::{self, NumStat},
    Game, GameState,
};
use itertools::fold;
use option_ext::OptionExt;

use super::{
    CardStreamer, DefensivePlay, OffenseCall, OffensivePlayInfo, OffensivePlayType, PlayLogicState,
    PlayResult, PlaySetup, RunMetaData,
};

#[derive(Clone)]
pub struct RunPlayData {
    details: Vec<String>,
    modifier: i32,
    yardage: i32,
    // result: Option<PlayResult>,
    ob: bool,
    md: RunMetaData,
}

impl RunPlayData {
    fn new(playinfo: &OffensivePlayInfo) -> Self {
        return Self {
            details: vec![],
            modifier: 0,
            yardage: 0,
            ob: false,
            md: playinfo.play_type.as_run().unwrap().clone(),
        };
    }
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
        return start_run(&mut context);
    }

    pub fn get_sl_fac_result<'a>(card: &'a FacData) -> &'a RunResult {
        &card.sl
    }
    pub fn get_sr_fac_result<'a>(card: &'a FacData) -> &'a RunResult {
        &card.sr
    }
    pub fn get_il_fac_result<'a>(card: &'a FacData) -> &'a RunResult {
        &card.il
    }
    pub fn get_ir_fac_result<'a>(card: &'a FacData) -> &'a RunResult {
        &card.ir
    }
}

struct RunContext<'a> {
    state: &'a GameState,
    play: &'a PlaySetup<'a>,
    cards: &'a mut CardStreamer<'a>,
    data: RunPlayData,
}

fn get_run_block(ctxt: &mut RunContext, player: &dyn BasePlayer) -> PlayResult {
    // println!("RunStateStart::get_run_block");

    ctxt.data
        .details
        .push(format!("Handoff to {}", player.get_name()));

    // play.offense_metadata.play_type

    let card = &ctxt.cards.get_fac();
    // let res = RunPlayData::get_fac_result(&play.offense_call.play_type, card);
    let res = (ctxt.data.md.card_val)(card);
    match res {
        RunResult::Actual(actual) => calculate_run_yardage(ctxt, actual),
        RunResult::Break => {
            ctxt.data.details.push("It's a breakaway".to_string());
            let rb = get_rb_stats(ctxt.play);
            ctxt.data.yardage = get_lg_yardage(rb.lg);
            return finalize_yardage(ctxt);
        }
    }
}

fn calculate_run_yardage(ctxt: &mut RunContext, actual: &RunResultActual) -> PlayResult {
    let (modifier, mut logs) = calculate_run_yardage_modifier(actual, ctxt.play);
    ctxt.data.modifier = modifier;
    ctxt.data.details.append(&mut logs);

    let rb = get_rb_stats(ctxt.play);

    let (modifier, mut logs) = get_run_modifier(ctxt.play);
    ctxt.data.details.append(&mut logs);

    let card = ctxt.cards.get_fac();
    let run_num = min(card.run_num.num + modifier, 12);

    ctxt.data
        .details
        .push(format!("{} carries the ball", rb.get_name()));
    let stat = get_rush_stat(&rb, run_num);
    match stat {
        stats::NumStat::Sg | stats::NumStat::Lg => {
            ctxt.data
                .details
                .push("He gets out for short gain".to_string());
            calculate_sg_yardage(ctxt);
        }
        stats::NumStat::Val(num) => {
            ctxt.data.yardage = *num;
            ctxt.data.ob = card.run_num.ob;
        }
    }
    return finalize_yardage(ctxt);
}

fn calculate_sg_yardage(ctxt: &mut RunContext) {
    let card = ctxt.cards.get_fac();
    let rn = card.run_num.num;
    ctxt.data.yardage = rn + 5;
    ctxt.data.ob = card.run_num.ob;
}

fn start_run(ctxt: &mut RunContext) -> PlayResult {
    let player = ctxt
        .play
        .offense
        .get_player_in_pos(&ctxt.play.offense_call.target);

    match player {
        Some(p) => {
            return get_run_block(ctxt, p);
        }
        None => {
            // this should really never happen
            ctxt.data.details.push(format!(
                "Handoff to nobody in position {:?}",
                ctxt.play.offense_call.target
            ));
            return handle_bad_play(ctxt);
        }
    };
}

fn calculate_run_yardage_modifier(
    result: &RunResultActual,
    play: &PlaySetup,
) -> (i32, Vec<String>) {
    let mut logs: Vec<String> = Vec::new();
    logs.push(format!(
        "It's {:?} against {:?}",
        result.offensive_boxes, result.defensive_boxes
    ));

    let tackles: i32 = result
        .defensive_boxes
        .iter()
        .flat_map(|s| play.defense.get_players_in_pos(s))
        // .flatten()
        .fold(0, |acc, ele| acc + PlayerUtils::get_tackles(ele));

    let blocks = result
        .offensive_boxes
        .iter()
        .map(|s| play.offense.get_player_in_pos(s))
        // .flatten()
        .fold(0, |acc, ele| acc + PlayerUtils::get_blocks(ele));

    let modifier = match tackles.cmp(&blocks) {
        std::cmp::Ordering::Less => {
            logs.push(format!(
                "Block spring the runner for an extra {} yards",
                blocks
            ));
            blocks
        }
        std::cmp::Ordering::Equal => {
            logs.push("Runner gets by blocks and tackles".to_string());
            0
        }
        std::cmp::Ordering::Greater => {
            logs.push(format!("Big tackle to save {} yards", tackles));
            -tackles
        }
    };

    return (modifier, logs);
}

fn handle_bad_play(ctxt: &RunContext) -> PlayResult {
    return PlayResult {
        result: 0,
        time: 10,
        details: ctxt.data.details.clone(),
        mechanic: vec![],
        extra: None,
        cards: ctxt.cards.get_results(),
    };
    // return Box::new(RunStateEnd { data: data });
}

fn finalize_yardage(ctxt: &mut RunContext) -> PlayResult {
    let data: &mut RunPlayData = &mut ctxt.data;
    let result = max(data.yardage + data.modifier, data.md.max_loss);

    let mut time = 40;

    if data.ob && data.md.can_go_ob {
        data.details.push("Play ends out of bounds".to_string());
        time = 10;
    }

    data.details.push(format!("Gain of {} yards", data.yardage));
    return PlayResult {
        result,
        time,
        details: data.details.clone(),
        mechanic: vec![],
        extra: None,
        cards: ctxt.cards.get_results(),
    };
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

fn get_run_modifier(play: &PlaySetup) -> (i32, Vec<String>) {
    let mut modifier = 0;
    let mut logs: Vec<String> = Vec::new();
    if play.defense_call.defense_type == DefensivePlay::RunDefense {
        logs.push("The run defense focuses on the run".to_string());
        modifier = 2;
        if let Some(pos) = &play.defense_call.key {
            if *pos == play.offense_call.target {
                logs.push(" and they key on the right back".to_string());
                modifier += 2;
            } else {
                logs.push("But they focus on the wrong back".to_string());
                modifier -= 2;
            }
        }
    }
    return (modifier, logs);
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
