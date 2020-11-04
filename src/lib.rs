use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Intcode {
    code: Vec<u32>,
    mem: HashMap<usize, u32>,
    rel_base: u32,
}

/// Read intcode from reader.
fn read_code<R: BufRead>(reader: R) -> Result<Vec<u32>,io::Error> {
    let mut c: Vec<u32> = Vec::new();
    for l in reader.lines() {
        for s in l?.split(',') {
            match u32::from_str(s) {
                Ok(n) => c.push(n),
                Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData,error)),
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
fn vec_to_map<T: Copy>(code: &Vec<T>) -> HashMap<usize,T> {
    let mut m = HashMap::new();
    for (k,v) in (0..).zip(code.iter()) {
        m.insert(k,v.clone());
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

}


#[cfg(test)]
mod tests {
    use super::*;
    use super::Intcode;
    use std::io;

    #[test]
    fn test_read_code() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let r = vec![1,0,0,3,1,1];
        assert_eq!(read_code(code).unwrap(),r)
    }

    #[test]
    fn test_read_code_error() {
        let code = io::Cursor::new("1,0,a,3,1,1");
        assert_eq!(read_code(code).map_err(|e| e.kind()),
                   Err(io::ErrorKind::InvalidData))
    }

    #[test]
    fn test_new() {
        let code = io::Cursor::new("1,0,0,3,1,1");
        let ic = Intcode::new(code);

        let cv = vec![1,0,0,3,1,1];
        assert_eq!(ic.code,cv);

        let mut m = HashMap::new();
        m.insert(0,1);
        m.insert(1,0);
        m.insert(2,0);
        m.insert(3,3);
        m.insert(4,1);
        m.insert(5,1);
        assert_eq!(ic.mem,m);
    }

    #[test]
    fn test_vec_to_map() {
        let r = vec![1,0,0,3];
        let mut exp = HashMap::new();
        exp.insert(0,1);
        exp.insert(1,0);
        exp.insert(2,0);
        exp.insert(3,3);
        assert_eq!(vec_to_map(&r),exp);
    }
}
