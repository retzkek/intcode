/// implementation of Heap's algorithm
/// https://en.wikipedia.org/wiki/Heap%27s_algorithm
pub struct Permutator<T> {
    last: Vec<T>,
    i: usize,
    state: Vec<usize>,
    count: usize,
}

impl<T: Clone> Permutator<T> {
    pub fn new(v: &[T]) -> Permutator<T> {
        Permutator {
            last: v.to_vec(),
            i: 0,
            state: vec![0; v.len()],
            count: 0,
        }
    }
}

impl<T: Clone> Iterator for Permutator<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count == 1 {
            return Some(self.last.clone());
        }
        if self.i >= self.last.len() {
            return None;
        }
        while self.state[self.i] >= self.i {
            self.state[self.i] = 0;
            self.i += 1;
            if self.i >= self.last.len() {
                return None;
            }
        }
        if self.i % 2 == 0 {
            self.last.swap(self.i, 0);
        } else {
            self.last.swap(self.state[self.i], self.i);
        }
        self.state[self.i] += 1;
        self.i = 0;
        Some(self.last.clone())
    }
}

#[test]
fn test_permutations() {
    let expected = vec![
        vec![0, 1, 2, 3],
        vec![1, 0, 2, 3],
        vec![2, 0, 1, 3],
        vec![0, 2, 1, 3],
        vec![1, 2, 0, 3],
        vec![2, 1, 0, 3],
        vec![3, 1, 0, 2],
        vec![1, 3, 0, 2],
        vec![0, 3, 1, 2],
        vec![3, 0, 1, 2],
        vec![1, 0, 3, 2],
        vec![0, 1, 3, 2],
        vec![0, 2, 3, 1],
        vec![2, 0, 3, 1],
        vec![3, 0, 2, 1],
        vec![0, 3, 2, 1],
        vec![2, 3, 0, 1],
        vec![3, 2, 0, 1],
        vec![3, 2, 1, 0],
        vec![2, 3, 1, 0],
        vec![1, 3, 2, 0],
        vec![3, 1, 2, 0],
        vec![2, 1, 3, 0],
        vec![1, 2, 3, 0],
    ];
    let p = Permutator::new(&vec![0, 1, 2, 3]);
    let mut count = 0;
    for pp in p {
        eprintln!["{:?}", pp];
        assert_eq![pp, expected[count]];
        count += 1;
    }
    assert_eq![count, 24];
}
