use std::collections::HashMap;
use std::io::prelude::*;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Intcode {
    code: Vec<u32>,
    mem: HashMap<u32, u32>,
    rel_base: u32,
}

fn read_code<R: BufRead>(reader: R) -> Option<Vec<u32>> {
    let mut c: Vec<u32> = Vec::new();
    for l in reader.lines() {
        for s in l.ok()?.split(',') {
            c.push(u32::from_str(s).ok()?)
        }
    }
    Some(c)
    // tried to do it functionally, too hard to propogate errors!
    // leaving this here in case I learn there's an easy way to do it.
    //reader.lines().
    // map(|l| l.unwrap().split(',').
    // map(|s| u32::from_str(s).unwrap())).
    // flatten().
    // collect::<Vec<u32>>()
}

impl Intcode {

    pub fn new<R: BufRead>(reader: R) -> Intcode {
        Intcode {
            code: read_code(reader).unwrap(),
            mem: HashMap::new(),
            rel_base: 0,
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use super::Intcode;
    use std::io::Cursor;

    #[test]
    fn test_read_code() {
        let code = Cursor::new("1,0,0,3,1,1");
        let r = vec![1,0,0,3,1,1];
        assert_eq!(read_code(code).unwrap(),r)
    }

    #[test]
    fn test_read_code_error() {
        let code = Cursor::new("1,0,a,3,1,1");
        assert_eq!(read_code(code), None)
    }

    #[test]
    fn test_new() {
        let code = Cursor::new("1,0,0,3,1,1");
        let r = vec![1,0,0,3,1,1];
        let ic = Intcode::new(code);
        assert_eq!(ic.code,r)
    }
}
