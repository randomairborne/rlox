#[derive(Clone, Debug, Default)]
pub struct RunLengthEncoded<T: std::fmt::Debug + Clone + Copy + Default + PartialEq + Eq> {
    inner: Vec<Run<T>>,
}

impl<T: std::fmt::Debug + Clone + Copy + Default + PartialEq + Eq> RunLengthEncoded<T> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }
    pub fn push(&mut self, value: T) {
        if let Some(last) = self.inner.last_mut() {
            if value == last.data {
                last.len += 1;
                return;
            }
        }
        self.inner.push(Run {
            len: 1,
            data: value,
        });
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        let mut last_index = 0;
        for item in &self.inner {
            let next_index = last_index + item.len;
            if index >= last_index && index < next_index {
                return Some(&item.data);
            }
            last_index = next_index;
        }
        None
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let mut last_index = 0;
        for item in &mut self.inner {
            let next_index = last_index + item.len;
            if index >= last_index && index < next_index {
                return Some(&mut item.data);
            }
            last_index = next_index;
        }
        None
    }
}

impl<T: std::fmt::Debug + Clone + Copy + Default + PartialEq + Eq> std::ops::Index<usize>
    for RunLengthEncoded<T>
{
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(v) = self.get(index) {
            v
        } else {
            panic!("Index {index} out of bounds");
        }
    }
}

impl<T: std::fmt::Debug + Clone + Copy + Default + PartialEq + Eq> std::ops::IndexMut<usize>
    for RunLengthEncoded<T>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(v) = self.get_mut(index) {
            v
        } else {
            panic!("Index {index} out of bounds");
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Run<T: std::fmt::Debug + Clone + Copy + Default + PartialEq + Eq> {
    pub len: usize,
    pub data: T,
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;
    #[test]
    fn basic() {
        let test_array = [1, 1, 1, 2, 3, 3, 3, 1, 1, 2, 1, 1];
        let mut rle: RunLengthEncoded<usize> = RunLengthEncoded::new();
        for item in &test_array {
            rle.push(*item);
        }
        for (index, item) in test_array.into_iter().enumerate() {
            assert_eq!(rle[index], item);
        }
    }
    #[test]
    fn randomized() {
        let mut test_array = Vec::with_capacity(1000);
        let mut rng = rand::thread_rng();
        for _ in 1..=1000 {
            test_array.push(rng.gen_range(1..5))
        }
        let mut rle: RunLengthEncoded<usize> = RunLengthEncoded::new();
        for item in &test_array {
            rle.push(*item);
        }
        for (index, item) in test_array.into_iter().enumerate() {
            assert_eq!(rle[index], item);
        }
    }
}
