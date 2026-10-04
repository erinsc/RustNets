#[derive(Debug)]
enum Entry<T> {
    Full(T),
    Empty(usize)
}
#[derive(Debug)]
pub struct SparseVec<T> {
    free: usize,
    size: usize, // Number of currently held elements
    storage: Vec<Entry<T>>,
}

impl<T> SparseVec<T> {
    pub fn new() -> SparseVec<T> {
        Self::with_capacity(0)
    }
    pub fn with_capacity(capacity: usize) -> SparseVec<T> {
        let storage = (0..capacity)
        .map(|i| Entry::Empty(i+1))
        .collect();

        Self { free: 0, size: 0, storage}
    }
    pub fn count(&self) -> usize {
        self.size
    }
    pub fn insert(&mut self, value: T) -> usize {
        if self.free == self.storage.len() {
            let additional = self.storage.len() / 2 + 1;

            let extension = (0..additional)
            .map(|i| Entry::Empty(i + self.free + 1));

            self.storage.extend(extension);
        }
        let id = self.free;
        let entry = Entry::Full(value);
        let out = std::mem::replace(&mut self.storage[id], entry);
        self.free = match out {
            Entry::Full(_) => unreachable!("The free stack should not hold entries"),
            Entry::Empty(free) => free
        };
        self.size += 1;
        id
    }
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        let current = self.storage.get(idx)?;
        match current {
            Entry::Full(_) => {},
            Entry::Empty(_) => return None
        }
        let entry = Entry::Empty(self.free);
        self.free = idx;
        let out = std::mem::replace(&mut self.storage[idx], entry);
        self.size -= 1;

        match out {
            Entry::Full(data) => Some(data),
            Entry::Empty(_) => unreachable!()
        }
    }
    pub fn get(&self, idx: usize) -> Option<&T> {
        let out = self.storage.get(idx)?;
        match out {
            Entry::Full(data) => Some(data),
            Entry::Empty(_) => None
        }
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        let out = self.storage.get_mut(id)?;
        match out {
            Entry::Full(data) => Some(data),
            Entry::Empty(_) => None
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.storage.iter()
            .enumerate()
            .filter_map(|(i, entry)|
            if let Entry::Full(data) = entry {
                Some((i, data))
            } else {
                None
            })
            
    }
}

impl<T> std::ops::Index<usize> for SparseVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let out = &self.storage[index];
        match out {
            Entry::Full(data) => data,
            Entry::Empty(_) => panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let sv: SparseVec<u32> = SparseVec::new();
        assert_eq!(sv.iter().count(), 0);
        assert_eq!(sv.get(0), None);
    }
    #[test]
    fn with_capacity_is_empty() {
        let sv: SparseVec<u32> = SparseVec::with_capacity(10);
        assert_eq!(sv.get(5), None);
    }

    #[test]
    fn insert_and_get() {
        let mut sv = SparseVec::new();
        let id = sv.insert(42);
        assert_eq!(sv.get(id), Some(&42));
    }

    #[test]
    fn mutate() {
        let mut sv = SparseVec::new();
        let id = sv.insert(10);
        if let Some(v) = sv.get_mut(id) {
            *v += 5;
        }
        assert_eq!(sv.get(id), Some(&15));
    }

    #[test]
    fn indexing() {
        let mut sv = SparseVec::new();
        let id = sv.insert(42);
        assert_eq!(sv[id], 42);
    }

    #[test]
    #[should_panic]
    fn indexing_panic_removed() {
        let mut sv = SparseVec::new();
        let id = sv.insert(42);
        sv.remove(id);
        let _ = sv[id];
    }

    #[test]
    #[should_panic]
    fn index_panic_oob() {
        let sv: SparseVec<i32> = SparseVec::new();
        let _ = sv[5];
    }

    #[test]
    fn get_oob() {
        let sv: SparseVec<i32> = SparseVec::new();
        assert_eq!(sv.get(10), None);
    }

    #[test]
    fn remove_oob() {
        let mut sv: SparseVec<i32> = SparseVec::new();
        assert_eq!(sv.remove(10), None);
    }

    #[test]
    fn remove_twice() {
        let mut sv = SparseVec::new();
        let id = sv.insert(7);
        assert_eq!(sv.remove(id), Some(7));
        assert_eq!(sv.remove(id), None);
    }

    #[test]
    fn reuse() {
        let mut sv = SparseVec::new();
        let a = sv.insert("a");
        let b = sv.insert("b");
        assert_eq!(sv.remove(a), Some("a"));
        let c = sv.insert("c");
        assert_eq!(c, a); // freed slot should be reused
        assert_eq!(sv.get(b), Some(&"b"));
        assert_eq!(sv.get(c), Some(&"c"));
    }

    #[test]
    fn iter_occupied() {
        let mut sv = SparseVec::with_capacity(4);
        let a = sv.insert(1);
        let b = sv.insert(2);
        let c = sv.insert(3);
        sv.remove(b);
        let mut values: Vec<_> = sv.iter().collect();
        values.sort();
        assert_eq!(values, vec![(a, &1), (c, &3)]);
    }
}