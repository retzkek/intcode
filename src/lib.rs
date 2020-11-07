use std::convert::TryInto;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;

pub mod permutations;

// the fundamental type of an Intcode program, used for both addresses and
// values (since one can easily become the other)
type Int = i64;

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

pub struct Program {
    source: Vec<Int>,
    mem: Vec<Int>,
    rel_base: Int,
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
        }
    }

    /// Read intcode from reader.
    fn read_code<R: BufRead>(reader: R) -> Result<Vec<Int>, io::Error> {
        let mut c: Vec<Int> = Vec::new();
        for l in reader.lines() {
            for s in l?.split(',') {
                match Int::from_str(s) {
                    Ok(n) => c.push(n),
                    Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, error)),
                };
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

    pub fn exe<I: BufRead, O: Write>(
        &mut self,
        addr: Int,
        trace: bool,
        input: I,
        output: &mut O,
    ) -> io::Result<()> {
        let mut addr = addr;
        let mut input_lines = input.lines();
        loop {
            let v = self.peek(addr);
            let op = v.op();
            let modes = v.modes();
            if trace {
                eprintln!["{}: {} ({:?})", addr, v, op];
            }
            match op {
                Operation::End => break,
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
                    output.write_all(b"?")?;
                    output.flush()?;
                    let s = input_lines.next().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "no more data to read")
                    })??;
                    if trace {
                        eprintln!["input data: \"{}\"", s];
                    }
                    self.poke(
                        self.paddr(&modes[0], self.peek(addr + 1)),
                        match Int::from_str(&s) {
                            Ok(n) => n,
                            Err(error) => {
                                return Err(io::Error::new(io::ErrorKind::InvalidData, error))
                            }
                        },
                    );
                    addr += 2;
                }
                Operation::Output => {
                    if trace {
                        eprintln![
                            "output data: \"{}\"",
                            self.pval(&modes[0], self.peek(addr + 1))
                        ];
                    }
                    writeln!(output, "{}", self.pval(&modes[0], self.peek(addr + 1)))?;
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
        }
        Ok(())
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
