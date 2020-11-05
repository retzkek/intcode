//! AoC 2019 day 2: https://adventofcode.com/2019/day/2

extern crate intcode;
use intcode::Intcode;
use std::io;
use std::fs::File;

fn main() -> io::Result<()> {
    let f = File::open("input/day2.int")?;
    let reader = io::BufReader::new(f);
    let mut ic = Intcode::new(reader);

    // before running the program, replace position 1 with the value 12 and
    // replace position 2 with the value 2.
    ic.poke(1,12);
    ic.poke(2,2);

    ic.exe(0);

    // What value is left at position 0 after the program halts?
    println!["{}",ic.peek(0)];
    Ok(())
}
