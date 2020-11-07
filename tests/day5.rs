//! AoC 2019 day 5: https://adventofcode.com/2019/day/5

extern crate intcode;
use std::fs::File;
use std::io;

fn run_day5(input: &[u8], output: &[u8]) {
    let f = File::open("input/day5.int").unwrap();
    let reader = io::BufReader::new(f);
    let mut ic = intcode::Program::new(reader);
    let mut inc = io::Cursor::new(input);
    let mut outc = io::Cursor::new(vec![0; output.len()]);

    ic.exe(0, false, &mut inc, &mut outc)
        .expect("execution error");
    assert_eq![outc.get_ref(), output];
}

#[test]
fn part1() {
    run_day5(b"1", b"?0\n0\n0\n0\n0\n0\n0\n0\n0\n6745903\n\0");
}

#[test]
fn part2() {
    run_day5(b"5", b"?9168267\n\0");
}
