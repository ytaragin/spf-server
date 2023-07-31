use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::players::{
    DBStats, DLStats, LBStats, OLStats, Player, Position, QBStats, RBStats, Roster, TEStats,
    WRStats,
};
use lazy_static::lazy_static;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Down {
    First,
    Second,
    Third,
    Fourth,
}
pub trait Validatable {
    fn validate(&self, play: &Play) -> Result<(), String>;
}

#[derive(Debug, Clone, Serialize)]
enum EndPlayer {
    TE(TEStats),
    WR(WRStats),
    RB(RBStats),
}
impl EndPlayer {
    fn gen_from_player(inplayer: Player) -> Option<Self> {
        return match inplayer {
            Player::TE(terec) => Some(EndPlayer::TE(terec.clone())),
            Player::WR(wrrec) => Some(EndPlayer::WR(wrrec.clone())),
            Player::RB(rbrec) => Some(EndPlayer::RB(rbrec.clone())),
            _ => None,
        };
    }
}

#[derive(Debug, Clone, Serialize)]
enum FlankerPlayer {
    WR(WRStats),
    RB(RBStats),
}

impl FlankerPlayer {
    fn gen_from_player(inplayer: Player) -> Option<Self> {
        return match inplayer {
            Player::WR(wrrec) => Some(FlankerPlayer::WR(wrrec.clone())),
            Player::RB(rbrec) => Some(FlankerPlayer::RB(rbrec.clone())),
            _ => None,
        };
    }
}

pub struct IDBasedOffensiveLineup {
    LE_split: Option<String>,
    LE_tight: Option<String>,
    RE_split: Option<String>,
    RE_tight: Option<String>,
    FL1: Option<String>,
    FL2: Option<String>,
    QB: Option<String>,
    B1: Option<String>,
    B2: Option<String>,
    B3: Option<String>,
    LT: Option<String>,
    LG: Option<String>,
    C: Option<String>,
    RG: Option<String>,
    RT: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OffensiveLineup {
    le_split: Option<EndPlayer>,
    le_tight: Option<EndPlayer>,
    re_split: Option<EndPlayer>,
    re_tight: Option<EndPlayer>,
    fl1: Option<FlankerPlayer>,
    fl2: Option<FlankerPlayer>,
    qb: QBStats,
    b1: Option<RBStats>,
    b2: Option<RBStats>,
    b3: Option<RBStats>,
    lt: OLStats,
    lg: OLStats,
    c: OLStats,
    rg: OLStats,
    rt: OLStats,
}

impl OffensiveLineup {
    pub fn create_lineup(id_lineup: IDBasedOffensiveLineup, team: Roster) -> Result<Self, String> {
        let qb = LineupUtilities::get_player_from_id(id_lineup.QB, "QB", &team, Player::is_qb)?;

        let le_split = Some(LineupUtilities::get_player_from_id(
            id_lineup.LE_split,
            "LE_split",
            &team,
            EndPlayer::gen_from_player,
        )?);
        let le_tight = Some(LineupUtilities::get_player_from_id(
            id_lineup.LE_tight,
            "LE_tight",
            &team,
            EndPlayer::gen_from_player,
        )?);

        let re_split = Some(LineupUtilities::get_player_from_id(
            id_lineup.RE_split,
            "RE_split",
            &team,
            EndPlayer::gen_from_player,
        )?);
        let re_tight = Some(LineupUtilities::get_player_from_id(
            id_lineup.RE_tight,
            "RE_tight",
            &team,
            EndPlayer::gen_from_player,
        )?);

        let fl1 = Some(LineupUtilities::get_player_from_id(
            id_lineup.FL1,
            "Flanker",
            &team,
            FlankerPlayer::gen_from_player,
        )?);
        let fl2 = Some(LineupUtilities::get_player_from_id(
            id_lineup.FL2,
            "Flanker",
            &team,
            FlankerPlayer::gen_from_player,
        )?);

        let b1 = Some(LineupUtilities::get_player_from_id(
            id_lineup.B1,
            "B1",
            &team,
            Player::is_rb,
        )?);
        let b2 = Some(LineupUtilities::get_player_from_id(
            id_lineup.B2,
            "B2",
            &team,
            Player::is_rb,
        )?);
        let b3 = Some(LineupUtilities::get_player_from_id(
            id_lineup.B3,
            "B3",
            &team,
            Player::is_rb,
        )?);

        let lt = LineupUtilities::get_player_from_id(id_lineup.LT, "LT", &team, Player::is_ol)?;
        let lg = LineupUtilities::get_player_from_id(id_lineup.LG, "LG", &team, Player::is_ol)?;
        let c = LineupUtilities::get_player_from_id(id_lineup.C, "C", &team, Player::is_ol)?;
        let rg = LineupUtilities::get_player_from_id(id_lineup.RG, "RG", &team, Player::is_ol)?;
        let rt = LineupUtilities::get_player_from_id(id_lineup.RT, "RT", &team, Player::is_ol)?;

        return Ok(Self {
            le_split,
            le_tight,
            re_split,
            re_tight,
            fl1,
            fl2,
            qb,
            b1,
            b2,
            b3,
            lt,
            lg,
            c,
            rg,
            rt,
        });
    }

    fn is_legal_lineup(&self) -> Result<(), String> {
        let b_count = LineupUtilities::count_spots(vec![&self.b1, &self.b2, &self.b3]);
        if b_count <= 0 || b_count > 3 {
            return Err("Invalid number of Backs".to_string());
        }

        let left_end_count = LineupUtilities::count_spots(vec![&self.le_split, &self.le_tight]);
        LineupUtilities::validate_count(left_end_count, 1, 1, "Only one Left End")?;

        let right_end_count = LineupUtilities::count_spots(vec![&self.re_split, &self.re_tight]);
        LineupUtilities::validate_count(right_end_count, 1, 1, "Only one Right End")?;

        let flanker_count = LineupUtilities::count_spots(vec![&self.fl1, &self.fl2]);
        let remaining_spots = 3 - b_count;
        LineupUtilities::validate_count(
            flanker_count,
            remaining_spots,
            remaining_spots,
            "Invalid number of Flankers",
        )?;

        return Ok(());
    }
}

impl Validatable for OffensiveLineup {
    fn validate(&self, play: &Play) -> Result<(), String> {
        self.is_legal_lineup()?;

        return Ok(());
    }
}

pub struct IDBasedDefensiveLineup {
    box_a: Vec<String>,
    box_b: Vec<String>,
    box_c: Vec<String>,
    box_d: Vec<String>,
    box_e: Vec<String>,
    box_f: Option<String>,
    box_g: Option<String>,
    box_h: Option<String>,
    box_i: Option<String>,
    box_j: Option<String>,
    box_k: Option<String>,
    box_l: Vec<String>,
    box_m: Option<String>,
    box_n: Option<String>,
    box_o: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
enum Row1Player {
    DL(DLStats),
    LB(LBStats),
}
impl Row1Player {
    fn gen_from_player(inplayer: Player) -> Option<Self> {
        return match inplayer {
            Player::DL(dlrec) => Some(Self::DL(dlrec.clone())),
            Player::LB(lbrec) => Some(Self::LB(lbrec.clone())),
            _ => None,
        };
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DefensiveLineup {
    box_a: Vec<Row1Player>,
    box_b: Vec<Row1Player>,
    box_c: Vec<Row1Player>,
    box_d: Vec<Row1Player>,
    box_e: Vec<Row1Player>,
    box_f: Option<LBStats>,
    box_g: Option<LBStats>,
    box_h: Option<LBStats>,
    box_i: Option<LBStats>,
    box_j: Option<LBStats>,
    box_k: Option<DBStats>,
    box_l: Vec<DBStats>,
    box_m: Option<DBStats>,
    box_n: Option<DBStats>,
    box_o: Option<DBStats>,
}

impl DefensiveLineup {
    pub fn create_lineup(id_lineup: IDBasedDefensiveLineup, team: Roster) -> Result<Self, String> {
        let box_a = LineupUtilities::transform_vector(
            &id_lineup.box_a,
            "box_a",
            &team,
            Row1Player::gen_from_player,
        )?;
        let box_b = LineupUtilities::transform_vector(
            &id_lineup.box_b,
            "box_b",
            &team,
            Row1Player::gen_from_player,
        )?;
        let box_c = LineupUtilities::transform_vector(
            &id_lineup.box_c,
            "box_c",
            &team,
            Row1Player::gen_from_player,
        )?;
        let box_d = LineupUtilities::transform_vector(
            &id_lineup.box_d,
            "box_d",
            &team,
            Row1Player::gen_from_player,
        )?;
        let box_e = LineupUtilities::transform_vector(
            &id_lineup.box_e,
            "box_e",
            &team,
            Row1Player::gen_from_player,
        )?;

        let box_f = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_f,
            "box_f",
            &team,
            Player::is_lb,
        )?);
        let box_g = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_g,
            "box_g",
            &team,
            Player::is_lb,
        )?);
        let box_h = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_h,
            "box_h",
            &team,
            Player::is_lb,
        )?);
        let box_i = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_i,
            "box_i",
            &team,
            Player::is_lb,
        )?);
        let box_j = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_j,
            "box_j",
            &team,
            Player::is_lb,
        )?);

        let box_k = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_k,
            "box_k",
            &team,
            Player::is_db,
        )?);
        let box_m = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_m,
            "box_m",
            &team,
            Player::is_db,
        )?);
        let box_n = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_n,
            "box_n",
            &team,
            Player::is_db,
        )?);
        let box_o = Some(LineupUtilities::get_player_from_id(
            id_lineup.box_o,
            "box_o",
            &team,
            Player::is_db,
        )?);
        let box_l =
            LineupUtilities::transform_vector(&id_lineup.box_l, "box_l", &team, Player::is_db)?;

        return Ok(Self {
            box_a,
            box_b,
            box_c,
            box_d,
            box_e,
            box_f,
            box_g,
            box_h,
            box_i,
            box_j,
            box_k,
            box_l,
            box_m,
            box_n,
            box_o,
        });
    }

    fn is_legal_lineup(&self) -> Result<(), String> {
        let first_row_spots = LineupUtilities::count_array_spots(
            vec![
                &self.box_a,
                &self.box_b,
                &self.box_c,
                &self.box_d,
                &self.box_e,
            ],
            3,
            "Only 3 allowed in a First Row Box",
        )?;

        LineupUtilities::validate_count(first_row_spots, 3, 10, "Need between 3-10 in First Row");

        let row2_spots = LineupUtilities::count_spots(vec![
            &self.box_f,
            &self.box_g,
            &self.box_h,
            &self.box_i,
            &self.box_j,
        ]);

        let mut remaining_spots = 11 - (row2_spots + first_row_spots);
        if remaining_spots < 0 {
            return Err("Too many Lineman and Linebackers".to_string());
        }

        let non_box_l_db_count =
            LineupUtilities::count_spots(vec![&self.box_k, &self.box_m, &self.box_n, &self.box_o]);

        let remaining_spots = remaining_spots - non_box_l_db_count;
        if remaining_spots < 0 {
            return Err("Too many in the Secondary".to_string());
        }

        if remaining_spots > 0 && non_box_l_db_count < 4 {
            return Err("Can only put in Box L after the other 4 Row 3 spots are full".to_string());
        }

        return Ok(());
    }
}

impl Validatable for DefensiveLineup {
    fn validate(&self, play: &Play) -> Result<(), String> {
        self.is_legal_lineup()?;

        
        return Ok(());
    }
}

struct LineupUtilities {}
impl LineupUtilities {
    fn get_player_from_id<T, F>(
        id_opt: Option<String>,
        pos_str: &str,
        team: &Roster,
        transform: F,
    ) -> Result<T, String>
    where
        F: Fn(Player) -> Option<T>,
    {
        let id = id_opt.ok_or(format!("Missing {}", pos_str))?;

        let p = team.get_player(id).ok_or(format!("No Such {}", pos_str))?;
        let t =
            transform(p.get_full_player()).ok_or(format!("Not a valid type for {}", pos_str))?;
        return Ok(t);
    }

    fn transform_vector<T, F>(
        id_vecs: &Vec<String>,
        pos_str: &str,
        team: &Roster,
        transform: F,
    ) -> Result<Vec<T>, String>
    where
        F: Fn(Player) -> Option<T>,
    {
        let v = id_vecs
            .iter()
            .map(|item| LineupUtilities::get_player_from_id(Some(*item), pos_str, team, transform))
            .collect::<Result<Vec<T>, String>>();

        return v;
    }

    fn validate_count(actual: i32, low: i32, high: i32, msg: &str) -> Result<(), String> {
        if actual < low || actual > high {
            return Err(msg.to_string());
        }
        return Ok(());
    }

    fn count_spots<T>(pos: Vec<&Option<T>>) -> i32 {
        pos.iter().map(|t| if t.is_none() { 1 } else { 0 }).sum()
    }

    fn count_array_spots<T>(pos: Vec<&Vec<T>>, max_per: i32, msg: &str) -> Result<i32, String> {
        let total = pos.iter().try_fold(0, |acc, item| {
            let count = item.len() as i32;
            if count > max_per {
                return Err(msg.to_string());
            }
            Ok(acc + count)
        })?;

        return Ok(total);
    }
}

pub type Yard = i32;

#[derive(Debug, Clone, Copy)]
pub enum OffensivePlayTypes {
    Run,
    Pass,
}

#[derive(Debug, Clone, Copy)]
pub struct OffensivePlay {
    pub play_type: OffensivePlayTypes,
    pub name: &'static str,
    pub code: &'static str,
}

impl Validatable for OffensivePlay {
    fn validate(&self, play: &Play) -> Result<(), String> {
        return Ok(());
    }
}

lazy_static! {
    static ref OffensivePlays: HashMap<&'static str, OffensivePlay> = {
        let mut map = HashMap::new();
        map.insert(
            "SL",
            OffensivePlay {
                play_type: OffensivePlayTypes::Run,
                name: "Sweep Left",
                code: "SL",
            },
        );
        map.insert(
            "SR",
            OffensivePlay {
                play_type: OffensivePlayTypes::Run,
                name: "Sweep Right",
                code: "SL",
            },
        );
        map
    };
}

const OffensivePlays2: [OffensivePlay; 9] = [
    OffensivePlay {
        play_type: OffensivePlayTypes::Run,
        name: "Sweep Left",
        code: "SL",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Run,
        name: "Sweep Right",
        code: "SR",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Run,
        name: "Inside Left",
        code: "IL",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Run,
        name: "Inside Left",
        code: "IR",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Run,
        name: "End Around",
        code: "ER",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Pass,
        name: "Quick",
        code: "QK",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Pass,
        name: "Short",
        code: "SH",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Pass,
        name: "Long",
        code: "LG",
    },
    OffensivePlay {
        play_type: OffensivePlayTypes::Pass,
        name: "Screen",
        code: "SC",
    },
];

#[derive(Debug, Clone, Copy)]
pub enum DefensivePlay {
    RunDefense,
    PassDefense,
    PreventDefense,
    Blitz,
}

impl Validatable for DefensivePlay {
    fn validate(&self, play: &Play) -> Result<(), String> {
        return Ok(());
    }
}

#[derive(Debug, Default, Clone)]
pub struct Play {
    pub offense: Option<OffensiveLineup>,
    pub offense_play: Option<OffensivePlay>,
    pub defense: Option<DefensiveLineup>,
    pub defense_play: Option<DefensivePlay>,
    pub result: Option<PlayResult>,
}

impl Play {
    pub fn new() -> Self {
        return Self {
            ..Default::default()
        };
    }

    fn play_ready(&self) -> bool {
        return true;
    }

    pub fn run_play(&self) -> Option<PlayResult> {
        return None;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PlayResult {
    pub result: Yard,
}
