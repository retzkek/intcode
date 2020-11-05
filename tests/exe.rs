use intcode::Intcode;
use std::io;

#[test]
fn test_add() {
    let code = io::Cursor::new("1,4,5,4,11,88");
    let mut ic = Intcode::new(code);
    ic.exe(0);
    assert_eq![ic.peek(4),99];
}
