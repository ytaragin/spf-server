// use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum_macros::Display;

use super::{
    loader::{load_dbs, load_dls, load_lbs, load_ols, load_qbs, load_rbs, load_tes, load_wrs},
    stats::{NumStat, Range, RangedStats, TripleStat, TwelveStats},
};

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

pub trait BasePlayer {
    fn get_id(&self) -> String;
    fn get_team(&self) -> TeamID;
    fn get_name(&self) -> String;
    fn get_json(&self) -> String;
    fn get_pos(&self) -> Position;
    fn get_full_player(&self) -> Player;
}

#[derive(Debug, Clone, Serialize)]
pub struct QBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub endurance: char,
    pub quick: RangedStats,
    pub short: RangedStats,
    pub long: RangedStats,
    pub long_run: char,
    pub pass_rush: RangedStats,
    pub endurance_rushing: i32,
    pub rushing: TwelveStats<NumStat>,
}

impl BasePlayer for QBStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::QB(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub rushing: TwelveStats<TripleStat>,
    pub pass_gain: TwelveStats<TripleStat>,
    pub lg: char,
    pub blocks: i32,
}

impl BasePlayer for RBStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::RB(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub rushing: TwelveStats<TripleStat>,
    pub pass_gain: TwelveStats<TripleStat>,
    pub end: i32,
    pub lg: char,
    pub blocks: i32,
}

impl BasePlayer for WRStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::WR(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub pass_def: i32,
    pub pass_rush: i32,
    pub intercepts: Range,
}

impl BasePlayer for DBStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::DB(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
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

impl BasePlayer for LBStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::LB(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DLStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub tackles: i32,
    pub pass_rush: i32,
}

impl BasePlayer for DLStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::DL(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TEStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub rushing: TwelveStats<TripleStat>,
    pub pass_gain: TwelveStats<TripleStat>,
    pub blocks: i32,
    pub long_rush: char,
}

impl BasePlayer for TEStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::TE(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OLStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub blocks: i32,
    pub pass_block: i32,
}

impl BasePlayer for OLStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::OL(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

impl BasePlayer for KStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::K(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

impl BasePlayer for PStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::P(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

impl BasePlayer for KRStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::KR(*self);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
}

impl BasePlayer for PRStats {
    fn get_team(&self) -> TeamID {
        self.team.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pos(&self) -> Position {
        return self.position;
    }
    fn get_json(&self) -> String {
        let res = serde_json::to_string(self);
        match res {
            Ok(js) => js,
            Err(_) => "".to_string(),
        }
    }
    fn get_full_player(&self) -> Player {
        return Player::PR(*self);
    }
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
// impl BasePlayer for TeamStats {
//     fn get_team(&self) -> TeamID {
//         self.team.clone()
//     }
//     fn get_id(&self) -> String {
//         self.team.to_string()
//     }
//     fn get_name(&self) -> String {
//         self.team.name.clone()
//     }
//     fn get_json(&self) -> String {
//         let res = serde_json::to_string(self);
//         match res {
//             Ok(js) => js,
//             Err(_) => "".to_string(),
//         }
//     }
//     fn get_full_player(&self) -> Player {
//         return Player::T(*self);
//     }
// }

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
    pub fn is_ol(val: Player) -> Option<OLStats> {
        if let Player::OL(v) = val {
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

struct P {
    pla: Player,
    base: dyn BasePlayer,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Display)]
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

pub struct Roster {
    team_name: TeamID,
    players: Vec<Box<dyn BasePlayer>>,
    // player_by_pos: HashMap<Position, Vec<Player>>,

    // team: TeamStats,
    qb: Vec<QBStats>,
    rb: Vec<RBStats>,
    wr: Vec<WRStats>,
    te: Vec<TEStats>,
    db: Vec<DBStats>,
    lb: Vec<LBStats>,
    dl: Vec<DLStats>,
    ol: Vec<OLStats>,
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
                qb,
                rb,
                wr,
                te,
                db,
                lb,
                dl,
                ol,
            }
        }
    }

    pub fn get_player(&self, id: String) -> Option<&Box<dyn BasePlayer>> {
        return self.players.iter().find(|&x| x.get_id() == id);
    }

    fn print_pos<T>(pos: &str, players: &Vec<T>)
    where
        T: BasePlayer,
    {
        print!("{}: ", pos);
        for p in players {
            print!("{} ({}) ", p.get_name(), p.get_id());
        }
        println!("");
    }

    pub fn print_team(&self) {
        println!("Team: {} ({})", self.team_name.name, self.team_name.year);
        println!("*** Offense ***");
        Roster::print_pos("QB", &self.qb);
        Roster::print_pos("RB", &self.rb);
        Roster::print_pos("WR", &self.wr);
        Roster::print_pos("TE", &self.te);
        Roster::print_pos("OL", &self.ol);
        println!("*** Defense ***");
        Roster::print_pos("DL", &self.dl);
        Roster::print_pos("LB", &self.lb);
        Roster::print_pos("DB", &self.db);
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
