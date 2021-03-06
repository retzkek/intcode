use std::convert::TryInto;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};

pub mod debugger;
pub mod permutations;

// the fundamental type of an Intcode program, used for both addresses and
// values (since one can easily become the other)
pub type Int = i64;

#[derive(Debug, PartialEq)]
enum Operation {
    End,
    Add,
    Mul,
    Input,
    Output,
    JumpNotZero,
    JumpZero,
    LessThan,
    EqualTo,
    RelBase,
}

impl Operation {
    pub fn len(&self) -> usize {
        match self {
            Operation::End => 1,
            Operation::Add => 4,
            Operation::Mul => 4,
            Operation::Input => 2,
            Operation::Output => 2,
            Operation::JumpNotZero => 3,
            Operation::JumpZero => 3,
            Operation::LessThan => 4,
            Operation::EqualTo => 4,
            Operation::RelBase => 2,
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:3}",
            match self {
                Operation::End => "END",
                Operation::Add => "ADD",
                Operation::Mul => "MUL",
                Operation::Input => "INP",
                Operation::Output => "OUT",
                Operation::JumpNotZero => "JNZ",
                Operation::JumpZero => "JZ",
                Operation::LessThan => "LT",
                Operation::EqualTo => "EQ",
                Operation::RelBase => "REL",
            }
        )
    }
}

#[derive(Debug, PartialEq)]
enum Mode {
    Pointer,
    Value,
    Relative,
}

trait Instruction {
    fn op(&self) -> Operation;
    fn modes(&self) -> Vec<Mode>;
}

impl Instruction for Int {
    fn op(&self) -> Operation {
        match self % 100 {
            99 => Operation::End,
            1 => Operation::Add,
            2 => Operation::Mul,
            3 => Operation::Input,
            4 => Operation::Output,
            5 => Operation::JumpNotZero,
            6 => Operation::JumpZero,
            7 => Operation::LessThan,
            8 => Operation::EqualTo,
            9 => Operation::RelBase,
            i => panic!["unknown Instruction {}", i],
        }
    }

    fn modes(&self) -> Vec<Mode> {
        let mut m: Vec<Mode> = Vec::new();
        let mut r = self / 100;
        for _ in 0..3 {
            m.push(match r % 10 {
                0 => Mode::Pointer,
                1 => Mode::Value,
                2 => Mode::Relative,
                _ => panic!["unknown mode {}", r],
            });
            r /= 10;
        }
        m
    }
}

#[cfg(test)]
mod test_instruction {
    use super::*;

    #[test]
    fn test_op_end() {
        let cells: Vec<Int> = vec![99, 1099, 11199];
        for c in cells {
            assert_eq!(c.op(), Operation::End, "cell value: {}", c);
        }
    }

    #[test]
    fn test_op_add() {
        let cells: Vec<Int> = vec![1, 101, 11101];
        for c in cells {
            assert_eq!(c.op(), Operation::Add, "cell value: {}", c);
        }
    }

    #[test]
    #[should_panic]
    fn test_op_other() {
        let c: Int = 10;
        c.op();
    }

    #[test]
    fn test_modes_000() {
        let c: Int = 99;
        assert_eq!(c.modes(), vec![Mode::Pointer, Mode::Pointer, Mode::Pointer])
    }

    #[test]
    fn test_modes_001() {
        let c: Int = 199;
        assert_eq!(c.modes(), vec![Mode::Value, Mode::Pointer, Mode::Pointer])
    }

    #[test]
    fn test_modes_100() {
        let c: Int = 10001;
        assert_eq!(c.modes(), vec![Mode::Pointer, Mode::Pointer, Mode::Value])
    }

    #[test]
    fn test_modes_102() {
        let c: Int = 10209;
        assert_eq!(c.modes(), vec![Mode::Relative, Mode::Pointer, Mode::Value])
    }

    #[test]
    #[should_panic]
    fn test_modes_other() {
        let c: Int = 399;
        c.modes();
    }
}

pub enum Input<'a> {
    None,
    String(&'a str),
    Reader(&'a mut dyn BufRead),
    Channel(Receiver<Int>),
}

pub enum Output<'a> {
    None,
    Writer(&'a mut dyn Write),
    Channel(Sender<Int>),
}

#[derive(Clone)]
pub struct StackEntry {
    address: Int,
    value: Int,
}

#[derive(Clone)]
pub struct Program {
    source: Vec<Int>,
    mem: Vec<Int>,
    rel_base: Int,
    stack: Vec<StackEntry>,
}

impl Program {
    pub fn new<R: BufRead>(reader: R) -> Program {
        let c = match Program::read_code(reader) {
            Ok(x) => x,
            Err(error) => panic!["{:}", error],
        };
        Program {
            source: c.clone(),
            mem: c,
            rel_base: 0,
            stack: Vec::new(),
        }
    }

    /// Read intcode from reader.
    fn read_code<R: BufRead>(reader: R) -> Result<Vec<Int>, io::Error> {
        let mut c: Vec<Int> = Vec::new();
        for l in reader.lines() {
            for s in l?.split(',') {
                if s.len() > 0 {
                    match Int::from_str(s) {
                        Ok(n) => c.push(n),
                        Err(error) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, error))
                        }
                    };
                }
            }
        }
        Ok(c)
    }

    /// Reset program memory to source.
    pub fn reset(&mut self) {
        self.mem = self.source.clone();
    }

    pub fn peek(&self, addr: Int) -> Int {
        let addr: usize = addr.try_into().unwrap();
        *self.mem.get(addr).unwrap()
    }

    pub fn poke(&mut self, addr: Int, value: Int) -> Option<Int> {
        let addr: usize = addr.try_into().unwrap();
        let mut old: Option<Int> = None;
        if addr > self.mem.len() - 1 {
            self.mem.resize_with(addr + 1, Default::default);
        } else {
            old = Some(self.mem[addr]);
        }
        self.mem[addr] = value;
        old
    }

    fn pval(&self, mode: &Mode, addr: Int) -> Int {
        match mode {
            Mode::Pointer => self.peek(addr),
            Mode::Value => addr,
            Mode::Relative => self.peek(addr + self.rel_base),
        }
    }

    fn paddr(&self, mode: &Mode, addr: Int) -> Int {
        match mode {
            Mode::Pointer => addr,
            Mode::Value => panic!["Value mode not valid for address"],
            Mode::Relative => addr + self.rel_base,
        }
    }

    pub fn rel_base(&self) -> Int {
        self.rel_base
    }

    pub fn exe(
        &mut self,
        addr: Int,
        trace: bool,
        mut input: Input,
        mut output: Output,
    ) -> io::Result<()> {
        let mut addr = addr;
        loop {
            match self.step(addr, trace, &mut input, &mut output) {
                Ok(-1) => break,
                Ok(r) => addr = r,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn step(
        &mut self,
        addr: Int,
        trace: bool,
        input: &mut Input,
        output: &mut Output,
    ) -> io::Result<Int> {
        let mut addr = addr;
        let v = self.peek(addr);
        let op = v.op();
        let modes = v.modes();
        self.stack.push(StackEntry {
            address: addr,
            value: v,
        });
        if trace {
            eprintln!["{}: {} ({:?})", addr, v, op];
        }
        match op {
            Operation::End => return Ok(-1),
            Operation::Add => {
                self.poke(
                    self.paddr(&modes[2], self.peek(addr + 3)),
                    self.pval(&modes[0], self.peek(addr + 1))
                        + self.pval(&modes[1], self.peek(addr + 2)),
                );
                addr += 4;
            }
            Operation::Mul => {
                self.poke(
                    self.paddr(&modes[2], self.peek(addr + 3)),
                    self.pval(&modes[0], self.peek(addr + 1))
                        * self.pval(&modes[1], self.peek(addr + 2)),
                );
                addr += 4;
            }
            Operation::Input => {
                let i = match input {
                    Input::String(s) => match Int::from_str(&s.trim()) {
                        Ok(n) => n,
                        Err(error) => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, error))
                        }
                    },
                    Input::Reader(ref mut r) => {
                        if let Output::Writer(ref mut w) = output {
                            w.write_all(b"?")?;
                            w.flush()?;
                        }
                        let mut s = String::new();
                        r.read_line(&mut s)?;
                        match Int::from_str(&s.trim()) {
                            Ok(n) => n,
                            Err(error) => {
                                return Err(io::Error::new(io::ErrorKind::InvalidData, error))
                            }
                        }
                    }
                    Input::Channel(ref c) => c.recv().unwrap(),
                    Input::None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "input required but no input channel provided",
                        ));
                    }
                };
                if trace {
                    eprintln!["input data: \"{}\"", i];
                }
                self.poke(self.paddr(&modes[0], self.peek(addr + 1)), i);
                addr += 2;
            }
            Operation::Output => {
                let o = self.pval(&modes[0], self.peek(addr + 1));
                if trace {
                    eprintln!["output data: \"{}\"", o];
                }
                match output {
                    Output::Writer(ref mut w) => {
                        writeln!(w, "{}", o)?;
                    }
                    Output::Channel(ref c) => {
                        c.send(o).unwrap();
                    }
                    Output::None => {}
                }
                addr += 2;
            }
            Operation::JumpNotZero => {
                if self.pval(&modes[0], self.peek(addr + 1)) != 0 {
                    addr = self.pval(&modes[1], self.peek(addr + 2));
                } else {
                    addr += 3;
                }
            }
            Operation::JumpZero => {
                if self.pval(&modes[0], self.peek(addr + 1)) == 0 {
                    addr = self.pval(&modes[1], self.peek(addr + 2));
                } else {
                    addr += 3;
                }
            }
            Operation::LessThan => {
                self.poke(
                    self.paddr(&modes[2], self.peek(addr + 3)),
                    if self.pval(&modes[0], self.peek(addr + 1))
                        < self.pval(&modes[1], self.peek(addr + 2))
                    {
                        1
                    } else {
                        0
                    },
                );
                addr += 4;
            }
            Operation::EqualTo => {
                self.poke(
                    self.paddr(&modes[2], self.peek(addr + 3)),
                    if self.pval(&modes[0], self.peek(addr + 1))
                        == self.pval(&modes[1], self.peek(addr + 2))
                    {
                        1
                    } else {
                        0
                    },
                );
                addr += 4;
            }
            Operation::RelBase => {
                self.rel_base += self.pval(&modes[0], self.peek(addr + 1));
                addr += 2;
            }
        }
        Ok(addr)
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.mem)
    }
}

#[cfg(test)]
mod test_intcode {
    use super::Program;
    use super::*;
    use std::io;

    #[test]
    fn test_read_code() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let r = vec![1, 0, 0, 3, 1, 1];
        assert_eq!(Program::read_code(code).unwrap(), r)
    }

    #[test]
    fn test_read_code_error() {
        let code = io::Cursor::new("1,0,a,3,1,1");
        assert_eq!(
            Program::read_code(code).map_err(|e| e.kind()),
            Err(io::ErrorKind::InvalidData)
        )
    }

    #[test]
    fn test_read_newline() {
        let code = io::Cursor::new(
            "1,0,0,
3,1,1",
        );
        let r = vec![1, 0, 0, 3, 1, 1];
        assert_eq!(Program::read_code(code).unwrap(), r)
    }

    #[test]
    fn test_new() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let ic = Program::new(code);

        let cv = vec![1, 0, 0, 3, 1, 1];
        assert_eq!(ic.mem, cv);
    }

    #[test]
    fn test_peek() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let ic = Program::new(code);
        assert_eq!(ic.peek(3), 3);
    }

    #[test]
    fn test_poke() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let mut ic = Program::new(code);
        assert_eq!(ic.peek(3), 3);
        assert_eq!(ic.poke(3, 5).unwrap(), 3);
        assert_eq!(ic.peek(3), 5);
    }

    #[test]
    fn test_reset() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let mut ic = Program::new(code);
        assert_eq!(ic.peek(3), 3);
        assert_eq!(ic.poke(3, 5).unwrap(), 3);
        assert_eq!(ic.peek(3), 5);
        ic.reset();
        assert_eq!(ic.peek(3), 3);
    }

    #[test]
    fn test_pval() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let mut ic = Program::new(code);
        assert_eq!(ic.pval(&Mode::Pointer, 4), 1);
        assert_eq!(ic.pval(&Mode::Value, 4), 4);
        assert_eq!(ic.pval(&Mode::Relative, 3), 3);
        ic.rel_base = 1;
        assert_eq!(ic.pval(&Mode::Relative, 3), 1);
    }

    #[test]
    fn test_paddr() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let mut ic = Program::new(code);
        assert_eq!(ic.paddr(&Mode::Pointer, 4), 4);
        assert_eq!(ic.paddr(&Mode::Relative, 3), 3);
        ic.rel_base = 1;
        assert_eq!(ic.paddr(&Mode::Relative, 3), 4);
    }
}
