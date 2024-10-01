use std::collections::{HashMap, HashSet};

use chrono::{NaiveDate, Datelike};

fn datenum(date: NaiveDate) -> u32 {
    (date.year() as u32) * 10000 + date.month() * 100 + date.day()
}

fn main() {
    let start = NaiveDate::from_ymd_opt(2001, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();

    let datenum_set = {
        let mut date = start;
        let mut datenum_set = HashSet::new();
        loop {
            assert!(datenum_set.insert(datenum(date)));
            date = date.checked_add_signed(chrono::TimeDelta::days(1)).unwrap();
            if date > end {
                break;
            }
        }
        datenum_set
    };
    
    let full_sqrt_set = {
        // let start_num = datenum(start);
        let end_num = datenum(end);
        let mut num = 1u32;
        let mut num_set = HashMap::new();
        loop {
            let powered = num.checked_pow(2).unwrap();
            assert!(matches!(num_set.insert(powered, num), None));
            num += 1;
            if powered > end_num {
                break;
            }
        }
        num_set
    };
}
