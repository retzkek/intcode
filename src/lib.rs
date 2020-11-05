use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Operation {
    End,
    Add,
    Mul,
    Input,
    Output,
    JumpNotEq,
    JumpEq,
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

#[derive(Debug, Clone, Copy)]
struct Address(usize);

impl Address {
    pub fn as_cell(&self) -> Cell {
        Cell(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
struct Cell(usize);

impl Cell {
     pub fn as_address(&self) -> Address {
         Address(self.0)
     }

    pub fn op(&self) -> Operation {
        match self.0 % 100 {
            99 => Operation::End,
            1 => Operation::Add,
            2 => Operation::Mul,
            3 => Operation::Input,
            4 => Operation::Output,
            5 => Operation::JumpNotEq,
            6 => Operation::JumpEq,
            7 => Operation::LessThan,
            8 => Operation::EqualTo,
            9 => Operation::RelBase,
            i => panic!["unknown Instruction {}", i],
        }
    }

    pub fn modes(&self) -> Vec<Mode> {
        let mut m: Vec<Mode> = Vec::new();
        let mut r = self.0 / 100;
        for _ in 0..3 {
            m.push(match r % 10 {
                0 => Mode::Pointer,
                1 => Mode::Value,
                2 => Mode::Relative,
                _ => panic!["unknown mode {}", r],
            });
            r = r / 10;
        }
        m
    }
}

#[cfg(test)]
mod test_instruction {
    use super::*;

    #[test]
    fn test_op_end() {
        let cells: Vec<Cell> = vec![99, 1099, 11199];
        for c in cells {
            assert_eq!(c.op(), Operation::End, "cell value: {}", c);
        }
    }

    #[test]
    fn test_op_add() {
        let cells: Vec<Cell> = vec![1, 101, 11101];
        for c in cells {
            assert_eq!(c.op(), Operation::Add, "cell value: {}", c);
        }
    }

    #[test]
    #[should_panic]
    fn test_op_other() {
        let c: Cell = 10;
        c.op();
    }

    #[test]
    fn test_modes_000() {
        let c: Cell = 99;
        assert_eq!(c.modes(), vec![Mode::Pointer, Mode::Pointer, Mode::Pointer])
    }

    #[test]
    fn test_modes_001() {
        let c: Cell = 199;
        assert_eq!(c.modes(), vec![Mode::Value, Mode::Pointer, Mode::Pointer])
    }

    #[test]
    fn test_modes_100() {
        let c: Cell = 10001;
        assert_eq!(c.modes(), vec![Mode::Pointer, Mode::Pointer, Mode::Value])
    }

    #[test]
    fn test_modes_102() {
        let c: Cell = 10209;
        assert_eq!(c.modes(), vec![Mode::Relative, Mode::Pointer, Mode::Value])
    }

    #[test]
    #[should_panic]
    fn test_modes_other() {
        let c: Cell = 399;
        c.modes();
    }
}

#[derive(Debug, Clone)]
pub struct Intcode {
    code: Vec<Cell>,
    mem: HashMap<Address, Cell>,
    rel_base: Address,
}

/// Read intcode from reader.
fn read_code<R: BufRead>(reader: R) -> Result<Vec<Cell>, io::Error> {
    let mut c: Vec<Cell> = Vec::new();
    for l in reader.lines() {
        for s in l?.split(',') {
            match usize::from_str(s) {
                Ok(n) => c.push(Cell(n)),
                Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, error)),
            };
        }
    }
    Ok(c)
    // tried to do it functionally, too hard to propogate errors!
    // leaving this here in case I learn there's an easy way to do it.
    //reader.lines().
    // map(|l| l.unwrap().split(',').
    // map(|s| u32::from_str(s).unwrap())).
    // flatten().
    // collect::<Vec<u32>>()
}

/// Copy Vec to HashMap, where each element's index is its key.
fn vec_to_map<T: Copy>(code: &Vec<T>) -> HashMap<Address, T> {
    let mut m = HashMap::new();
    for (k, v) in (0..).zip(code.iter()) {
        m.insert(k, v);
    }
    m
}

impl Intcode {
    pub fn new<R: BufRead>(reader: R) -> Intcode {
        let c = read_code(reader).unwrap();
        let v = vec_to_map(&c);
        Intcode {
            code: c,
            mem: v,
            rel_base: 0,
        }
    }

    pub fn peek(&self, addr: &Address) -> Option<&Cell>{
        self.mem.get(addr)
    }

    pub fn poke(&mut self, addr: Address, value: Cell) -> Option<Cell> {
        self.mem.insert(addr, value)
    }

    fn pval(&self, mode: &Mode, addr: &Address) -> Cell {
        match mode {
            Mode::Pointer => *self.peek(addr).unwrap(),
            Mode::Value => addr.as_cell(),
            Mode::Relative => *self.peek(addr + self.rel_base).unwrap(),
        }
    }

    fn paddr(&self, mode: &Mode, addr: &Address) -> Address {
        match mode {
            Mode::Pointer => *addr,
            Mode::Value => panic!["Value mode not valid for address"],
            Mode::Relative => *addr + self.rel_base,
        }
    }

    pub fn exe(&mut self, addr:Address) {
        let mut addr = addr;
        loop {
            println!["{}: {}",addr,self.peek(addr).unwrap()];
            let v = self.peek(addr).unwrap();
            let op = v.op();
            let modes = v.modes();
            match op {
                Operation::End => break,
                Operation::Add => {
                    self.poke(self.paddr(&modes[2], &self.peek(addr+3).unwrap().as_address()),
                              self.pval(&modes[0], &self.peek(addr+1).unwrap().as_address()) +
                              self.pval(&modes[1], &self.peek(addr+2).unwrap().as_address()));
                    addr += 4;
                },
                Operation::Mul => addr += 4,
                Operation::Input => addr += 2,
                Operation::Output => addr += 2,
                Operation::JumpNotEq => panic!["JumpNotEq not implemented"],
                Operation::JumpEq => panic!["JumpEq not implemented"],
                Operation::LessThan => addr += 4,
                Operation::EqualTo => addr += 4,
                Operation::RelBase => addr += 2,
            }
        }
    }
}

#[cfg(test)]
mod test_intcode {
    use super::Intcode;
    use super::*;
    use std::io;

    #[test]
    fn test_read_code() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let r = vec![1, 0, 0, 3, 1, 1];
        assert_eq!(read_code(code).unwrap(), r)
    }

    #[test]
    fn test_read_code_error() {
        let code = io::Cursor::new("1,0,a,3,1,1");
        assert_eq!(
            read_code(code).map_err(|e| e.kind()),
            Err(io::ErrorKind::InvalidData)
        )
    }

    #[test]
    fn test_new() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let ic = Intcode::new(code);

        let cv = vec![1, 0, 0, 3, 1, 1];
        assert_eq!(ic.code, cv);

        let mut m = HashMap::new();
        m.insert(0, 1);
        m.insert(1, 0);
        m.insert(2, 0);
        m.insert(3, 3);
        m.insert(4, 1);
        m.insert(5, 1);
        assert_eq!(ic.mem, m);
    }

    #[test]
    fn test_vec_to_map() {
        let r = vec![1, 0, 0, 3];
        let mut exp = HashMap::new();
        exp.insert(0, 1);
        exp.insert(1, 0);
        exp.insert(2, 0);
        exp.insert(3, 3);
        assert_eq!(vec_to_map(&r), exp);
    }

    #[test]
    fn test_peek() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let ic = Intcode::new(code);
        assert_eq!(*ic.peek(3).unwrap(), 3);
    }

    #[test]
    #[should_panic]
    fn test_peek_invalid() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let ic = Intcode::new(code);
        ic.peek(6).unwrap();
    }

    #[test]
    fn test_poke() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let mut ic = Intcode::new(code);
        assert_eq!(*ic.peek(3).unwrap(), 3);
        assert_eq!(ic.poke(3, 5).unwrap(), 3);
        assert_eq!(*ic.peek(3).unwrap(), 5);
    }

    #[test]
    fn test_pval() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let mut ic = Intcode::new(code);
        assert_eq!(ic.pval(Mode::Pointer, 4), 1);
        assert_eq!(ic.pval(Mode::Value, 4), 4);
        assert_eq!(ic.pval(Mode::Relative, 3), 3);
        ic.rel_base = 1;
        assert_eq!(ic.pval(Mode::Relative, 3), 1);
    }

    #[test]
    fn test_paddr() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let mut ic = Intcode::new(code);
        assert_eq!(ic.paddr(Mode::Pointer, 4), 4);
        assert_eq!(ic.paddr(Mode::Relative, 3), 3);
        ic.rel_base = 1;
        assert_eq!(ic.paddr(Mode::Relative, 3), 4);
    }
}
