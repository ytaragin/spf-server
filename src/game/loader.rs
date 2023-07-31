use crate::players::{DBStats, QBStats, RBStats, TeamID, WRStats};
use std::fs;

use crate::players::Position;

use super::{
    players::{DLStats, LBStats, OLStats, TEStats},
    stats::{NumStat, Range, RangedStats, TripleStat, TwelveStats},
};

pub fn load_rbs(filename: String) -> Vec<RBStats> {
    return parse_records(filename, 35, Position::RB, parse_rb_record).unwrap();

    // println!("{:?}", res);
}

fn parse_rb_record((id, lines): (String, &[&str])) -> Option<RBStats> {
    let team = TeamID::create_from_str(lines[0]);

    let name = lines[2].to_string();

    let blocks = text_str_to_num(lines[20]);
    let lg = get_char(lines[18]);

    let rushing_slice = &lines[5..=16];

    let rushing_func = TripleStat::curry_create(lines[4]);
    let rushing = TwelveStats::create_from_strs(rushing_slice, rushing_func);

    let passing_slice = &lines[23..=34];
    let passing_func = TripleStat::curry_create(lines[22]);
    let pass_gain = TwelveStats::create_from_strs(passing_slice, passing_func);

    return Some(RBStats {
        team,
        name,
        id,
        position: Position::RB,
        rushing,
        pass_gain,
        lg,
        blocks,
    });
}

pub fn load_qbs(filename: String) -> Vec<QBStats> {
    return parse_records(filename, 39, Position::QB, parse_qb_record).unwrap();

    // println!("{:?}", res);
}

fn parse_qb_record((id, lines): (String, &[&str])) -> Option<QBStats> {
    let team = TeamID::create_from_str(lines[0]);
    let name = lines[2].to_string();
    let position = Position::QB;

    let endurance = get_char_from_val(lines[3], "A");

    let quick = RangedStats::create_from_strs(&lines[6..=8]);
    let short = RangedStats::create_from_strs(&lines[10..=12]);
    let long = RangedStats::create_from_strs(&lines[14..=16]);

    let pass_rush = RangedStats::create_from_strs(&lines[18..=21]);

    let long_run = get_char_from_val(lines[36], "R");
    let endurance_rushing = get_i32_from_val(lines[38], 0);

    let rushing = TwelveStats::create_from_strs(&lines[23..=34], NumStat::gen_from_str);

    return Some(QBStats {
        team,
        name,
        id,
        position,
        endurance,
        quick,
        short,
        long,
        long_run,
        pass_rush,
        endurance_rushing,
        rushing,
    });
}

pub fn load_wrs(filename: String) -> Vec<WRStats> {
    return parse_records(filename, 35, Position::WR, parse_wr_record).unwrap();

    // println!("{:?}", res);
}

fn parse_wr_record((id, lines): (String, &[&str])) -> Option<WRStats> {
    let team = TeamID::create_from_str(lines[0]);
    let name = lines[2].to_string();
    let position = Position::WR;

    let rushing_func = TripleStat::curry_create(lines[4]);
    let rushing = TwelveStats::create_from_strs(&lines[5..=16], rushing_func);

    let passing_func = TripleStat::curry_create(lines[20]);
    let pass_gain = TwelveStats::create_from_strs(&lines[21..=32], passing_func);

    let end = get_i32_from_val(lines[17], 0);
    let lg = get_char_from_val(lines[18], "-");
    let blocks = text_str_to_num(lines[34]);

    Some(WRStats {
        team,
        name,
        id,
        position,
        rushing,
        pass_gain,
        end,
        lg,
        blocks,
    })
}

pub fn load_dbs(filename: String) -> Vec<DBStats> {
    return parse_records(filename, 8, Position::DB, parse_db_record).unwrap();

    // println!("{:?}", res);
}

fn parse_db_record((id, lines): (String, &[&str])) -> Option<DBStats> {
    let team = TeamID::create_from_str(lines[0]);
    let name = lines[2].to_string();
    let position = Position::DB;

    let pass_def = text_str_to_num(lines[4]);
    let pass_rush = 0;
    let intercepts = Range::from_str(lines[7]);

    Some(DBStats {
        team,
        name,
        id,
        position,
        pass_def,
        pass_rush,
        intercepts,
    })
}

pub fn load_dls(filename: String) -> Vec<DLStats> {
    return parse_records(filename, 7, Position::DL, parse_dl_record).unwrap();

    // println!("{:?}", res);
}

fn parse_dl_record((id, lines): (String, &[&str])) -> Option<DLStats> {
    let team = TeamID::create_from_str(lines[0]);
    let name = lines[2].to_string();
    let position = Position::DL;

    let tackles = text_str_to_num(lines[4]);
    let pass_rush = get_i32(lines[6], 0);

    Some(DLStats {
        team,
        name,
        id,
        position,
        tackles,
        pass_rush,
    })
}

pub fn load_lbs(filename: String) -> Vec<LBStats> {
    return parse_records(filename, 11, Position::LB, parse_lb_record).unwrap();

    // println!("{:?}", res);
}

fn parse_lb_record((id, lines): (String, &[&str])) -> Option<LBStats> {
    let team = TeamID::create_from_str(lines[0]);
    let name = lines[2].to_string();
    let position = Position::LB;

    let tackles = text_str_to_num(lines[4]);
    let pass_rush = get_i32(lines[6], 0);
    let pass_def = text_str_to_num(lines[8]);
    let intercepts = Range::from_str(lines[10]);

    Some(LBStats {
        team,
        name,
        id,
        position,
        tackles,
        pass_rush,
        pass_def,
        intercepts,
    })
}

pub fn load_ols(filename: String) -> Vec<OLStats> {
    return parse_records(filename, 7, Position::OL, parse_ol_record).unwrap();

    // println!("{:?}", res);
}

fn parse_ol_record((id, lines): (String, &[&str])) -> Option<OLStats> {
    let team = TeamID::create_from_str(lines[0]);
    let name = lines[2].to_string();
    let position = Position::OL;

    let blocks = text_str_to_num(lines[4]);
    let pass_block = get_i32(lines[6], 0);

    Some(OLStats {
        team,
        name,
        id,
        position,
        blocks,
        pass_block,
    })
}

pub fn load_tes(filename: String) -> Vec<TEStats> {
    return parse_records(filename, 35, Position::TE, parse_te_record).unwrap();

    // println!("{:?}", res);
}

fn parse_te_record((id, lines): (String, &[&str])) -> Option<TEStats> {
    let team = TeamID::create_from_str(lines[0]);
    let name = lines[2].to_string();
    let position = Position::TE;

    let rushing_func = TripleStat::curry_create(lines[4]);
    let rushing = TwelveStats::create_from_strs(&lines[5..=16], rushing_func);

    let passing_func = TripleStat::curry_create(lines[22]);
    let pass_gain = TwelveStats::create_from_strs(&lines[23..=34], passing_func);

    let blocks = text_str_to_num(lines[18]);
    let long_rush = get_char_from_val(lines[20], "R");

    Some(TEStats {
        team,
        name,
        id,
        position,
        rushing,
        blocks,
        long_rush,
        pass_gain,
    })
}

fn parse_records<T, F>(
    filename: String,
    size: usize,
    pos: Position,
    parse: F,
) -> Result<Vec<T>, std::io::Error>
where
    F: Fn((String, &[&str])) -> Option<T>,
{
    let file_contents = fs::read_to_string(filename)?;

    let binding = file_contents.lines().collect::<Vec<&str>>();

    let raw_recs: std::slice::Chunks<'_, &str> = binding.chunks(size);

    Ok(raw_recs
        .filter(|rec| rec.len() == size)
        .enumerate()
        .map(|(c, data)| (format!("{}-{}", pos, c), data))
        .filter_map(parse)
        .collect::<Vec<T>>())
}

fn dump_problematic_record(lines: &[&str]) {
    println!("Error Record***********\n{:?}", lines)
}

fn text_str_to_num(instr: &str) -> i32 {
    let parts: Vec<&str> = instr.split_whitespace().collect();

    if parts.len() < 2 {
        return 0;
    }

    let sign = match parts[0] {
        "Plus" => 1,
        "Minus" => -1,
        _ => 0, // This will be the default value if the input_string doesn't match any of the above cases
    };

    let val = match parts[1].parse::<i32>() {
        Ok(number) => number,
        Err(err) => 0,
    };

    val * sign
}

fn get_val(line: &str) -> Option<&str> {
    let vals: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
    if vals.len() > 1 {
        return Some(vals[1]);
    }
    return None;
}

fn get_char(instr: &str) -> char {
    return instr.chars().next().unwrap_or_else(|| ' ');
}

fn get_char_from_val(val: &str, def: &str) -> char {
    get_char(get_val(val).unwrap_or_else(|| def))
}

fn get_i32(val: &str, def: i32) -> i32 {
    val.parse::<i32>().unwrap_or(def)
}

fn get_i32_from_val(val: &str, def: i32) -> i32 {
    let mut ret = def;
    if let Some(s) = get_val(val) {
        ret = s.parse::<i32>().unwrap_or(def);
    }

    ret
}
