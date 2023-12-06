use csv::ReaderBuilder;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde_derive::Serialize;
use std::error::Error;
use std::fs::File;
use std::str::FromStr;

use rand::seq::SliceRandom;
use rand::thread_rng;

use super::lineup::{DefensiveBox, OffensiveBox};

#[derive(Debug, Serialize, Clone)]
pub struct RunNum {
    pub num: i32,
    pub ob: bool,
}

impl<'de> Deserialize<'de> for RunNum {
    fn deserialize<D>(deserializer: D) -> Result<RunNum, D::Error>
    where
        D: Deserializer<'de>,
    {
        let instr = String::deserialize(deserializer)?;
        // println!("Attempting to parse: {}", instr);

        let re = Regex::new(r"^(\d+) ?(\(OB\))?$").unwrap();
        if let Some(caps) = re.captures(instr.as_str()) {
            // parse the number as an i32
            let num = caps[1].parse::<i32>().unwrap_or(-1);
            // check if the suffix is present
            let ob = caps.get(2).is_some();
            // return a new RunNum instance
            Ok(RunNum { num, ob })
        } else {
            println!("Fac Error with {}", instr);
            Ok(RunNum { num: -1, ob: false })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDirectionActual {
    pub offensive_boxes: Vec<OffensiveBox>,
    pub defensive_boxes: Vec<DefensiveBox>,
}

#[derive(Debug, Clone, Serialize)]
pub enum RunDirection {
    Actual(RunDirectionActual),
    Break,
}

impl From<RunDirectionActual> for RunDirection {
    fn from(v: RunDirectionActual) -> Self {
        Self::Actual(v)
    }
}

impl<'de> Deserialize<'de> for RunDirection {
    fn deserialize<D>(deserializer: D) -> Result<RunDirection, D::Error>
    where
        D: Deserializer<'de>,
    {
        let instr = String::deserialize(deserializer)?;

        if instr.eq("break") {
            return Ok(RunDirection::Break);
        }
        let re = Regex::new(r"[A-Z]{1,2}").unwrap();

        // create an empty vector to store the matches
        let mut def_list: Vec<DefensiveBox> = Vec::new();
        let mut off_list: Vec<OffensiveBox> = Vec::new();

        // iterate over the matches and push them to the vector
        for m in re.find_iter(instr.as_str()) {
            let s = m.as_str();
            match s.len() {
                1 => def_list.push(
                    DefensiveBox::from_str(format!("{}", s.to_lowercase()).as_str()).unwrap(),
                ),
                2 => off_list.push(OffensiveBox::from_str(s.to_lowercase().as_str()).unwrap()),
                _ => println!("Error {}", instr),
            };
        }

        Ok(RunDirection::from(RunDirectionActual {
            offensive_boxes: off_list,
            defensive_boxes: def_list,
        }))
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum PassTarget {
    Orig,
    PassRush,
    Actual(OffensiveBox),
}

impl From<String> for PassTarget {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Orig" => PassTarget::Orig,
            "PassRush" => PassTarget::PassRush,
            _ => PassTarget::Actual(OffensiveBox::from_str(s.to_lowercase().as_str()).unwrap()),
        }
    }
}

impl<'de> Deserialize<'de> for PassTarget {
    fn deserialize<D>(deserializer: D) -> Result<PassTarget, D::Error>
    where
        D: Deserializer<'de>,
    {
        let instr = String::deserialize(deserializer)?;

        Ok(PassTarget::from(instr))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FacData {
    pub id: i32,
    pub run_num: RunNum,
    pub pass_num: i32,
    pub sl: RunDirection,
    pub il: RunDirection,
    pub ir: RunDirection,
    pub sr: RunDirection,
    pub er: String,
    pub sc: String,
    pub sh: PassTarget,
    pub qk: PassTarget,
    pub lg: PassTarget,
    pub z_result: String,
    pub solitaire: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FacCard {
    Z,
    Data(FacData),
}

impl From<FacData> for FacCard {
    fn from(v: FacData) -> Self {
        Self::Data(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacManager {
    facs: Vec<FacCard>,
    deck: Vec<FacCard>,
}

impl FacManager {
    pub fn new(filename: &str) -> Self {
        // return an instance of the struct with the given values
        let fac_data = read_csv_file(filename).unwrap();
        let facs = fac_data.into_iter().map(|f| FacCard::from(f)).collect();

        let deck: Vec<FacCard> = vec![];

        return Self { facs, deck };
    }

    pub fn get_fac(&mut self, force_shuffle: bool) -> FacCard {
        if force_shuffle || self.deck.is_empty() {
            self.deck = self.facs.clone();
            self.deck.shuffle(&mut thread_rng());
        }

        let c = self.deck.pop().unwrap_or(FacCard::Z);

        return c;
    }
}

// fn parse_record()

pub fn read_csv_file(filename: &str) -> Result<Vec<FacData>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: FacData = result?;
        records.push(record);
    }

    println!("Done");

    return Ok(records);
}
