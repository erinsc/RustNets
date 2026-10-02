enum Entry<T> {
    Full(T),
    Empty(usize)
}

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

        Self { free: 0, size: capacity, storage}
    }
    pub fn insert(&mut self, value: T) -> usize {
        if self.free == self.storage.len() {
            let additional = self.storage.len() / 2 + 1;

            let extension = (0..additional)
            .map(|i| Entry::Empty(i+self.free));

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
    pub fn iter(&mut self) -> impl Iterator<Item = &T> {
        self.storage.iter().filter_map(|entry|
            if let Entry::Full(data) = entry {
                Some(data)
            } else {
                None
            }
        )
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