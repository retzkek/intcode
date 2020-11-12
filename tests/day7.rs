//! AoC 2019 day 7: https://adventofcode.com/2019/day/7

extern crate intcode;
use intcode::permutations::Permutator;
use intcode::{Input, Int, Output, Program};
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::io::Write;
use std::str::FromStr;
use std::sync::mpsc::channel;

fn amp(program: &mut Program, phases: &[Int]) -> i64 {
    let mut sig: Int = 0;
    for p in phases.iter() {
        let (itx, irx) = channel::<Int>();
        let (otx, orx) = channel::<Int>();
        itx.send(*p).unwrap();
        itx.send(sig).unwrap();
        program
            .exe(0, false, Input::Channel(irx), Output::Channel(otx))
            .expect("execution error");
        sig = orx.recv().unwrap();
    }
    sig.try_into().unwrap()
}

#[test]
fn test_amp_1() {
    let code = io::Cursor::new("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0");
    let mut ic = Program::new(code);
    assert_eq![amp(&mut ic, &vec![4, 3, 2, 1, 0]), 43210];
}

#[test]
fn test_amp_2() {
    let code =
        io::Cursor::new("3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0");
    let mut ic = Program::new(code);
    assert_eq![amp(&mut ic, &vec![0, 1, 2, 3, 4]), 54321];
}

#[test]
fn test_amp_3() {
    let code =
        io::Cursor::new("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0");
    let mut ic = Program::new(code);
    assert_eq![amp(&mut ic, &vec![1, 0, 4, 3, 2]), 65210];
}

#[test]
fn part1() {
    let f = File::open("input/day7.int").unwrap();
    let reader = io::BufReader::new(f);
    let mut ic = Program::new(reader);

    let p = Permutator::new(&vec![0, 1, 2, 3, 4]);
    let mut max = 0;
    for pp in p {
        let sig = amp(&mut ic, &pp);
        if sig > max {
            max = sig;
        }
        eprintln!["{:?}: {}", pp, sig];
    }
    assert_eq![max, 567045];
}
