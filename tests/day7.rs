//! AoC 2019 day 7: https://adventofcode.com/2019/day/7

extern crate intcode;
use intcode::permutations::Permutator;
use std::fs::File;
use std::io;
use std::io::Write;
use std::str::FromStr;

fn strip_output(output: &[u8]) -> &[u8] {
    let e = output.iter().position(|&x| x == b'\0').unwrap();
    // there will be two ? at the front, and \n at the end
    &output[2..e - 1]
}

fn amp(program: &mut intcode::Program, phases: &[u8]) -> i64 {
    let mut inp = io::Cursor::new(vec![0; 10]);
    let mut out = io::Cursor::new(vec![0; 10]);
    out.write(b"??0\n").unwrap();
    for p in phases.iter() {
        inp.set_position(0);
        inp.write(&[*p]).unwrap();
        inp.write(b"\n").unwrap();
        inp.write(&strip_output(out.get_ref())).unwrap();
        inp.write(b"\n").unwrap();
        inp.set_position(0);
        out.set_position(0);
        program
            .exe(0, false, &mut inp, &mut out)
            .expect("execution error");
    }
    i64::from_str(std::str::from_utf8(strip_output(out.get_ref())).unwrap()).unwrap()
}

#[test]
fn test_amp_1() {
    let code = io::Cursor::new("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0");
    let mut ic = intcode::Program::new(code);
    assert_eq![amp(&mut ic, b"43210"), 43210];
}

#[test]
fn test_amp_2() {
    let code =
        io::Cursor::new("3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0");
    let mut ic = intcode::Program::new(code);
    assert_eq![amp(&mut ic, b"01234"), 54321];
}

#[test]
fn test_amp_3() {
    let code =
        io::Cursor::new("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0");
    let mut ic = intcode::Program::new(code);
    assert_eq![amp(&mut ic, b"10432"), 65210];
}

#[test]
fn part1() {
    let f = File::open("input/day7.int").unwrap();
    let reader = io::BufReader::new(f);
    let mut ic = intcode::Program::new(reader);

    let p = Permutator::new(b"01234");
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
