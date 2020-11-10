//! AoC 2019 day 9: https://adventofcode.com/2019/day/9

extern crate intcode;
use intcode::{Input, Output, Program};
use std::fs::File;
use std::io;

fn run_day9(input: &[u8], output: &[u8]) {
    let f = File::open("input/day9.int").unwrap();
    let reader = io::BufReader::new(f);
    let mut ic = Program::new(reader);
    let mut inc = io::Cursor::new(input);
    let mut outc = io::Cursor::new(vec![0; output.len()]);

    ic.exe(0, false, Input::Reader(&mut inc), Output::Writer(&mut outc))
        .expect("execution error");
    assert_eq![outc.get_ref().as_slice(), output];
}

#[test]
fn part1() {
    run_day9(b"1", b"?2171728567\n\0");
}

#[test]
fn part2() {
    run_day9(b"2", b"?49815\n\0");
}
