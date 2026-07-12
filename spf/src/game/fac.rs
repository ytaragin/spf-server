use csv::ReaderBuilder;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde_derive::Serialize;
use std::error::Error;
use std::fs::File;
use std::str::FromStr;

use rand::seq::SliceRandom;
use rand::thread_rng;

use super::{
    lineup::{DefensiveBox, OffensiveBox},
    standard_play::PassResult,
};

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

#[derive(Debug, Clone, Serialize)]
pub struct ScreenResult {
    pub result: PassResult,
    pub multiplier: f32,
}

impl From<String> for ScreenResult {
    fn from(s: String) -> Self {
        let parts: Vec<&str> = s.split(' ').collect();
        let result: PassResult = parts[0].parse().unwrap();
        if parts.len() > 2 {
            ScreenResult {
                result,
                multiplier: parts[2].parse::<f32>().unwrap(),
            }
        } else {
            ScreenResult {
                result,
                multiplier: 1.0,
            }
        }
    }
}

impl<'de> Deserialize<'de> for ScreenResult {
    fn deserialize<D>(deserializer: D) -> Result<ScreenResult, D::Error>
    where
        D: Deserializer<'de>,
    {
        let instr = String::deserialize(deserializer)?;

        Ok(ScreenResult::from(instr))
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
    pub sc: ScreenResult,
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
impl FacCard {
    pub fn get_max_rn() -> i32 {
        12
    }
    // unused: symmetric twin of the used `get_max_rn`; kept pending removal.
    #[allow(dead_code)]
    pub fn get_max_pn() -> i32 {
        48
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacManager {
    facs: Vec<FacCard>,
    deck: Vec<FacCard>,
    /// Whether refilling the draw deck reshuffles it. `true` for real (CSV-loaded) decks —
    /// the sole source of engine nondeterminism. `false` for decks injected via
    /// [`from_cards`](Self::from_cards) so tests get a reproducible, ordered draw sequence.
    shuffle_on_refill: bool,
}

impl FacManager {
    /// Build a manager from an in-memory, explicitly ordered card list, bypassing the
    /// shuffle. Cards are drawn in the given order (front to back). This is the
    /// deterministic testing seam (see `docs/design/testing-strategy.md` §5); it performs
    /// no I/O.
    // Test-only seam today; keep the non-test build warning-free (T6).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_cards(cards: Vec<FacCard>) -> Self {
        // `get_fac` pops from the tail, so store reversed to draw in the caller's order.
        let mut deck: Vec<FacCard> = cards.clone();
        deck.reverse();
        Self {
            facs: cards,
            deck,
            shuffle_on_refill: false,
        }
    }

    /// Load a shuffling deck from a FAC CSV file, surfacing any I/O / parse error instead of
    /// panicking. This is the production loading path.
    pub fn from_csv(filename: &str) -> Result<Self, Box<dyn Error>> {
        let fac_data = read_csv_file(filename)?;
        let facs = fac_data.into_iter().map(FacCard::from).collect();

        Ok(Self {
            facs,
            deck: vec![],
            shuffle_on_refill: true,
        })
    }

    pub fn get_fac(&mut self, force_shuffle: bool) -> FacCard {
        if force_shuffle || self.deck.is_empty() {
            self.deck = self.facs.clone();
            if self.shuffle_on_refill {
                self.deck.shuffle(&mut thread_rng());
            } else {
                // Preserve the caller's draw order across refills.
                self.deck.reverse();
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn data_card(id: i32) -> FacCard {
        let d = FacData {
            id,
            run_num: RunNum { num: 1, ob: false },
            pass_num: 1,
            sl: RunDirection::Break,
            il: RunDirection::Break,
            ir: RunDirection::Break,
            sr: RunDirection::Break,
            er: String::new(),
            sc: ScreenResult {
                result: PassResult::Complete,
                multiplier: 1.0,
            },
            sh: PassTarget::Orig,
            qk: PassTarget::Orig,
            lg: PassTarget::Orig,
            z_result: String::new(),
            solitaire: String::new(),
        };
        FacCard::Data(d)
    }

    fn card_id(c: &FacCard) -> i32 {
        match c {
            FacCard::Data(d) => d.id,
            FacCard::Z => -1,
        }
    }

    #[test]
    fn test_from_cards_draws_in_order_without_shuffle() {
        let cards = vec![data_card(1), data_card(2), data_card(3)];
        let mut mgr = FacManager::from_cards(cards);

        // Drawn in the exact order supplied (no shuffle).
        assert_eq!(card_id(&mgr.get_fac(false)), 1);
        assert_eq!(card_id(&mgr.get_fac(false)), 2);
        assert_eq!(card_id(&mgr.get_fac(false)), 3);
    }

    #[test]
    fn test_from_cards_refills_in_same_order() {
        let cards = vec![data_card(10), data_card(20)];
        let mut mgr = FacManager::from_cards(cards);

        // Exhaust the deck, then the next draw refills — deterministically, same order.
        assert_eq!(card_id(&mgr.get_fac(false)), 10);
        assert_eq!(card_id(&mgr.get_fac(false)), 20);
        assert_eq!(card_id(&mgr.get_fac(false)), 10);
        assert_eq!(card_id(&mgr.get_fac(false)), 20);
    }

    #[test]
    fn test_empty_injected_deck_yields_z() {
        let mut mgr = FacManager::from_cards(vec![]);
        assert!(matches!(mgr.get_fac(false), FacCard::Z));
    }
}
