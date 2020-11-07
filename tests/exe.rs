use intcode::Program;
use std::io;

#[test]
fn test_add() {
    let code = io::Cursor::new("1,4,5,4,11,88");
    let mut ic = Program::new(code);
    ic.exe(0, false, &mut io::empty(), &mut io::sink())
        .expect("execution error");
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_mul() {
    let code = io::Cursor::new("2,4,5,4,3,33");
    let mut ic = Program::new(code);
    ic.exe(0, false, &mut io::empty(), &mut io::sink())
        .expect("execution error");
    assert_eq![ic.peek(4), 99];
}

#[test]
fn test_input() {
    let code = io::Cursor::new("3,2,0");
    let mut ic = Program::new(code);
    let mut input = io::Cursor::new("99");
    ic.exe(0, false, &mut input, &mut io::sink())
        .expect("execution error");
    assert_eq![ic.peek(2), 99];
}

#[test]
fn test_output() {
    let code = io::Cursor::new("4,2,99");
    let mut ic = Program::new(code);
    let mut output = io::Cursor::new(vec![0; 3]);
    ic.exe(0, false, &mut io::empty(), &mut output)
        .expect("execution error");
    assert_eq![output.get_ref(), b"99\n"];
}
