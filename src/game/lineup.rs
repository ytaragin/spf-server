use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use spf_macros::ToBasePlayer;

use super::players::{
    BasePlayer, DBStats, DLStats, KRStats, KStats, LBStats, OLStats, Player, QBStats, RBStats,
    Roster, TEStats, ToBasePlayer, WRStats,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, Eq, Hash)]
pub enum OffensiveBox {
    QB,
    B1,
    B2,
    B3,
    RE,
    LE,
    FL1,
    FL2,
    LT,
    LG,
    C,
    RG,
    RT,
}

impl OffensiveBox {
    
    pub fn get_receiver_spots() -> Vec<OffensiveBox> {
        return vec![Self::FL1, Self::FL2, Self::LE, Self::RE, Self::B1, Self::B2, Self::B3];
    }
}

impl FromStr for OffensiveBox {
    type Err = String; // you can use any type that implements std::error::Error

    fn from_str(input: &str) -> Result<OffensiveBox, String> {
        let val = input.to_lowercase();
        // println!("Validating {}", val);
        match val.as_str() {
            "qb" => Ok(OffensiveBox::QB),
            "bk" | "bk1" | "b1" => Ok(OffensiveBox::B1),
            "bk2" | "b2" => Ok(OffensiveBox::B2),
            "bk3" | "b3" => Ok(OffensiveBox::B3),
            "re" => Ok(OffensiveBox::RE),
            "le" => Ok(OffensiveBox::LE),
            "fl" | "fl1 " => Ok(OffensiveBox::FL1),
            "fl2" => Ok(OffensiveBox::FL2),
            "lt" => Ok(OffensiveBox::LT),
            "lg" => Ok(OffensiveBox::LG),
            "c" | "cn" => Ok(OffensiveBox::C),
            "rg" => Ok(OffensiveBox::RG),
            "rt" => Ok(OffensiveBox::RT),
            _ => Err(format!("Invalid OffensiveBox: {}", input)), // return an error if the input does not match any variant
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub enum DefensiveBox {
    BoxA,
    BoxB,
    BoxC,
    BoxD,
    BoxE,
    BoxF,
    BoxG,
    BoxH,
    BoxI,
    BoxJ,
    BoxK,
    BoxL,
    BoxM,
    BoxN,
    BoxO,
}

impl FromStr for DefensiveBox {
    type Err = String; // you can use any type that implements std::error::Error

    fn from_str(input: &str) -> Result<DefensiveBox, String> {
        match input.to_lowercase().as_str() {
            "a" => Ok(DefensiveBox::BoxA),
            "b" => Ok(DefensiveBox::BoxB),
            "c" => Ok(DefensiveBox::BoxC),
            "d" => Ok(DefensiveBox::BoxD),
            "e" => Ok(DefensiveBox::BoxE),
            "f" => Ok(DefensiveBox::BoxF),
            "g" => Ok(DefensiveBox::BoxG),
            "h" => Ok(DefensiveBox::BoxH),
            "i" => Ok(DefensiveBox::BoxI),
            "j" => Ok(DefensiveBox::BoxJ),
            "k" => Ok(DefensiveBox::BoxK),
            "l" => Ok(DefensiveBox::BoxL),
            "m" => Ok(DefensiveBox::BoxM),
            "n" => Ok(DefensiveBox::BoxN),
            "o" => Ok(DefensiveBox::BoxO),
            _ => Err(format!("Invalid DefensiveBox: {}", input)), // return an error if the input does not match any variant
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToBasePlayer)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToBasePlayer)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardIDOffenseLineup {
    le: Option<String>,
    re: Option<String>,
    fl1: Option<String>,
    fl2: Option<String>,
    qb: Option<String>,
    b1: Option<String>,
    b2: Option<String>,
    b3: Option<String>,
    lt: Option<String>,
    lg: Option<String>,
    c: Option<String>,
    rg: Option<String>,
    rt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StandardOffensiveLineup {
    le: Option<EndPlayer>,
    re: Option<EndPlayer>,
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

impl StandardOffensiveLineup {
    pub fn create_lineup(
        id_lineup: &StandardIDOffenseLineup,
        team: &Roster,
        ) -> Result<Self, String> {
        let qb =
            LineupUtilities::get_player_from_id_or_err(&id_lineup.qb, "QB", &team, Player::is_qb)?;

        let le = LineupUtilities::get_option_player_from_id(
            &id_lineup.le,
            "LE",
            &team,
            EndPlayer::gen_from_player,
            )?;

        let re = LineupUtilities::get_option_player_from_id(
            &id_lineup.re,
            "RE",
            &team,
            EndPlayer::gen_from_player,
            )?;

        let fl1 = LineupUtilities::get_option_player_from_id(
            &id_lineup.fl1,
            "Flanker",
            &team,
            FlankerPlayer::gen_from_player,
            )?;
        let fl2 = LineupUtilities::get_option_player_from_id(
            &id_lineup.fl2,
            "Flanker",
            &team,
            FlankerPlayer::gen_from_player,
            )?;

        let b1 =
            LineupUtilities::get_option_player_from_id(&id_lineup.b1, "B1", &team, Player::is_rb)?;
        let b2 =
            LineupUtilities::get_option_player_from_id(&id_lineup.b2, "B2", &team, Player::is_rb)?;
        let b3 =
            LineupUtilities::get_option_player_from_id(&id_lineup.b3, "B3", &team, Player::is_rb)?;

        let lt =
            LineupUtilities::get_player_from_id_or_err(&id_lineup.lt, "LT", &team, Player::is_ol)?;
        let lg =
            LineupUtilities::get_player_from_id_or_err(&id_lineup.lg, "LG", &team, Player::is_ol)?;
        let c =
            LineupUtilities::get_player_from_id_or_err(&id_lineup.c, "C", &team, Player::is_ol)?;
        let rg =
            LineupUtilities::get_player_from_id_or_err(&id_lineup.rg, "RG", &team, Player::is_ol)?;
        let rt =
            LineupUtilities::get_player_from_id_or_err(&id_lineup.rt, "RT", &team, Player::is_ol)?;

        return Ok(Self {
            le,
            re,
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

    pub fn convert_to_id_lineup(&self) -> StandardIDOffenseLineup {
        return StandardIDOffenseLineup {
            le: LineupUtilities::get_id_from_player(&self.le),
            re: LineupUtilities::get_id_from_player(&self.re),
            fl1: LineupUtilities::get_id_from_player(&self.fl1),
            fl2: LineupUtilities::get_id_from_player(&self.fl2),
            qb: Some(self.qb.get_id()),
            b1: LineupUtilities::get_id_from_player(&self.b1),
            b2: LineupUtilities::get_id_from_player(&self.b2),
            b3: LineupUtilities::get_id_from_player(&self.b3),
            lt: Some(self.lt.get_id()),
            lg: Some(self.lg.get_id()),
            c: Some(self.c.get_id()),
            rg: Some(self.rg.get_id()),
            rt: Some(self.rt.get_id()),
        };
    }

    pub fn is_legal_lineup(&self) -> Result<(), String> {
        let b_count = LineupUtilities::count_spots(vec![&self.b1, &self.b2, &self.b3]);
        println!("Backs: {}", b_count);
        LineupUtilities::validate_count(b_count, 1, 3, "Invalid number of Backs")?;

        let left_end_count = LineupUtilities::count_spots(vec![&self.le]);
        LineupUtilities::validate_count(left_end_count, 1, 1, "Only one Left End")?;

        let right_end_count = LineupUtilities::count_spots(vec![&self.re]);
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

    pub fn get_player_in_pos(&self, spot: &OffensiveBox) -> Option<&dyn BasePlayer> {
        // Use match to compare the field name with each possible option
        match spot {
            OffensiveBox::QB => Some(&self.qb),
            OffensiveBox::B1 => LineupUtilities::get_player_from_option(&self.b1),
            OffensiveBox::B2 => LineupUtilities::get_player_from_option(&self.b2),
            OffensiveBox::B3 => LineupUtilities::get_player_from_option(&self.b3),
            OffensiveBox::RE => LineupUtilities::get_player_from_option(&self.re),
            OffensiveBox::LE => LineupUtilities::get_player_from_option(&self.le),
            OffensiveBox::FL1 => LineupUtilities::get_player_from_option(&self.fl1),
            OffensiveBox::FL2 => LineupUtilities::get_player_from_option(&self.fl2),
            OffensiveBox::LT => Some(self.lt.get_player()),
            OffensiveBox::LG => Some(self.lg.get_player()),
            OffensiveBox::C => Some(self.c.get_player()),
            OffensiveBox::RG => Some(self.rg.get_player()),
            OffensiveBox::RT => Some(self.rt.get_player()),
        }
    }

}

// impl Validatable for OffensiveLineup {
//     fn validate(&self, play: &Play) -> Result<(), String> {
//         self.is_legal_lineup()?;

//         return Ok(());
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardIDDefenseLineup {
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

pub enum DefensiveRow {
    Row1,
    Row2,
    Row3
}


#[derive(Debug, Clone, Serialize, Deserialize, ToBasePlayer)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardDefensiveLineup {
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

impl StandardDefensiveLineup {
    pub fn create_lineup(
        id_lineup: &StandardIDDefenseLineup,
        team: &Roster,
        ) -> Result<Self, String> {
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

        let box_f = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_f,
            "box_f",
            &team,
            Player::is_lb,
            )?;
        let box_g = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_g,
            "box_g",
            &team,
            Player::is_lb,
            )?;
        let box_h = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_h,
            "box_h",
            &team,
            Player::is_lb,
            )?;
        let box_i = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_i,
            "box_i",
            &team,
            Player::is_lb,
            )?;
        let box_j = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_j,
            "box_j",
            &team,
            Player::is_lb,
            )?;

        let box_k = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_k,
            "box_k",
            &team,
            Player::is_db,
            )?;
        let box_m = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_m,
            "box_m",
            &team,
            Player::is_db,
            )?;
        let box_n = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_n,
            "box_n",
            &team,
            Player::is_db,
            )?;
        let box_o = LineupUtilities::get_option_player_from_id(
            &id_lineup.box_o,
            "box_o",
            &team,
            Player::is_db,
            )?;
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

    pub fn convert_to_id_lineup(&self) -> StandardIDDefenseLineup {
        return StandardIDDefenseLineup {
            box_a: LineupUtilities::get_ids_for_vec(&self.box_a),
            box_b: LineupUtilities::get_ids_for_vec(&self.box_b),
            box_c: LineupUtilities::get_ids_for_vec(&self.box_c),
            box_d: LineupUtilities::get_ids_for_vec(&self.box_d),
            box_e: LineupUtilities::get_ids_for_vec(&self.box_e),
            box_f: LineupUtilities::get_id_from_player(&self.box_f),
            box_g: LineupUtilities::get_id_from_player(&self.box_g),
            box_h: LineupUtilities::get_id_from_player(&self.box_h),
            box_i: LineupUtilities::get_id_from_player(&self.box_i),
            box_j: LineupUtilities::get_id_from_player(&self.box_j),
            box_k: LineupUtilities::get_id_from_player(&self.box_k),
            box_l: LineupUtilities::get_ids_for_vec(&self.box_l),
            box_m: LineupUtilities::get_id_from_player(&self.box_m),
            box_n: LineupUtilities::get_id_from_player(&self.box_n),
            box_o: LineupUtilities::get_id_from_player(&self.box_o),
        };
    }

    pub fn get_count_in_row(&self, row:DefensiveRow) -> i32 {
        match row {
            DefensiveRow::Row1 => LineupUtilities::count_array_spots(
                vec![
                &self.box_a,
                &self.box_b,
                &self.box_c,
                &self.box_d,
                &self.box_e,
                ],3, "").unwrap_or(0),
            DefensiveRow::Row2 => LineupUtilities::count_spots(vec![
                                                      &self.box_f,
                                                      &self.box_g,
                                                      &self.box_h,
                                                      &self.box_i,
                                                      &self.box_j,
        ]),
            DefensiveRow::Row3 =>  LineupUtilities::count_spots(vec![&self.box_k, &self.box_m, &self.box_n, &self.box_o])+self.box_l.len() as i32,
        }
    }
    pub fn is_legal_lineup(&self) -> Result<(), String> {
        let row1_spots = LineupUtilities::count_array_spots(
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

        LineupUtilities::validate_count(row1_spots, 3, 10, "Need between 3-10 in First Row")?;

        let row2_spots = LineupUtilities::count_spots(vec![
                                                      &self.box_f,
                                                      &self.box_g,
                                                      &self.box_h,
                                                      &self.box_i,
                                                      &self.box_j,
        ]);

        let remaining_row3_spots = 11 - (row2_spots + row1_spots);

        if remaining_row3_spots < 0 {
            return Err("Too many Lineman and Linebackers".to_string());
        }

        let non_box_l_db_count =
            LineupUtilities::count_spots(vec![&self.box_k, &self.box_m, &self.box_n, &self.box_o]);
        let l_count = self.box_l.len() as i32;
        if l_count > 0 && non_box_l_db_count < 4 {
            return Err("Can only put in Box L after the other 4 Row 3 spots are full".to_string());
        }

        LineupUtilities::validate_count(
            non_box_l_db_count + l_count,
            remaining_row3_spots,
            remaining_row3_spots,
            "Improper secondary size",
            )?;

        return Ok(());
    }

    pub fn find_player(&self, id: &String) -> Option<DefensiveBox> {
        if LineupUtilities::is_player_in_vec(&self.box_a, id) {
            return Some(DefensiveBox::BoxA);
        }
        if LineupUtilities::is_player_in_vec(&self.box_b, id) {
            return Some(DefensiveBox::BoxB);
        }
        if LineupUtilities::is_player_in_vec(&self.box_c, id) {
            return Some(DefensiveBox::BoxC);
        }
        if LineupUtilities::is_player_in_vec(&self.box_d, id) {
            return Some(DefensiveBox::BoxD);
        }
        if LineupUtilities::is_player_in_vec(&self.box_e, id) {
            return Some(DefensiveBox::BoxE);
        }

        if LineupUtilities::is_same_player(&self.box_f, id) {
            return Some(DefensiveBox::BoxF);
        }
        if LineupUtilities::is_same_player(&self.box_g, id) {
            return Some(DefensiveBox::BoxG);
        }
        if LineupUtilities::is_same_player(&self.box_h, id) {
            return Some(DefensiveBox::BoxH);
        }
        if LineupUtilities::is_same_player(&self.box_i, id) {
            return Some(DefensiveBox::BoxI);
        }
        if LineupUtilities::is_same_player(&self.box_j, id) {
            return Some(DefensiveBox::BoxJ);
        }
        if LineupUtilities::is_same_player(&self.box_k, id) {
            return Some(DefensiveBox::BoxK);
        }
        if LineupUtilities::is_player_in_vec(&self.box_l, id) {
            return Some(DefensiveBox::BoxL);
        }
        if LineupUtilities::is_same_player(&self.box_m, id) {
            return Some(DefensiveBox::BoxM);
        }
        if LineupUtilities::is_same_player(&self.box_n, id) {
            return Some(DefensiveBox::BoxN);
        }
        if LineupUtilities::is_same_player(&self.box_o, id) {
            return Some(DefensiveBox::BoxO);
        }

        return None;
    }

    pub fn get_players_in_pos(&self, spot: &DefensiveBox) -> Vec<&dyn BasePlayer> {
        // Use match to compare the field name with each possible option
        match spot {
            DefensiveBox::BoxA => LineupUtilities::convert_vec_to_base_player(&self.box_a),
            DefensiveBox::BoxB => LineupUtilities::convert_vec_to_base_player(&self.box_b),
            DefensiveBox::BoxC => LineupUtilities::convert_vec_to_base_player(&self.box_c),
            DefensiveBox::BoxD => LineupUtilities::convert_vec_to_base_player(&self.box_d),
            DefensiveBox::BoxE => LineupUtilities::convert_vec_to_base_player(&self.box_e),
            DefensiveBox::BoxF => LineupUtilities::convert_option_to_vec(&self.box_f),
            DefensiveBox::BoxG => LineupUtilities::convert_option_to_vec(&self.box_g),
            DefensiveBox::BoxH => LineupUtilities::convert_option_to_vec(&self.box_h),
            DefensiveBox::BoxI => LineupUtilities::convert_option_to_vec(&self.box_i),
            DefensiveBox::BoxJ => LineupUtilities::convert_option_to_vec(&self.box_j),
            DefensiveBox::BoxK => LineupUtilities::convert_option_to_vec(&self.box_k),
            DefensiveBox::BoxL => LineupUtilities::convert_vec_to_base_player(&self.box_l),
            DefensiveBox::BoxM => LineupUtilities::convert_option_to_vec(&self.box_m),
            DefensiveBox::BoxN => LineupUtilities::convert_option_to_vec(&self.box_n),
            DefensiveBox::BoxO => LineupUtilities::convert_option_to_vec(&self.box_o),
        }
    }

    fn filter_players_from_vec<T: ToBasePlayer + Clone>(
        ids: &Vec<String>,
        players: &Vec<T>,
        ) -> Vec<T> {
        let v = players
            .iter()
            .filter(|p| !ids.contains(&p.get_player().get_id()))
            .map(|p| (*p).clone())
            .collect::<Vec<T>>();

        v
    }

    fn filter_players_from_pos<T: ToBasePlayer + Clone>(
        ids: &Vec<String>,
        player: &Option<T>,
        ) -> Option<T> {
        let v = player
            .clone()
            .and_then(|p| match ids.contains(&p.get_player().get_id()) {
                true => None,
                false => Some(p),
            });

        v
    }

    pub fn filter_players(&self, ids: &Vec<String>) -> Self {
        return Self {
            box_a: Self::filter_players_from_vec(ids, &self.box_a),
            box_b: Self::filter_players_from_vec(ids, &self.box_b),
            box_c: Self::filter_players_from_vec(ids, &self.box_c),
            box_d: Self::filter_players_from_vec(ids, &self.box_d),
            box_e: Self::filter_players_from_vec(ids, &self.box_e),
            box_f: Self::filter_players_from_pos(ids, &self.box_f),
            box_g: Self::filter_players_from_pos(ids, &self.box_g),
            box_h: Self::filter_players_from_pos(ids, &self.box_h),
            box_i: Self::filter_players_from_pos(ids, &self.box_i),
            box_j: Self::filter_players_from_pos(ids, &self.box_j),
            box_k: Self::filter_players_from_pos(ids, &self.box_k),
            box_l: Self::filter_players_from_vec(ids, &self.box_l),
            box_m: Self::filter_players_from_pos(ids, &self.box_m),
            box_n: Self::filter_players_from_pos(ids, &self.box_n),
            box_o: Self::filter_players_from_pos(ids, &self.box_o),
        };
    }
}

// impl Validatable for DefensiveLineup {
//     fn validate(&self, play: &Play) -> Result<(), String> {
//         self.is_legal_lineup()?;

//         return Ok(());
//     }
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KickoffIDOffenseLineup {
    pub k: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KickoffIDDefenseLineup {
    pub kr: String,
}

struct LineupUtilities {}
impl LineupUtilities {
    fn get_ids_for_vec<T: ToBasePlayer>(players: &Vec<T>) -> Vec<String> {
        players
            .into_iter()
            .map(|p| p.get_player().get_id())
            .collect_vec()
    }

    fn get_id_from_player<T: ToBasePlayer>(player: &Option<T>) -> Option<String> {
        match player {
            None => None,
            Some(p) => Some(p.get_player().get_id()),
        }
    }

    fn get_player_from_option<'a, T: ToBasePlayer>(
        player: &'a Option<T>,
        ) -> Option<&'a dyn BasePlayer> {
        match player {
            None => None,
            Some(p) => Some(p.get_player()),
        }
    }

    fn is_same_player<T: ToBasePlayer>(player: &Option<T>, id: &String) -> bool {
        match player {
            None => false,
            Some(p) => (p.get_player().get_id() == *id),
        }
    }

    fn is_player_in_vec<T: ToBasePlayer>(players: &Vec<T>, id: &String) -> bool {
        let ids = LineupUtilities::get_ids_for_vec(players);
        return ids.contains(id);
    }

    fn get_player_from_id_or_err<T, F>(
        id_opt: &Option<String>,
        pos_str: &str,
        team: &Roster,
        transform: F,
        ) -> Result<T, String>
        where
            F: Fn(Player) -> Option<T>,
        {
            // let id = &id_opt.ok_or(format!("Missing {}", pos_str))?;

            let id = match id_opt {
                None => return Err(format!("Missing {}", pos_str)),
                Some(val) => val,
            };

            let p = team.get_player(id).ok_or(format!("No Such {}", pos_str))?;
            let t =
                transform(p.get_full_player()).ok_or(format!("Not a valid type for {}", pos_str))?;
            return Ok(t);
        }

    fn get_option_player_from_id<T, F>(
        id_opt: &Option<String>,
        pos_str: &str,
        team: &Roster,
        transform: F,
        ) -> Result<Option<T>, String>
        where
            F: Fn(Player) -> Option<T>,
        {
            // let id = &id_opt.ok_or(format!("Missing {}", pos_str))?;

            let id = match id_opt {
                None => return Ok(None),
                Some(val) => val,
            };

            let p = team
                .get_player(id)
                .ok_or(format!("No Such {} from {}", pos_str, id))?;
            let t =
                transform(p.get_full_player()).ok_or(format!("Not a valid type for {}", pos_str))?;
            return Ok(Some(t));
        }

    fn get_player_from_valid_id<T, F>(
        id_opt: Option<String>,
        pos_str: &str,
        team: &Roster,
        transform: F,
        ) -> Result<T, String>
        where
            F: Fn(Player) -> Option<T>,
        {
            let id = &id_opt.ok_or(format!("Missing {}", pos_str))?;

            let p = team
                .get_player(id)
                .ok_or(format!("No Such {} with id {}", pos_str, id))?;
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
                .map(|item| {
                    LineupUtilities::get_player_from_id_or_err(
                        &Some(item.clone()),
                        pos_str,
                        team,
                        &transform,
                        )
                })
            .collect::<Result<Vec<T>, String>>();

            return v;
        }

    fn convert_vec_to_base_player<'a, T: ToBasePlayer>(v: &'a Vec<T>) -> Vec<&'a dyn BasePlayer> {
        v.iter().map(ToBasePlayer::get_player).collect()
    }

    fn convert_option_to_vec<'a, T: ToBasePlayer>(o: &'a Option<T>) -> Vec<&'a dyn BasePlayer> {
        match o {
            Some(p) => vec![p.get_player()],
            None => vec![],
        }
    }

    fn validate_count(actual: i32, low: i32, high: i32, msg: &str) -> Result<(), String> {
        if actual < low || actual > high {
            let m = format!("{}: Expected {}-{} but was {}", msg, low, high, actual);
            return Err(m);
        }
        return Ok(());
    }

    fn count_spots<T: std::fmt::Debug>(pos: Vec<&Option<T>>) -> i32 {
        // println!("Checking: {:?}", pos);
        pos.iter().map(|t| if t.is_some() { 1 } else { 0 }).sum()
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
