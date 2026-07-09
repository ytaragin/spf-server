use dyn_clone::{clone_trait_object, DynClone};
// use itertools::Itertools;
use erased_serde::serialize_trait_object;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use spf_macros::{ImplBasePlayer, IsBlocker};
use std::collections::HashMap;
use strum_macros::Display;
use utoipa::ToSchema;

use crate::{
    loader::{
        load_dbs, load_dls, load_krs, load_ks, load_lbs, load_ols, load_qbs, load_rbs, load_tes,
        load_wrs,
    },
    shiftable::{PassResult, PassRushResult},
    stats::{NumStat, Range, RangedStats, TripleStat, TwelveStats},
};

pub type PassGain = TwelveStats<TripleStat>;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize, Deserialize, ToSchema)]
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
        fixes.insert("New York G", "N.Y. Giants");
        fixes.insert("SanDiego", "San Diego");
        fixes.insert("NewOrleans", "New Orleans");
        fixes.insert("SanFran", "San Francisco");
        fixes.insert("St.Louis", "St. Louis");
        fixes.insert("NewEngland", "New England");
        fixes.insert("N.Y.Jets", "N.Y. Jets");
        fixes.insert("New York J", "N.Y. Jets");
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

pub trait BasePlayer: Sync + Send + DynClone + erased_serde::Serialize {
    fn get_id(&self) -> String;
    fn get_team(&self) -> TeamID;
    fn get_name(&self) -> String;
    fn get_json(&self) -> Value;
    fn get_pos(&self) -> Position;
    fn get_full_player(&self) -> Player;
}
clone_trait_object!(BasePlayer);
serialize_trait_object!(BasePlayer);

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer, IsBlocker)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer, IsBlocker)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
pub struct DBStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub pass_def: i32,
    pub pass_rush: i32,
    pub intercepts: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
pub struct DLStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub tackles: i32,
    pub pass_rush: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer, IsBlocker)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer, IsBlocker)]
pub struct OLStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub blocks: i32,
    pub pass_block: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
pub struct KStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,

    pub field_goals: RangedStats<Range>,
    pub over_fifty: Range,
    pub extra_points: Range,
    pub longest_fg: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PuntResultDetails {
    FairCatch,
    Returner(i32),
}
// pub enum P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PuntResult {
    Special,
    Actual {
        yards: i32,
        target: PuntResultDetails,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
pub struct PStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,

    pub punt_results: TwelveStats<PuntResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct ReturnStat {
    pub yards: i32,
    pub fumble: bool,
    pub asterisk: bool,
}
impl ReturnStat {
    pub fn build_from_str(instr: &str) -> Self {
        let re = regex::Regex::new(r"(\d{1,2})([*f]?)").ok().unwrap();

        // println!("Attempting |{}|", instr);

        let caps = match re.captures(instr) {
            Some(c) => c,
            None => {
                return Self {
                    yards: 0,
                    fumble: false,
                    asterisk: false,
                }
            }
        };

        let yards = caps
            .get(1)
            .map_or("0", |m| m.as_str())
            .parse::<i32>()
            .unwrap_or(0); // "42"
        let suffix = caps.get(2).map_or("", |m| m.as_str()); // "f"

        return Self {
            yards,
            fumble: suffix == "f",
            asterisk: suffix == "*",
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Returner {
    SameAs(i32),
    Actual {
        name: String,
        return_stats: TwelveStats<ReturnStat>,
        asterisk_val: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
pub struct KRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,

    pub returners: Vec<Returner>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ImplBasePlayer)]
pub struct PRStats {
    pub team: TeamID,
    pub name: String,
    pub id: String,

    pub position: Position,
    pub returners: Vec<Returner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStats {
    pub team: TeamID,
    pub position: Position,
    pub big_play_home: i32,
    pub big_play_road: i32,
    pub fumbles_lost: Range,
    pub def_adj: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn is_k(val: Player) -> Option<KStats> {
        if let Player::K(v) = val {
            return Some(v);
        }
        return None;
    }

    pub fn is_kr(val: Player) -> Option<KRStats> {
        if let Player::KR(v) = val {
            return Some(v);
        }
        return None;
    }

    pub fn is_p(val: Player) -> Option<PStats> {
        if let Player::P(v) = val {
            return Some(v);
        }
        return None;
    }

    pub fn is_pr(val: Player) -> Option<PRStats> {
        if let Player::PR(v) = val {
            return Some(v);
        }
        return None;
    }

    /// Convert a tagged `Player` into a boxed trait object for the in-memory roster.
    ///
    /// This is the inverse of `BasePlayer::get_full_player` and is the key step that
    /// lets the persistent (deserializable) representation be rebuilt into the
    /// runtime model, which stores `Box<dyn BasePlayer>`.
    pub fn into_base_player(self) -> Box<dyn BasePlayer> {
        match self {
            Player::QB(s) => Box::new(s),
            Player::RB(s) => Box::new(s),
            Player::WR(s) => Box::new(s),
            Player::TE(s) => Box::new(s),
            Player::DB(s) => Box::new(s),
            Player::LB(s) => Box::new(s),
            Player::DL(s) => Box::new(s),
            Player::OL(s) => Box::new(s),
            Player::K(s) => Box::new(s),
            Player::KR(s) => Box::new(s),
            Player::P(s) => Box::new(s),
            Player::PR(s) => Box::new(s),
        }
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

    pub fn get_pass_defense(player: &dyn BasePlayer) -> i32 {
        match player.get_full_player() {
            Player::DB(db) => db.pass_def,
            Player::LB(lb) => lb.pass_def,
            _ => 0,
        }
    }

    pub fn get_pass_block(player: &dyn BasePlayer) -> i32 {
        match player.get_full_player() {
            Player::OL(ol) => ol.pass_block,
            _ => 0,
        }
    }

    pub fn get_pass_rush(player: &dyn BasePlayer) -> i32 {
        match player.get_full_player() {
            Player::DL(dl) => dl.pass_rush,
            Player::LB(lb) => lb.pass_rush,
            Player::DB(db) => db.pass_rush,
            _ => 0,
        }
    }
}

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

#[derive(Clone, Serialize, ToSchema)]
#[allow(non_camel_case_types)]
pub struct Serializable_Roster {
    team: TeamID,
    #[schema(value_type = Object)]
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

#[derive(Clone, Serialize)]
pub struct Roster {
    team_name: TeamID,

    #[serde(bound(serialize = "Vec<Box<dyn BasePlayer>>: Serialize"))]
    players: Vec<Box<dyn BasePlayer>>,
}

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
        k: Vec<KStats>,
        kr: Vec<KRStats>,
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
            players.extend(k.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));
            players.extend(kr.into_iter().map(|s| Box::new(s) as Box<dyn BasePlayer>));

            Self { players, team_name }
        }
    }

    /// Build a roster from a flat list of tagged `Player`s (the persistent form).
    ///
    /// Used when loading the JSON data model back into memory.
    pub fn from_players(team_name: TeamID, players: Vec<Player>) -> Self {
        Self {
            team_name,
            players: players.into_iter().map(|p| p.into_base_player()).collect(),
        }
    }

    pub fn get_team_name(&self) -> &TeamID {
        &self.team_name
    }

    pub fn get_player(&self, id: &String) -> Option<&Box<dyn BasePlayer>> {
        return self.players.iter().find(|&x| x.get_id() == *id);
    }

    pub fn get_all_players(&self) -> &Vec<Box<dyn BasePlayer>> {
        &self.players
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
        println!("*** Special Teams ***");
        self.print_pos(Position::K);
        self.print_pos(Position::KR);
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
    // FIXME: latent bug - `TeamList::get_player` has no "OL" match arm, so this map is never
    // queried (hence "never read"). Currently masked because `get_player` has no callers; live
    // OL lookups go through `Roster::get_player`. Add `"OL" => get_player_from_map(id, &self.all_ols)`.
    #[allow(dead_code)]
    all_ols: HashMap<String, OLStats>,
    all_ks: HashMap<String, KStats>,
    all_krs: HashMap<String, KRStats>,
}

impl TeamList {
    pub fn get_team(&self, id: &TeamID) -> Option<&Roster> {
        self.teams.get(id)
    }

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
            "K" => TeamList::get_player_from_map(id, &self.all_ks),
            "KR" => TeamList::get_player_from_map(id, &self.all_krs),
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

    pub fn create_teams(dir: &str) -> Self {
        let (qbs, all_qbs) = TeamList::disperse_players(load_qbs(format!("{}/83QB.txt", dir)));
        let (rbs, all_rbs) = TeamList::disperse_players(load_rbs(format!("{}/83RB.txt", dir)));
        let (wrs, all_wrs) = TeamList::disperse_players(load_wrs(format!("{}/83WR.txt", dir)));
        let (tes, all_tes) = TeamList::disperse_players(load_tes(format!("{}/83TE.txt", dir)));

        let (ols, all_ols) = TeamList::disperse_players(load_ols(format!("{}/83OL.txt", dir)));
        let (dls, all_dls) = TeamList::disperse_players(load_dls(format!("{}/83DL.txt", dir)));
        let (lbs, all_lbs) = TeamList::disperse_players(load_lbs(format!("{}/83LB.txt", dir)));
        let (dbs, all_dbs) = TeamList::disperse_players(load_dbs(format!("{}/83DB.txt", dir)));
        let (ks, all_ks) = TeamList::disperse_players(load_ks(format!("{}/83K.txt", dir)));
        let (krs, all_krs) = TeamList::disperse_players(load_krs(format!("{}/83KR.txt", dir)));

        let mut teams: HashMap<TeamID, Roster> = HashMap::new();
        for t in qbs.keys() {
            println!("Load Team {}", t.name);

            teams.insert(
                t.clone(),
                Roster::create_roster(
                    t.clone(),
                    qbs.get(t).unwrap().to_vec(),
                    rbs.get(t).unwrap().to_vec(),
                    wrs.get(t).unwrap().to_vec(),
                    tes.get(t).unwrap().to_vec(),
                    dbs.get(t).unwrap().to_vec(),
                    lbs.get(t).unwrap().to_vec(),
                    dls.get(t).unwrap().to_vec(),
                    ols.get(t).unwrap().to_vec(),
                    ks.get(t).unwrap().to_vec(),
                    krs.get(t).unwrap().to_vec(),
                ),
            );
        }

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
            all_ks,
            all_krs,
        }
    }

    /// Rebuild a `TeamList` from already-parsed rosters (the persistent-load path).
    ///
    /// Reconstructs the per-position `all_*` id lookup maps by walking every
    /// player in every roster and matching on the concrete `Player` variant.
    pub fn from_rosters(rosters: Vec<Roster>) -> Self {
        let mut teams: HashMap<TeamID, Roster> = HashMap::new();
        let mut all_qbs: HashMap<String, QBStats> = HashMap::new();
        let mut all_rbs: HashMap<String, RBStats> = HashMap::new();
        let mut all_wrs: HashMap<String, WRStats> = HashMap::new();
        let mut all_tes: HashMap<String, TEStats> = HashMap::new();
        let mut all_dbs: HashMap<String, DBStats> = HashMap::new();
        let mut all_lbs: HashMap<String, LBStats> = HashMap::new();
        let mut all_dls: HashMap<String, DLStats> = HashMap::new();
        let mut all_ols: HashMap<String, OLStats> = HashMap::new();
        let mut all_ks: HashMap<String, KStats> = HashMap::new();
        let mut all_krs: HashMap<String, KRStats> = HashMap::new();

        for roster in rosters {
            for p in &roster.players {
                let id = p.get_id();
                match p.get_full_player() {
                    Player::QB(s) => {
                        all_qbs.insert(id, s);
                    }
                    Player::RB(s) => {
                        all_rbs.insert(id, s);
                    }
                    Player::WR(s) => {
                        all_wrs.insert(id, s);
                    }
                    Player::TE(s) => {
                        all_tes.insert(id, s);
                    }
                    Player::DB(s) => {
                        all_dbs.insert(id, s);
                    }
                    Player::LB(s) => {
                        all_lbs.insert(id, s);
                    }
                    Player::DL(s) => {
                        all_dls.insert(id, s);
                    }
                    Player::OL(s) => {
                        all_ols.insert(id, s);
                    }
                    Player::K(s) => {
                        all_ks.insert(id, s);
                    }
                    Player::KR(s) => {
                        all_krs.insert(id, s);
                    }
                    Player::P(_) | Player::PR(_) => {}
                }
            }
            teams.insert(roster.team_name.clone(), roster);
        }

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
            all_ks,
            all_krs,
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
}

#[cfg(test)]
mod tests {
    //! Pure-logic unit tests (Testing Stage T2, see `docs/plans/testing-plan.md`).
    //!
    //! Covers `TeamID::create_from_str`: the hardcoded name-normalization ("fixup")
    //! table and the `splitn` year/name split with its defaults. Pure, no I/O.

    use super::*;

    #[test]
    fn test_create_from_str_normalizes_known_aliases() {
        // (raw "YEAR NAME", expected year, expected normalized name) for the fixup table.
        let cases = [
            ("1983 N.Y.Giants", "1983", "N.Y. Giants"),
            ("1983 NY Giants", "1983", "N.Y. Giants"),
            ("1983 New York G", "1983", "N.Y. Giants"),
            ("1983 SanDiego", "1983", "San Diego"),
            ("1983 NewOrleans", "1983", "New Orleans"),
            ("1983 SanFran", "1983", "San Francisco"),
            ("1983 St.Louis", "1983", "St. Louis"),
            ("1983 NewEngland", "1983", "New England"),
            ("1983 N.Y.Jets", "1983", "N.Y. Jets"),
            ("1983 New York J", "1983", "N.Y. Jets"),
            ("1983 NY Jets", "1983", "N.Y. Jets"),
            ("1983 KansasCity", "1983", "Kansas City"),
            ("1983 L.A.Raiders", "1983", "L.A. Raiders"),
            ("1983 L.A.Rams", "1983", "L.A. Rams"),
            ("1983 LA Raiders", "1983", "L.A. Raiders"),
            ("1983 LA Rams", "1983", "L.A. Rams"),
            ("1983 Balimore", "1983", "Baltimore"),
            ("1983 Cincinatti", "1983", "Cincinnati"),
        ];

        for (raw, year, name) in cases {
            let id = TeamID::create_from_str(raw);
            assert_eq!(id.year, year, "year for {:?}", raw);
            assert_eq!(id.name, name, "normalized name for {:?}", raw);
        }
    }

    #[test]
    fn test_create_from_str_passes_through_unmapped_name() {
        // A name that is not in the fixup table is used verbatim.
        let id = TeamID::create_from_str("1983 Chicago");
        assert_eq!(id.year, "1983");
        assert_eq!(id.name, "Chicago");
    }

    #[test]
    fn test_create_from_str_splits_year_and_multiword_name() {
        // Only the first space separates year from name; the rest stays intact.
        let id = TeamID::create_from_str("1983 Green Bay");
        assert_eq!(id.year, "1983");
        assert_eq!(id.name, "Green Bay");
    }

    #[test]
    fn test_create_from_str_trims_surrounding_whitespace() {
        let id = TeamID::create_from_str("  1983 SanDiego  ");
        assert_eq!(id.year, "1983");
        assert_eq!(id.name, "San Diego");
    }

    #[test]
    fn test_create_from_str_defaults_when_name_missing() {
        // Year present but no name -> default name "Omaha".
        let id = TeamID::create_from_str("1983");
        assert_eq!(id.year, "1983");
        assert_eq!(id.name, "Omaha");
    }

    #[test]
    fn test_create_from_str_defaults_when_empty() {
        // Characterization: `splitn(2, ' ')` on a trimmed-empty string still yields one
        // (empty) element, so `year` becomes "" and only the *name* falls back to its
        // default ("Omaha"). The "1980" year default is only reached when the first
        // `splitn` item is `None`, which a trimmed string never produces. This documents
        // current behavior (not asserted-as-desired); empty team input is not real data.
        let id = TeamID::create_from_str("");
        assert_eq!(id.year, "");
        assert_eq!(id.name, "Omaha");
    }
}
