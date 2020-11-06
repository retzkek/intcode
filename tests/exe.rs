use intcode::Program;
use std::io;

#[test]
fn test_add() {
    let code = io::Cursor::new("1,4,5,4,11,88");
    let mut ic = Program::new(code);
    ic.exe(0, false);
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_mul() {
    let code = io::Cursor::new("2,4,5,4,3,33");
    let mut ic = Program::new(code);
    ic.exe(0, false);
    assert_eq![ic.peek(4), 99];
}
