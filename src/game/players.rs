use dyn_clone::{clone_trait_object, DynClone};
// use itertools::Itertools;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use spf_macros::{ImplBasePlayer, IsBlocker, IsReceiver};
use std::collections::HashMap;
use strum_macros::Display;

use super::{
    engine::{PassResult, PassRushResult},
    loader::{load_dbs, load_dls, load_lbs, load_ols, load_qbs, load_rbs, load_tes, load_wrs},
    stats::{NumStat, Range, RangedStats, TripleStat, TwelveStats},
};

pub type PassGain = TwelveStats<TripleStat>;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize)]
pub struct TeamID {
    pub name: String,
    pub year: String,
}

impl ToString for TeamID {
    fn to_string(&self) -> String {
        format!("Team: {} ({})", self.name, self.year).to_string()
    }
}

impl TeamID {
    pub fn create_from_str(instr: &str) -> Self {
        let mut fixes: HashMap<&str, &str> = HashMap::new();
        fixes.insert("N.Y.Giants", "N.Y. Giants");
        fixes.insert("NY Giants", "N.Y. Giants");
        fixes.insert("SanDiego", "San Diego");
        fixes.insert("NewOrleans", "New Orleans");
        fixes.insert("SanFran", "San Francisco");
        fixes.insert("St.Louis", "St. Louis");
        fixes.insert("NewEngland", "New England");
        fixes.insert("N.Y.Jets", "N.Y. Jets");
        fixes.insert("NY Jets", "N.Y. Jets");
        fixes.insert("KansasCity", "Kansas City");
        fixes.insert("L.A.Raiders", "L.A. Raiders");
        fixes.insert("L.A.Rams", "L.A. Rams");
        fixes.insert("LA Raiders", "L.A. Raiders");
        fixes.insert("LA Rams", "L.A. Rams");
        fixes.insert("Balimore", "Baltimore");
        fixes.insert("Cincinatti", "Cincinnati");

        let mut vals = instr.trim().splitn(2, ' ');
        let year = vals.next().unwrap_or("1980").to_string();
        let tmpname = vals.next().unwrap_or("Omaha");
        let name = fixes.get(tmpname).unwrap_or(&tmpname).to_string();

        Self { name, year }
    }
}

pub trait Blocker {
    fn get_blocks(&self) -> i32;
}

pub trait Receiver {
    fn get_pass_gain(&self) -> TwelveStats<TripleStat>;
}

pub trait ToBasePlayer {
    fn get_player(&self) -> &dyn BasePlayer;
}

pub trait BasePlayer: Sync + Send + DynClone {
    fn get_id(&self) -> String;
    fn get_team(&self) -> TeamID;
    fn get_name(&self) -> String;
    fn get_json(&self) -> Value;
    fn get_pos(&self) -> Position;
    fn get_full_player(&self) -> Player;
}
clone_trait_object!(BasePlayer);

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct QBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub endurance: char,
    pub quick: RangedStats<PassResult>,
    pub short: RangedStats<PassResult>,
    pub long: RangedStats<PassResult>,
    pub long_run: char,
    pub pass_rush: RangedStats<PassRushResult>,
    pub endurance_rushing: i32,
    pub rushing: TwelveStats<NumStat>,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer, IsBlocker)]
pub struct RBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub rushing: TwelveStats<TripleStat>,
    pub pass_gain: PassGain,
    pub lg: char,
    pub blocks: i32,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer, IsBlocker)]
pub struct WRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub rushing: TwelveStats<TripleStat>,
    pub pass_gain: PassGain,
    pub end: i32,
    pub lg: char,
    pub blocks: i32,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct DBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub pass_def: i32,
    pub pass_rush: i32,
    pub intercepts: Range,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct LBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub tackles: i32,
    pub pass_rush: i32,
    pub pass_def: i32,
    pub intercepts: Range,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct DLStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub tackles: i32,
    pub pass_rush: i32,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer, IsBlocker)]
pub struct TEStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub rushing: TwelveStats<TripleStat>,
    pub pass_gain: PassGain,
    pub blocks: i32,
    pub long_rush: char,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer, IsBlocker)]
pub struct OLStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub blocks: i32,
    pub pass_block: i32,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct KStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct PStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct KRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

#[derive(Debug, Clone, Serialize, ImplBasePlayer)]
pub struct PRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamStats {
    pub team: TeamID,
    pub position: Position,
    pub big_play_home: i32,
    pub big_play_road: i32,
    pub fumbles_lost: Range,
    pub def_adj: i32,
}

#[derive(Debug, Clone, Serialize)]
pub enum Player {
    QB(QBStats),
    RB(RBStats),
    WR(WRStats),
    TE(TEStats),
    DB(DBStats),
    LB(LBStats),
    DL(DLStats),
    OL(OLStats),
    K(KStats),
    KR(KRStats),
    P(PStats),
    PR(PRStats),
}

impl Player {
    // fn get_struct_from_enum<T>(val: Player) -> Option<T> {
    //     match val {
    //         Player::QB(struct1) => Some(struct1 as T),
    //         Player::RB(struct2) => Some(struct2),
    //         _
    //     }
    // }

    // fn extract_type<T>(input: &Player) -> Option<&T> {
    //     if let Player::QB(my_struct1) = input {
    //         // Attempt to cast to the desired type
    //         Some(my_struct1 as &T)
    //     } else if let MyEnum::Val2(my_struct2) = input {
    //         // Attempt to cast to the desired type
    //         Some(my_struct2 as &T)
    //     } else {
    //         None
    //     }
    // }

    pub fn is_qb(val: Player) -> Option<QBStats> {
        if let Player::QB(v) = val {
            return Some(v);
        }
        return None;
    }

    pub fn is_rb(val: Player) -> Option<RBStats> {
        if let Player::RB(v) = val {
            return Some(v);
        }
        return None;
    }
    pub fn is_wr(val: Player) -> Option<WRStats> {
        if let Player::WR(v) = val {
            return Some(v);
        }
        return None;
    }
    pub fn is_te(val: Player) -> Option<TEStats> {
        if let Player::TE(v) = val {
            return Some(v);
        }
        return None;
    }
    pub fn is_ol(val: Player) -> Option<OLStats> {
        if let Player::OL(v) = val {
            return Some(v);
        }
        return None;
    }
    pub fn is_dl(val: Player) -> Option<DLStats> {
        if let Player::DL(v) = val {
            return Some(v);
        }
        return None;
    }

    pub fn is_lb(val: Player) -> Option<LBStats> {
        if let Player::LB(v) = val {
            return Some(v);
        }
        return None;
    }

    pub fn is_db(val: Player) -> Option<DBStats> {
        if let Player::DB(v) = val {
            return Some(v);
        }
        return None;
    }
}

pub struct PlayerUtils {}
impl PlayerUtils {
    pub fn get_blocks(player: Option<&dyn BasePlayer>) -> i32 {
        // let p = player.get_full_player();
        match player {
            Some(p) => match p.get_full_player() {
                Player::OL(ol) => ol.blocks,
                Player::RB(rb) => rb.blocks,
                Player::WR(wr) => wr.blocks,
                Player::TE(te) => te.blocks,
                _ => 0,
            },
            None => 0,
        }
    }

    pub fn get_tackles(player: &dyn BasePlayer) -> i32 {
        let p = player.get_full_player();
        match p {
            Player::DL(dl) => dl.tackles,
            Player::LB(lb) => lb.tackles,
            _ => 0,
        }
    }

    pub fn get_pass_gain(player: Option<&dyn BasePlayer>) -> Option<PassGain> {
        // let p = player.get_full_player();
        match player {
            Some(p) => match p.get_full_player() {
                Player::RB(rb) => Some(rb.pass_gain),
                Player::WR(wr) => Some(wr.pass_gain),
                Player::TE(te) => Some(te.pass_gain),
                _ => None,
            },
            None => None,
        }
    }
}

// struct P {
//     pla: Player,
//     base: dyn BasePlayer,
// }

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Display, PartialEq)]
pub enum Position {
    QB,
    RB,
    WR,
    TE,
    DB,
    LB,
    DL,
    OL,
    K,
    KR,
    P,
    PR,
}

pub enum OLPosition {
    Guard,
    Tackle,
    Center,
}

pub enum DBPosition {
    Cornerback,
    Safety,
}

#[derive(Clone, Serialize)]
pub struct Serializable_Roster {
    team: TeamID,
    players: HashMap<String, Value>,
}

impl Serializable_Roster {
    pub fn from_roster(roster: &Roster) -> Self {
        let players: HashMap<String, Value> =
            (&roster.players)
                .into_iter()
                .fold(HashMap::new(), |mut acc, p| {
                    acc.insert(p.get_id(), p.get_json());
                    acc
                });

        Self {
            team: roster.team_name.clone(),
            players,
        }
    }
}

#[derive(Clone)]
pub struct Roster {
    team_name: TeamID,
    players: Vec<Box<dyn BasePlayer>>,
    // player_by_pos: HashMap<Position, Vec<Player>>,

    // team: TeamStats,
    // qb: Vec<QBStats>,
    // rb: Vec<RBStats>,
    // wr: Vec<WRStats>,
    // te: Vec<TEStats>,
    // db: Vec<DBStats>,
    // lb: Vec<LBStats>,
    // dl: Vec<DLStats>,
    // ol: Vec<OLStats>,
    // k: Vec<KStats>,
    // p: Vec<PStats>,
    // kr: Vec<KRStats>,
    // pr: Vec<PRStats>,
}

// impl Clone for MyStruct {
//     fn clone(&self) -> Self {
//         let mut new_players = Vec::new();
//         for player in self.players.iter() {
//             new_players.push(Box::new(player.clone()));
//         }

//         MyStruct { players: new_players }
//     }
// }

impl Roster {
    fn create_roster(
        team_name: TeamID,
        qb: Vec<QBStats>,
        rb: Vec<RBStats>,
        wr: Vec<WRStats>,
        te: Vec<TEStats>,
        db: Vec<DBStats>,
        lb: Vec<LBStats>,
        dl: Vec<DLStats>,
        ol: Vec<OLStats>,
    ) -> Self {
        {
            let mut players = Vec::<Box<dyn BasePlayer>>::new();
            players.extend(qb.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(rb.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(wr.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(te.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(db.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(lb.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(dl.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(ol.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));

            Self {
                players,
                team_name,
                // qb,
                // rb,
                // wr,
                // te,
                // db,
                // lb,
                // dl,
                // ol,
            }
        }
    }

    pub fn get_player(&self, id: &String) -> Option<&Box<dyn BasePlayer>> {
        return self.players.iter().find(|&x| x.get_id() == *id);
    }

    pub fn get_players(&self, pos: Position) -> Vec<&Box<dyn BasePlayer>> {
        return self
            .players
            .iter()
            .filter(|&x| x.get_pos() == pos)
            .collect::<Vec<&Box<dyn BasePlayer>>>();
    }

    fn print_pos(&self, pos: Position) {
        print!("{}: ", pos);
        for p in self.get_players(pos) {
            print!("{} ({}) ", p.get_name(), p.get_id());
        }
        println!("");
    }

    pub fn print_team(&self) {
        println!("Team: {} ({})", self.team_name.name, self.team_name.year);
        println!("*** Offense ***");
        self.print_pos(Position::QB);
        self.print_pos(Position::RB);
        self.print_pos(Position::WR);
        self.print_pos(Position::TE);
        self.print_pos(Position::OL);
        println!("*** Defense ***");
        self.print_pos(Position::DL);
        self.print_pos(Position::LB);
        self.print_pos(Position::DB);
    }
}

pub struct TeamList {
    pub teams: HashMap<TeamID, Roster>,

    all_qbs: HashMap<String, QBStats>,
    all_rbs: HashMap<String, RBStats>,
    all_wrs: HashMap<String, WRStats>,
    all_tes: HashMap<String, TEStats>,
    all_dbs: HashMap<String, DBStats>,
    all_lbs: HashMap<String, LBStats>,
    all_dls: HashMap<String, DLStats>,
    all_ols: HashMap<String, OLStats>,
}

impl TeamList {
    pub fn get_player(&self, id: &String) -> Option<Box<&dyn BasePlayer>> {
        let pos = id.split('-').next().unwrap_or("");

        let player: Option<Box<&dyn BasePlayer>> = match pos {
            "QB" => TeamList::get_player_from_map(id, &self.all_qbs),
            "RB" => TeamList::get_player_from_map(id, &self.all_rbs),
            "WR" => TeamList::get_player_from_map(id, &self.all_wrs),
            "TE" => TeamList::get_player_from_map(id, &self.all_tes),
            "DL" => TeamList::get_player_from_map(id, &self.all_dls),
            "LB" => TeamList::get_player_from_map(id, &self.all_lbs),
            "DB" => TeamList::get_player_from_map(id, &self.all_dbs),
            _ => None,
        };

        return player;
    }

    fn get_player_from_map<'a, T: BasePlayer>(
        id: &String,
        playermap: &'a HashMap<String, T>,
    ) -> Option<Box<&'a dyn BasePlayer>> {
        if playermap.contains_key(id) {
            return Some(Box::new(playermap.get(id).unwrap()));
        }

        return None;
    }

    // pub fn get_player_json(&self, id: &String) -> String {
    //     let s = id.as_str();
    //     println!("Searching for {}", id);

    //     // if i.starts_with("QB") { TeamList::get_player_json_from_map(&id, &self.all_qbs) }
    //     // else
    //     let pos = id.split('-').next().unwrap_or("");

    //     match pos {
    //         "QB" => TeamList::get_player_json_from_map(id, &self.all_qbs),
    //         "RB" => TeamList::get_player_json_from_map(id, &self.all_rbs),
    //         "WR" => TeamList::get_player_json_from_map(id, &self.all_wrs),
    //         "TE" => TeamList::get_player_json_from_map(id, &self.all_tes),
    //         "DL" => TeamList::get_player_json_from_map(id, &self.all_dls),
    //         _ => "{\"err\": \"Invalid ID\"}".to_string(),
    //     }
    // }

    // fn get_player_json_from_map<T: BasePlayer>(
    //     id: &String,
    //     playermap: &HashMap<String, T>,
    // ) -> String {
    //     match playermap.get(id) {
    //         Some(p) => {
    //             println!("A Match");
    //             p.get_json()
    //         }
    //         _ => {
    //             println!("No Match");
    //             "{\"err\": \"No such player\"}".to_string()
    //         }
    //     }
    // }

    pub fn create_teams(dir: &str) -> Self {
        let (qbs, all_qbs) = TeamList::disperse_players(load_qbs(format!("{}/83QB.txt", dir)));
        let (rbs, all_rbs) = TeamList::disperse_players(load_rbs(format!("{}/83RB.txt", dir)));
        let (wrs, all_wrs) = TeamList::disperse_players(load_wrs(format!("{}/83WR.txt", dir)));
        let (tes, all_tes) = TeamList::disperse_players(load_tes(format!("{}/83TE.txt", dir)));

        let (ols, all_ols) = TeamList::disperse_players(load_ols(format!("{}/83OL.txt", dir)));
        let (dls, all_dls) = TeamList::disperse_players(load_dls(format!("{}/83DL.txt", dir)));
        let (lbs, all_lbs) = TeamList::disperse_players(load_lbs(format!("{}/83LB.txt", dir)));
        let (dbs, all_dbs) = TeamList::disperse_players(load_dbs(format!("{}/83DB.txt", dir)));

        // code to validate team_ids
        // for t in qbs.keys() {
        //     println!("|{:?}|", t);
        //     let r = ols.get(t);
        //     match r {
        //         Some(value) => println!("Value associated with key '{:?}': {}", t, value.len()),
        //         None => println!("Key not found."),
        //     }
        // }

        let mut teams: HashMap<TeamID, Roster> = HashMap::new();
        for t in qbs.keys() {
            teams.insert(
                t.clone(),
                Roster::create_roster(
                    t.clone(),
                    // team: (),
                    qbs.get(t).unwrap().to_vec(),
                    rbs.get(t).unwrap().to_vec(),
                    wrs.get(t).unwrap().to_vec(),
                    tes.get(t).unwrap().to_vec(),
                    dbs.get(t).unwrap().to_vec(),
                    lbs.get(t).unwrap().to_vec(),
                    dls.get(t).unwrap().to_vec(),
                    ols.get(t).unwrap().to_vec(), // k: (),
                                                  // p: (),
                                                  // kr: (),
                                                  // pr: (),
                ),
            );
        }

        // println!("All DLs************");
        // println!("{:?}", all_dls);

        Self {
            teams,
            all_qbs,
            all_rbs,
            all_wrs,
            all_tes,
            all_ols,
            all_dls,
            all_lbs,
            all_dbs,
        }
    }

    fn disperse_players<T>(players: Vec<T>) -> (HashMap<TeamID, Vec<T>>, HashMap<String, T>)
    where
        T: BasePlayer + Clone,
    {
        let mut team_map: HashMap<TeamID, Vec<T>> = HashMap::new();
        let mut id_map: HashMap<String, T> = HashMap::new();

        for ele in players {
            let e1 = ele.clone();
            team_map
                .entry(ele.get_team().clone())
                .or_insert(Vec::new())
                .push(e1);
            let e2 = ele.clone();
            // let s = ele.get_id().clone();
            id_map.insert(e2.get_id(), e2);
        }

        return (team_map, id_map);
    }

    // fn disperse_players2<T>(players: Vec<T>) -> HashMap<TeamID, Vec<&'static T>>
    // where
    //     T: TeamGroup,
    // {
    //     let grouped_map: HashMap<TeamID, Vec<&T>> =
    //         players.iter().into_group_map_by(|item| item.get_team());

    //     return grouped_map;
    // }
}
