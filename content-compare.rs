#![allow(unused)]

use std::{collections::HashSet, io::{BufRead, BufReader, Read}};

const HASH_LEN: usize = 64;
type Hash = [u8; HASH_LEN];
const START_END_SEP: Hash = *b"----------------------------------------------------------------";
const DIR_SEP:       Hash = *b"                                                                ";

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

fn parse_input<R: Read>(handle: R) -> HashSet<Hash> {
    let handle = BufReader::new(handle);
    let mut state = State::NotStarted;
    let mut res = HashSet::new();
    for orig_line in handle.lines().map(Result::unwrap) {
        let line = parse_line(&orig_line);
        match state {
            State::Started => {
                match line {
                    Line::Hash(hash) => {
                        let insert_res = res.insert(hash);
                        assert!(insert_res);
                    },
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
    res
}

fn main() {
}