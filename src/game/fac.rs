use csv::ReaderBuilder;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fs::File;

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
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

        let re = Regex::new(r"^(\d+)(\(OB\))?$").unwrap();
        if let Some(caps) = re.captures(instr.as_str()) {
            // parse the number as an i32
            let num = caps[1].parse::<i32>().unwrap_or(-1);
            // check if the suffix is present
            let ob = caps.get(2).is_some();
            // return a new RunNum instance
            Ok(RunNum { num, ob })
        } else {
            println!("Errpr with {}", instr);
            Ok(RunNum { num: -1, ob: false })
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunResultActual {
    offensive_boxes: Vec<String>,
    defensive_boxes: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RunResult {
    Actual(RunResultActual),
    Break,
}

impl From<RunResultActual> for RunResult {
    fn from(v: RunResultActual) -> Self {
        Self::Actual(v)
    }
}

impl<'de> Deserialize<'de> for RunResult {
    fn deserialize<D>(deserializer: D) -> Result<RunResult, D::Error>
    where
        D: Deserializer<'de>,
    {
        let instr = String::deserialize(deserializer)?;

        if instr.eq("break") {
            return Ok(RunResult::Break);
        }
        let re = Regex::new(r"[A-Z]{1,2}").unwrap();

        // create an empty vector to store the matches
        let mut def_list: Vec<String> = Vec::new();
        let mut off_list: Vec<String> = Vec::new();

        // iterate over the matches and push them to the vector
        for m in re.find_iter(instr.as_str()) {
            let s = m.as_str();
            match s.len() {
                1 => def_list.push(format!("box_{}", s.to_lowercase())),
                2 => off_list.push(s.to_lowercase()),
                _ => println!("Error {}", instr),
            };
        }

        Ok(RunResult::from(RunResultActual {
            offensive_boxes: off_list,
            defensive_boxes: def_list,
        }))
    }
}

#[derive(Debug, Clone)]
pub enum PassResult {
    Orig,
    PassRush,
    Actual(String),
}

impl From<String> for PassResult {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Orig" => PassResult::Orig,
            "PassRush" => PassResult::PassRush,
            _ => PassResult::Actual(s.to_lowercase()),
        }
    }
}

impl<'de> Deserialize<'de> for PassResult {
    fn deserialize<D>(deserializer: D) -> Result<PassResult, D::Error>
    where
        D: Deserializer<'de>,
    {
        let instr = String::deserialize(deserializer)?;

        Ok(PassResult::from(instr))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FacData {
    pub run_num: RunNum,
    pub pass_num: i32,
    pub sl: RunResult,
    pub il: RunResult,
    pub ir: RunResult,
    pub sr: RunResult,
    pub er: String,
    pub sc: String,
    pub sh: PassResult,
    pub qk: PassResult,
    pub lg: PassResult,
    pub z_result: String,
    pub solitaire: String,
}

#[derive(Debug, Clone)]
pub enum FacCard {
    Z,
    Data(FacData),
}

impl From<FacData> for FacCard {
    fn from(v: FacData) -> Self {
        Self::Data(v)
    }
}

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