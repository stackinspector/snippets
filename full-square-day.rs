use std::{collections::HashMap as Map, time::SystemTime};
use chrono::{NaiveDate, Datelike};
use deepsize::DeepSizeOf;

trait TypeNameOf {
    fn type_name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }
}

impl<T> TypeNameOf for T {}

type Int = u32;

fn datenum(date: NaiveDate) -> Int {
    (date.year() as Int) * 10000 + date.month() * 100 + date.day()
}

fn next_day(date: NaiveDate) -> NaiveDate {
    date.checked_add_signed(chrono::TimeDelta::days(1)).unwrap()
}

fn square(num: Int) -> Int {
    num.checked_pow(2).unwrap()
}

fn sqrt_prevint(num: Int) -> Int {
    (num as f64).sqrt().floor() as Int
}

fn sqrt_nextint(num: Int) -> Int {
    (num as f64).sqrt().ceil() as Int
}

fn next_num(num: Int) -> Int {
    num + 1
}

struct DateNumIter {
    end: NaiveDate,
    next: NaiveDate,
}

impl DateNumIter {
    fn new(start: NaiveDate, end: NaiveDate) -> DateNumIter {
        DateNumIter {
            end,
            next: start,
        }
    }
}

impl Iterator for DateNumIter {
    type Item = Int;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.next;
        if curr > self.end {
            None
        } else {
            self.next = next_day(self.next);
            Some(datenum(curr))
        }
    }
}

struct FullSquareNumIter {
    end: Int,
    next: Int,
}

impl FullSquareNumIter {
    fn new(start: Int, end: Int) -> FullSquareNumIter {
        FullSquareNumIter {
            end,
            next: start,
        }
    }
}

impl Iterator for FullSquareNumIter {
    type Item = (Int, Int);

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.next;
        if curr > self.end {
            None
        } else {
            self.next = next_num(self.next);
            Some((square(curr), curr))
        }
    }
}

fn main() {
    let start = NaiveDate::from_ymd_opt(2001, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();
    let min_root = sqrt_prevint(datenum(start));
    let max_root = sqrt_nextint(datenum(end));

    dbg!(start);
    dbg!(end);
    dbg!(min_root);
    dbg!(max_root);

    let num_start = SystemTime::now();

    let num_set: Map<Int, Int> = Map::from_iter(FullSquareNumIter::new(min_root, max_root));

    dbg!(num_start.elapsed().unwrap());
    dbg!(num_set.len());
    dbg!(num_set.type_name());
    dbg!(num_set.deep_size_of());

    let find_start = SystemTime::now();

    let mut result_set = Vec::new();
    for datenum in DateNumIter::new(start, end) {
        if let Some(root) = num_set.get(&datenum) {
            result_set.push((*root, datenum));
        }
    }

    dbg!(find_start.elapsed().unwrap());
    dbg!(result_set.len());

    for (root, datenum) in result_set.iter() {
        println!("root={root}, datenum={datenum}");
    }
}
