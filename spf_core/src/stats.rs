use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::cmp::min;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::{collections::HashMap, str::FromStr};

use crate::shiftable::Shiftable;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

    pub fn get_tag_and_range<'a>(instr: &'a str, splitter: &str) -> (&'a str, Self) {
        let vals: Vec<&str> = instr.split(splitter).map(|s| s.trim()).collect();
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

impl FromStr for Range {
    type Err = String;

    fn from_str(instr: &str) -> Result<Self, Self::Err> {
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

        Ok(Range { start, end })
    }
}

impl Shiftable<Range> for Range {
    fn get_first() -> Range {
        todo!()
    }

    fn get_second() -> Range {
        todo!()
    }
}

impl serde::Serialize for Range {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}-{}", self.start, self.end);
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for Range {
    fn deserialize<D>(deserializer: D) -> Result<Range, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // let (start, end) = parse_range_string(&s)?;
        let r = Range::from_str(s.as_str());
        Ok(r)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledStat<T> {
    stats: HashMap<String, T>,
}

impl<T: FromStr + Copy> LabeledStat<T>
where
    <T as FromStr>::Err: Debug,
{
    pub fn create_from_str(labels: String, val: &str) -> Self {
        let labels: Vec<&str> = labels.split('/').collect();
        let split_stats: Vec<T> = val.split('/').map(|s| T::from_str(s).unwrap()).collect();

        let mut stats: HashMap<String, T> = HashMap::new();

        for i in 0..split_stats.len() {
            // if i < split_stats.len() {
            stats.insert(labels[i].to_string(), split_stats[i]);
            // }
        }

        return Self { stats };
    }

    pub fn curry_create(labels: &str) -> impl Fn(&str) -> Self + '_ {
        move |vals| LabeledStat::create_from_str(labels.to_string(), vals)
    }

    pub fn get_val(&self, cat: String) -> Option<&T> {
        self.stats.get(&cat)
    }
}

impl<T: fmt::Display> fmt::Display for LabeledStat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut stats = self.stats.iter().collect::<Vec<_>>();
        stats.sort_by_key(|&(k, _)| k);
        let stats = stats
            .into_iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "LabeledStat({})", stats)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwelveStats<T> {
    pub stats: Vec<T>,
}

impl<T: Debug> TwelveStats<T> {
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

    pub fn print_out(&self) {
        for (i, e) in self.stats.iter().enumerate() {
            println!("The index is {} and the element is {:?}", i, e);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangedStats<T>
where
    T: Eq + Hash,
{
    // pub com: Range,
    // pub inc: Range,
    // pub int: Range,
    stats: HashMap<T, Range>,
}

impl<T: FromStr + Eq + Clone + PartialEq + Hash + Shiftable<T> + Debug> RangedStats<T> {
    pub fn create_from_strs(vals: &[&str], splitter: &str) -> Self {
        let mut stats: HashMap<T, Range> = HashMap::new();

        for v in vals {
            let p = Range::get_tag_and_range(v, splitter);
            if let Ok(k) = T::from_str(p.0) {
                stats.insert(k, p.1);
            } else {
                println!("Invalid type {:?}", v)
            }
        }

        Self { stats }
    }

    pub fn get_category(&self, val: i32, shift: i32) -> T {
        let first = T::get_first();
        let second = T::get_second();

        let first_range = self.stats.get(&first).unwrap();
        let second_range = self.stats.get(&second).unwrap();

        let new_first = Range {
            start: first_range.start,
            end: min(first_range.end + shift, second_range.end),
        };
        let new_second = Range {
            start: min(second_range.start + shift, second_range.end),
            end: second_range.end,
            // end: min(second_range.end, second_range.start + shift),
        };

        let mut new_stats = self.stats.clone();
        new_stats.insert(first, new_first);
        new_stats.insert(second, new_second);

        println!("Stats for category: {:?}", new_stats);

        let res = new_stats.iter().find_map(|(key, r)| {
            if r.in_range(val) {
                Some(key.clone())
            } else {
                None
            }
        });
        return res.unwrap();
    }
}

#[cfg(test)]
mod tests {
    //! Test-harness bootstrap (Testing Stage T1, see `docs/plans/testing-plan.md`).
    //!
    //! These exercise `Range`, a small, pure, dependency-free leaf type, purely to move
    //! the suite from "0 tests" to "green with real assertions" and confirm the
    //! `cargo test` loop works. Broader pure-logic coverage is Stage T2.

    use super::*;

    #[test]
    fn test_range_from_str_parses_start_and_end() {
        let r = Range::from_str("12-18");
        assert_eq!(r.start, 12);
        assert_eq!(r.end, 18);
    }

    #[test]
    fn test_range_from_str_single_value_sets_both_ends() {
        let r = Range::from_str("7");
        assert_eq!(r.start, 7);
        assert_eq!(r.end, 7);
    }

    #[test]
    fn test_range_from_str_defaults_to_49_on_garbage() {
        // Neither side parses as an int, so both fall back to the default of 49.
        let r = Range::from_str("abc");
        assert_eq!(r.start, 49);
        assert_eq!(r.end, 49);
    }

    #[test]
    fn test_range_fromstr_trait_matches_inherent() {
        // The `FromStr` impl never errors; it mirrors the inherent `from_str`.
        let via_trait: Range = "3-9".parse().expect("Range::from_str never returns Err");
        let via_inherent = Range::from_str("3-9");
        assert_eq!(via_trait, via_inherent);
    }

    #[test]
    fn test_range_in_range_is_inclusive() {
        let r = Range::from_str("4-6");
        assert!(!r.in_range(3));
        assert!(r.in_range(4)); // lower bound inclusive
        assert!(r.in_range(5));
        assert!(r.in_range(6)); // upper bound inclusive
        assert!(!r.in_range(7));
    }

    #[test]
    fn test_get_tag_and_range_splits_and_trims() {
        let (tag, range) = Range::get_tag_and_range("COM : 12-18", ":");
        assert_eq!(tag, "COM");
        assert_eq!(range.start, 12);
        assert_eq!(range.end, 18);
    }

    #[test]
    fn test_get_tag_and_range_tag_only_uses_default_range() {
        let (tag, range) = Range::get_tag_and_range("COM", ":");
        assert_eq!(tag, "COM");
        // No range portion -> default Range::new() (49-49).
        assert_eq!(range.start, 49);
        assert_eq!(range.end, 49);
    }
}
