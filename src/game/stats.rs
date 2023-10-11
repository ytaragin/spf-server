use serde_derive::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Range {
    pub start: i32,
    pub end: i32,
}

impl Range {
    pub fn new() -> Self {
        Self { start: 49, end: 49 }
    }

    pub fn from_str(instr: &str) -> Self {
        let mut start = 49;
        let mut end = 49;

        let vals: Vec<&str> = instr.split('-').collect();
        if vals.len() >= 1 {
            if let Ok(num) = vals[0].parse::<i32>() {
                start = num;
                end = num;
            }
        }

        if vals.len() >= 2 {
            if let Ok(num) = vals[1].parse::<i32>() {
                end = num;
            }
        }

        Range { start, end }
    }

    pub fn get_tag_and_range(instr: &str) -> (&str, Self) {
        let vals: Vec<&str> = instr.split(':').map(|s| s.trim()).collect();
        match vals.len() {
            0 => ("", Range::new()),
            1 => (vals[0], Range::new()),
            _ => (vals[0], Range::from_str(vals[1])),
        }
    }

    pub fn in_range(&self, num: i32) -> bool {
        return num >= self.start && num <= self.end;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TripleStat {
    stats: HashMap<String, NumStat>,
}

impl TripleStat {
    pub fn create_from_str(labels: String, val: &str) -> Self {
        let labels: Vec<&str> = labels.split('/').collect();
        let split_stats: Vec<NumStat> = val.split('/').map(|s| NumStat::gen_from_str(s)).collect();

        let mut stats: HashMap<String, NumStat> = HashMap::new();

        for i in 0..=2 {
            if i < split_stats.len() {
                stats.insert(labels[i].to_string(), split_stats[i]);
            }
        }

        return Self { stats };
    }

    pub fn curry_create(labels: &str) -> impl Fn(&str) -> Self + '_ {
        move |vals| TripleStat::create_from_str(labels.to_string(), vals)
    }

    pub fn get_val(&self, cat: String) -> Option<&NumStat> {
        self.stats.get(&cat)
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub enum NumStat {
    Sg,
    Lg,
    Val(i32),
}

impl NumStat {
    pub fn gen_from_str(val: &str) -> Self {
        match val {
            "Sg" => NumStat::Sg,
            "Lg" => NumStat::Lg,
            _ => match val.parse::<i32>() {
                Ok(number) => NumStat::Val(number),
                Err(_) => NumStat::Val(0),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TwelveStats<T> {
    pub stats: Vec<T>,
}

impl<T> TwelveStats<T> {
    pub fn get_stat(&self, num: usize) -> &T {
        return &self.stats[num - 1];
    }

    pub fn create_from_strs<F>(vals: &[&str], item_generator: F) -> Self
    where
        F: Fn(&str) -> T,
    {
        let stats: Vec<T> = vals
            .iter()
            .map(|v| v.split(':').map(|s| s.trim()))
            .map(|mut it| item_generator(it.nth(1).unwrap_or("0")))
            .collect();

        return Self { stats };
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RangedStats<T: FromStr + Eq + PartialEq + Hash> {
    // pub com: Range,
    // pub inc: Range,
    // pub int: Range,
    stats: HashMap<T, Range>,
}

impl<T: FromStr + Eq + PartialEq + Hash> RangedStats<T> {
    pub fn create_from_strs<F>(vals: &[&str]) -> Self {
        let mut stats: HashMap<T, Range> = HashMap::new();

        for v in vals {
            let p = Range::get_tag_and_range(v);
            if let Ok(k) = T::from_str(p.0) {
                stats.insert(k, p.1);
            } else {
                println!("Invalid type {:?}", v)
            }
        }

        Self { stats }
    }

    pub fn get_category(&self, val: i32) -> &T {
        let res = self
            .stats
            .iter()
            .find_map(|(key, r)| if r.in_range(val) { Some(key) } else { None });
        return res.unwrap();
    }
}

// pub struct RangedStats {
//     // pub com: Range,
//     // pub inc: Range,
//     // pub int: Range,
//     stats: HashMap<String, Range>,
// }

// impl RangedStats {
//     pub fn create_from_strs(vals: &[&str]) -> Self {
//         let mut stats: HashMap<String, Range> = HashMap::new();

//         for v in vals {
//             let p = Range::get_tag_and_range(v);
//             stats.insert(p.0.to_string(), p.1);
//         }

//         Self { stats }
//     }

//     pub fn get_category(&self, val: i32) -> String {
//         return "ZZZ".to_string();
//     }
// }
