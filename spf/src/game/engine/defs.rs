use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::game::{
    engine::{passplay::PassUtils, runplay::RunUtils},
    lineup::{DefensiveBox, OffensiveBox},
    standard_play::{
        OffensivePlayCategory, OffensivePlayInfo, OffensivePlayType, PassMetaData, RunMetaData, DefensiveStrategy,
    },
    stats::{LabeledStat, TwelveStats},
};

use super::Yard;

pub struct TimeTable {
    pub run_play: i32,
    pub run_play_ob: i32,
    pub pass_play_complete: i32,
    pub pass_play_incomplete: i32,
}

pub struct GameConstants {
    pub quarters: i32,
    pub sec_per_quarter: i32,
    pub points_for_td: i32,
    pub points_for_safety: i32,
    pub touchback_line: Yard,
    pub onside_kick_line: Yard,
}

#[derive(Debug, Clone)]
pub enum KickoffResult {
    Touchback,
    ColumnB,
    Return { recipient: i32, line: Yard },
}

pub struct DrawPlayImpact {
    pub run_defense: i32,
    pub pass_defense: i32,
    pub prevent_defense: i32,
    pub blitz: i32,
}

pub struct RunPlayDefenseImpact {
    pub pass_defense: i32,
    pub run_defense_keyed: i32,
    pub run_defense_nokey: i32,
    pub run_defense_wrongkey: i32,
    pub prevent_defense: i32,
    pub blitz: i32,
}

pub struct DefenseConsts {
    pub blitz_min: i32,
    pub blitz_max: i32,
    pub double_cover_defense: i32,
    pub triple_cover_defense: i32,
}

#[derive(Clone)]
pub struct DefenseStrategyRowVals {
    pub row2: i32,
    pub row3: i32,
}

pub struct PassPlayValues {
    pub qk_run_defense: i32,
    pub sh_run_defense: i32,
    pub lg_run_defense: i32,

    pub qk_pass_defense: i32,
    pub sh_pass_defense: i32,
    pub lg_pass_defense: i32,

    pub qk_prevent_defense: i32,
    pub sh_prevent_defense: i32,
    pub lg_prevent_defense: i32,

    pub blitz: i32,

    pub no_defender: i32,

    pub pa_run_defense: i32,
    pub pa_pass_defense: i32,
    pub pa_prevent_defense: i32,
}

lazy_static! {
    pub static ref TIMES: TimeTable = TimeTable {
        run_play: 40,
        run_play_ob: 10,
        pass_play_complete: 40,
        pass_play_incomplete: 10,
    };

    pub static ref GAMECONSTANTS: GameConstants = GameConstants {
        quarters: 4,
        sec_per_quarter: 15*60,
        points_for_td: 6,
        points_for_safety: 2,
        touchback_line: 20,
        onside_kick_line: 50,
    };


    pub static ref DRAW_IMPACT: DrawPlayImpact = DrawPlayImpact {
        run_defense: 2,
        pass_defense: -4,
        prevent_defense: -2,
        blitz: -4
    };

    pub static ref RUN_DEFENSE: RunPlayDefenseImpact = RunPlayDefenseImpact {
        pass_defense: 0,
        run_defense_nokey: 2,
        run_defense_keyed: 4,
        run_defense_wrongkey: 0,
        prevent_defense: 0,
        blitz:0 
    };
    pub static ref SCREEN_DEFENSE: RunPlayDefenseImpact = RunPlayDefenseImpact {
        pass_defense: 0,
        run_defense_nokey: 2,
        run_defense_keyed: 4,
        run_defense_wrongkey: 0,
        prevent_defense: -2,
        blitz: -4
    };

    pub static ref PASS_PLAY_VALUES: PassPlayValues = PassPlayValues {
        qk_run_defense: 0,
        sh_run_defense: 5,
        lg_run_defense: 7,
        qk_pass_defense: -10,
        sh_pass_defense: -5,
        lg_pass_defense: 0,
        qk_prevent_defense: 0,
        sh_prevent_defense: -5,
        lg_prevent_defense: -7,
        blitz: 0,
        no_defender: 5,
        pa_run_defense: 5,
        pa_pass_defense: -5,
        pa_prevent_defense: -10,
    };

    pub static ref PASS_DEFENDERS: HashMap<OffensiveBox, DefensiveBox> = {
        let mut map = HashMap::new();
        map.insert(OffensiveBox::RE, DefensiveBox::BoxN);
        map.insert(OffensiveBox::LE, DefensiveBox::BoxK);
        map.insert(OffensiveBox::FL1, DefensiveBox::BoxO);
        map.insert(OffensiveBox::FL2, DefensiveBox::BoxM);
        map.insert(OffensiveBox::B1, DefensiveBox::BoxF);
        map.insert(OffensiveBox::B2, DefensiveBox::BoxJ);
        map.insert(OffensiveBox::B3, DefensiveBox::BoxH);
        map
    };

    pub static ref INTERCEPTION_TABLE:TwelveStats<LabeledStat<DefensiveBox>> = {

        let int_vals = vec![
            "1: J/N/N/L",
            "2: F/O/M/M",
            "3: C/J/J/M",
            "4: I/I/F/O",
            "5: B/H/I/N",
            "6: G/G/H/K",
            "7: H/F/G/O",
            "8: E/J/O/N",
            "9: D/H/K/K",
            "10: A/F/L/M",
            "11: J/L/N/M",
            "12: F/M/M/L",
        ];

        TwelveStats::create_from_strs(&int_vals, LabeledStat::<DefensiveBox>::curry_create("SC/QK/SH/LG"))
    };
    // TwelveStats::<HashMap::<String, DefensiveBox>>(stats);

    pub static ref INTERCEPTION_RETURN_TABLE:TwelveStats<LabeledStat<i32>> = {

        let int_vals = vec![
            "1: 15/30/100",
            "2: 10/20/50",
            "3: 6/15/30",
            "4: 3/10/20",
            "5: 1/8/15",
            "6: 0/5/10",
            "7: 0/4/8",
            "8: 0/3/6",
            "9: 0/0/4",
            "10: 0/0/2",
            "11: 0/0/0",
            "12: 0/0/0",
        ];

        TwelveStats::create_from_strs(&int_vals, LabeledStat::<i32>::curry_create("DL/LB/DB"))
    };

    pub static ref DEFENSE_CONSTS: DefenseConsts = DefenseConsts{ 
        blitz_min: 2, 
        blitz_max: 5,
        double_cover_defense: -7, 
        triple_cover_defense: -15,
    };


    pub static ref DEFENSE_STRATEGY_LIMITS: HashMap<DefensiveStrategy, Vec<DefenseStrategyRowVals> > = {
        let mut map = HashMap::new();
        let def26 = DefenseStrategyRowVals{ row2: 2, row3: 6 }; 
        let def44 = DefenseStrategyRowVals{ row2: 4, row3: 4 }; 
        let def35 = DefenseStrategyRowVals{ row2: 3, row3: 5 }; 
        map.insert(DefensiveStrategy::DoubleCover, vec![def44, def35]);
        map.insert(DefensiveStrategy::DoubleCoverX2, vec![def26.clone()]);
        map.insert(DefensiveStrategy::TripleCover, vec![def26]);

        map
    };

    pub static ref KICKOFFRESULTSA: HashMap<i32, KickoffResult> = {
        let mut map = HashMap::new();
        map.insert(1, KickoffResult::ColumnB);
        map.insert(2, KickoffResult::Return { recipient: 1, line: 0 });
        map.insert(3, KickoffResult::Return { recipient: 1, line: 1 });
        map.insert(4, KickoffResult::Return { recipient: 2, line: 2 });
        map.insert(5, KickoffResult::Return { recipient: 1, line: 3 });
        map.insert(6, KickoffResult::Return { recipient: 1, line: 4 });
        map.insert(7, KickoffResult::Return { recipient: 2, line: 5 });
        map.insert(8, KickoffResult::Return { recipient: 3, line: 6 });
        map.insert(9, KickoffResult::Return { recipient: 3, line: 7 });
        map.insert(10, KickoffResult::Return { recipient: 2, line: 8 });
        map.insert(11, KickoffResult::Touchback);
        map.insert(12, KickoffResult::ColumnB);
        map
    };

    pub static ref KICKOFFRESULTSB: HashMap<i32, KickoffResult> = {
        let mut map = HashMap::new();
        map.insert(1, KickoffResult::Return { recipient: 1, line: 0 });
        map.insert(2, KickoffResult::Return { recipient: 2, line: 0 });
        map.insert(3, KickoffResult::Return { recipient: 4, line: 1 });
        map.insert(4, KickoffResult::Return { recipient: 2, line: 2 });
        map.insert(5, KickoffResult::Return { recipient: 3, line: 3 });
        map.insert(6, KickoffResult::Return { recipient: 1, line: 4 });
        map.insert(7, KickoffResult::Return { recipient: 1, line: 5 });
        map.insert(8, KickoffResult::Touchback);
        map.insert(9, KickoffResult::Touchback);
        map.insert(10, KickoffResult::Touchback);
        map.insert(11, KickoffResult::Return { recipient: 4, line: 0 });
        map.insert(12, KickoffResult::Return { recipient: 4, line: 0 });
        map
    };


    pub static ref OFFENSIVE_PLAYS_LIST: HashMap<OffensivePlayType, OffensivePlayInfo> = {
        let mut map = HashMap::new();
        map.insert(
            OffensivePlayType::SL,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -100,
                    can_go_ob: true,
                    card_val: RunUtils::get_sl_fac_result,
                }),
                name: "Sweep Left",
                code: "SL",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::SR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -100,
                    can_go_ob: true,
                    card_val: RunUtils::get_sr_fac_result,
                }),
                name: "Sweep Right",
                code: "SR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IL,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -3,
                    can_go_ob: false,
                    card_val: RunUtils::get_il_fac_result,
                }),
                name: "Inside Left",
                code: "IL",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::IR,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -3,
                    can_go_ob: false,
                    card_val: RunUtils::get_ir_fac_result,
                }),
                name: "Inside Right",
                code: "IR",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::ER,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Run(RunMetaData {
                    max_loss: -3,
                    can_go_ob: false,
                    card_val: RunUtils::get_ir_fac_result,
                }),
                name: "End Around",
                code: "ER",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: RunUtils::handle_run_play,
            },
        );
        map.insert(
            OffensivePlayType::QK,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_qk_fac_target,
                    completion_range: PassUtils::get_qk_qb_range,
                    pass_gain: "Q".to_string(),
                }),
                name: "Quick",
                code: "QK",
                allowed_targets: vec![
                    OffensiveBox::B1,
                    OffensiveBox::B2,
                    OffensiveBox::B3,
                    OffensiveBox::RE,
                    OffensiveBox::LE,
                    OffensiveBox::FL1,
                    OffensiveBox::FL2,
                ],
                handler: PassUtils::handle_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::SH,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_sh_fac_target,
                    completion_range: PassUtils::get_sh_qb_range,
                    pass_gain: "S".to_string(),
                }),
                name: "Short",
                code: "SH",
                allowed_targets: vec![
                    OffensiveBox::B1,
                    OffensiveBox::B2,
                    OffensiveBox::B3,
                    OffensiveBox::RE,
                    OffensiveBox::LE,
                    OffensiveBox::FL1,
                    OffensiveBox::FL2,
                ],
                handler: PassUtils::handle_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::LG,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_lg_fac_target,
                    completion_range: PassUtils::get_lg_qb_range,
                    pass_gain: "L".to_string(),
                }),
                name: "Long",
                code: "LG",
                allowed_targets: vec![
                    OffensiveBox::B1,
                    OffensiveBox::B2,
                    OffensiveBox::B3,
                    OffensiveBox::RE,
                    OffensiveBox::LE,
                    OffensiveBox::FL1,
                    OffensiveBox::FL2,
                ],
                handler: PassUtils::handle_pass_play,
            },
        );
        map.insert(
            OffensivePlayType::SC,
            OffensivePlayInfo {
                play_type: OffensivePlayCategory::Pass(PassMetaData {
                    target: PassUtils::get_qk_fac_target,
                    completion_range: PassUtils::get_qk_qb_range,
                    pass_gain: "Q".to_string(),
                }),
                name: "Screen",
                code: "SC",
                allowed_targets: vec![OffensiveBox::B1, OffensiveBox::B2, OffensiveBox::B3],
                handler: PassUtils::handle_pass_play,
            },
        );
        map
    };
}
