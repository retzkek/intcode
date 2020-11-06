//! AoC 2019 day 2: https://adventofcode.com/2019/day/2

extern crate intcode;
use std::fs::File;
use std::io;

#[test]
fn main() {
    let f = File::open("input/day2.int").unwrap();
    let reader = io::BufReader::new(f);
    let mut ic = intcode::Program::new(reader);

    // before running the program, replace position 1 with the value 12 and
    // replace position 2 with the value 2.
    ic.poke(1, 12);
    ic.poke(2, 2);

    let mut input = io::Cursor::new("");
    ic.exe(0, false, &mut input).expect("execution error");

    // What value is left at position 0 after the program halts?
    assert_eq![ic.peek(0), 4462686];
}
