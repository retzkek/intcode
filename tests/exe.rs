use intcode::Program;
use std::io;

#[test]
fn test_add() {
    let code = io::Cursor::new("1,4,5,4,11,88");
    let mut ic = Program::new(code);
    let mut input = io::Cursor::new("");
    ic.exe(0, false, &mut input).expect("execution error");
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_mul() {
    let code = io::Cursor::new("2,4,5,4,3,33");
    let mut ic = Program::new(code);
    let mut input = io::Cursor::new("");
    ic.exe(0, false, &mut input).expect("execution error");
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_input() {
    let code = io::Cursor::new("3,2,0");
    let mut ic = Program::new(code);
    let mut input = io::Cursor::new("99");
    ic.exe(0, false, &mut input).expect("execution error");
    assert_eq![ic.peek(2), 99];
}
