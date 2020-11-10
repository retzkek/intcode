use intcode::{Input, Int, Output, Program};
use std::io;
use std::sync::mpsc::channel;

#[test]
fn test_add() {
    let code = io::Cursor::new("1,4,5,4,11,88");
    let mut ic = Program::new(code);
    ic.exe(0, false, Input::None, Output::None)
        .expect("execution error");
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_mul() {
    let code = io::Cursor::new("2,4,5,4,3,33");
    let mut ic = Program::new(code);
    ic.exe(0, false, Input::None, Output::None)
        .expect("execution error");
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_input() {
    let code = io::Cursor::new("3,2,0");
    let mut ic = Program::new(code);
    let mut input = io::Cursor::new("99");
    ic.exe(0, false, Input::Reader(&mut input), Output::None)
        .expect("execution error");
    assert_eq![ic.peek(2), 99];
}

#[test]
fn test_input2() {
    let code = io::Cursor::new("3,0,3,4,0");
    let mut ic = Program::new(code);
    let mut input = io::Cursor::new("-1\n99\n");
    ic.exe(0, false, Input::Reader(&mut input), Output::None)
        .expect("execution error");
    assert_eq![ic.peek(0), -1];
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_input_chan() {
    let code = io::Cursor::new("3,2,0");
    let mut ic = Program::new(code);
    let (tx, rx) = channel::<Int>();
    tx.send(99).unwrap();
    ic.exe(0, false, Input::Channel(rx), Output::None)
        .expect("execution error");
    assert_eq![ic.peek(2), 99];
}

#[test]
fn test_output() {
    let code = io::Cursor::new("4,2,99");
    let mut ic = Program::new(code);
    let mut output = io::Cursor::new(vec![0; 3]);
    ic.exe(0, false, Input::None, Output::Writer(&mut output))
        .expect("execution error");
    assert_eq![output.get_ref(), b"99\n"];
}

#[test]
fn test_output_chan() {
    let code = io::Cursor::new("4,2,99");
    let mut ic = Program::new(code);
    let (tx, rx) = channel::<Int>();
    ic.exe(0, false, Input::None, Output::Channel(tx))
        .expect("execution error");
    assert_eq![rx.recv().unwrap(), 99];
}

#[test]
fn test_jnz() {
    let code = io::Cursor::new("5,0,4,99,4,6,99");
    let mut ic = Program::new(code);
    let mut output = io::Cursor::new(vec![0; 3]);
    ic.exe(0, false, Input::None, Output::Writer(&mut output))
        .expect("execution error");
    assert_eq![output.get_ref(), b"99\n"];
}

#[test]
fn test_jz() {
    let code = io::Cursor::new("106,0,4,99,4,6,99");
    let mut ic = Program::new(code);
    let mut output = io::Cursor::new(vec![0; 3]);
    ic.exe(0, false, Input::None, Output::Writer(&mut output))
        .expect("execution error");
    assert_eq![output.get_ref(), b"99\n"];
}

#[test]
fn test_rel_base() {
    let code = io::Cursor::new("9,0,99");
    let mut ic = Program::new(code);
    ic.exe(0, false, Input::None, Output::None)
        .expect("execution error");
    assert_eq![ic.rel_base(), 9];
}
