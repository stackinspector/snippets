#![allow(unused)]

use std::{collections::HashSet, io::{BufRead, BufReader, Read}};

const HASH_LEN: usize = 78;
type Hash = [u8; HASH_LEN];
const START_END_SEP: Hash = *b"---------------------------------------------------------------- -------------";
const DIR_SEP:       Hash = *b"                                                                              ";

enum State {
    NotStarted,
    Started,
    Ended,
}

#[derive(Debug)]
enum Line {
    Hash(Hash),
    StartEnd,
    Dir,
}

fn parse_line(line: &str) -> Line {
    let hash: Hash = line.as_bytes()[..HASH_LEN].try_into().unwrap();
    match hash {
        DIR_SEP => Line::Dir,
        START_END_SEP => Line::StartEnd,
        hash => Line::Hash(hash)
    }
    // TODO size
}

// struct Context {}

trait Set<T> {
    fn put(&mut self, item: T);
}

impl<T> Set<T> for Vec<T> {
    fn put(&mut self, item: T) {
        self.push(item);
    }
}

impl<T: Eq + core::hash::Hash> Set<T> for HashSet<T> {
    fn put(&mut self, item: T) {
        assert!(self.insert(item));
    }
}

fn parse_input<R: Read, S: Set<Hash>>(handle: R, set: &mut S) {
    let handle = BufReader::new(handle);
    let mut state = State::NotStarted;
    for orig_line in handle.lines().map(Result::unwrap) {
        let line = parse_line(&orig_line);
        match state {
            State::Started => {
                match line {
                    Line::Hash(hash) => set.put(hash),
                    Line::StartEnd => {
                        state = State::Ended;
                        eprintln!("ended");
                        break;
                    },
                    Line::Dir => {}, // TODO
                }
            },
            State::NotStarted => {
                match line {
                    Line::StartEnd => {
                        state = State::Started;
                        eprintln!("started");
                    },
                    _ => panic!("data before start: {:?} \"{}\"", line, orig_line),
                }
            },
            State::Ended => panic!("data before start: {:?} \"{}\"", line, orig_line),
        }
    }
}

fn main() {
    let mut args = std::env::args_os();
    let _ = args.next();
    let handle1 = std::fs::File::open(args.next().unwrap()).unwrap();
    let handle2 = std::fs::File::open(args.next().unwrap()).unwrap();
    let mut set1 = HashSet::new();
    let mut set2 = Vec::new();
    parse_input(handle1, &mut set1);
    parse_input(handle2, &mut set2);
    let mut error_set1 = false;
    for item in set2 {
        if set1.contains(&item) {
            assert!(set1.remove(&item));
        } else {
            error_set1 = true;
            println!("incorrect hash from set1 {}", core::str::from_utf8(&item).unwrap())
        }
    }
    if set1.is_empty() && !error_set1 {
        println!("equal");
    } else {
        for item in set1 {
            println!("incorrect hash from set2 {}", core::str::from_utf8(&item).unwrap())
        }
    }
}
