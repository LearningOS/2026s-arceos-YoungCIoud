// extern crate hashbrown;
extern crate alloc;

use hashbrown::DefaultHashBuilder;

use core::{
    hash::{
        BuildHasher,
        Hash,
        Hasher,
    },
    mem::swap,
};

use alloc:: {
    vec::Vec,
    boxed::Box,
};

// FACTOR_SIZE * size <= FACTOR_CAPACITY * capacity
const FACTOR_SIZE: usize = 4;
const FACTOR_CAPACITY: usize = 3;

const DEFAULT_CAPACITY: usize = 16;

pub struct HashMap<K: Hash + Eq, V> {
    buckets: Vec<Option<Box<Entry<K, V>>>>,
    capacity: usize,
    size: usize,
}
pub struct Iter<'a, K: Hash + Eq, V> {
    buckets: &'a [Option<Box<Entry<K, V>>>],
    idx: usize,
    cur: Option<&'a Entry<K, V>>,
}

struct Entry<K: Hash + Eq, V> {
    key: K,
    val: V,
    hash: u64,
    next: Option<Box<Entry<K, V>>>
}

impl <K: Hash + Eq, V> HashMap<K, V> {
    /// creates an empty HashMap with a capacity of 16
    pub fn new() -> Self {
        let mut buckets = Vec::with_capacity(DEFAULT_CAPACITY);
        buckets.resize_with(DEFAULT_CAPACITY, || None);
        Self {
            buckets,
            capacity: DEFAULT_CAPACITY,
            size: 0,
        }
    }

    /// Returns the number of elements the map can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.size
    }

    /// Returns true if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory for reuse.
    pub fn clear(&mut self) {
        self.size = 0;
        for bucket in &mut self.buckets {
            if let Some(entry) = bucket.take() {
                drop(entry)
            }
        }
    }

    /// Inserts a key-value pair into the map.
    /// 
    /// If the map did not have this key present, None is returned.
    /// 
    /// If the map did have this key present, the value is updated, and the old value is returned.
    pub fn insert(&mut self, key: K, mut val: V) -> Option<V> {
        if let Some(old_val) = self.get_mut(&key) {
            swap(old_val, &mut val);
            Some(val)
        } else {
            // 插入新的键会使容量超出阈值
            if self.size * FACTOR_SIZE >= self.capacity * FACTOR_CAPACITY {
                self.double_capacity();
            }
            self.size += 1;

            let mut entry = Entry::new(key, val);
            let idx = entry.hash as usize % self.capacity;
            // entry.next = self.buckets[idx];
            if let Some(next) = self.buckets[idx].take() {
                entry.next = Some(next);
            }
            self.buckets[idx] = Some(Box::new(entry));
            None
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = Entry::<K, V>::hash_of_key(key);
        let idx = hash as usize % self.capacity;

        if let Some(entry) = self.buckets[idx].as_mut() {
            entry.get_mut(key)
        } else {
            None
        }
    }

    /// An iterator visiting all key-value pairs in arbitrary order. The iterator element type is (&'a K, &'a V).
    pub fn iter<'a> (&'a self) -> Iter<'a, K, V> {
        Iter {
            buckets: &self.buckets,
            idx: 0,
            cur: None,
        }
    }

    fn double_capacity(&mut self) {
        let mut tmp = Vec::new();
        for bucket in &mut self.buckets {
            if let Some(entry) = bucket.take() {
                tmp.push(entry);
            }
        }

        self.capacity *= 2;
        self.buckets.resize_with(self.capacity, || None);
        
        // 重哈希
        self.size = 0;
        tmp
            .into_iter()
            .for_each(|entry| {
                let mut cur = Some(entry);
                while let Some(mut entry) = cur {
                    cur = entry.next.take();
                    self.insert(entry.key, entry.val);
                }
            });
    }
}

impl <'a, K: Hash + Eq, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entry) = self.cur.take() {
                self.cur = entry.next.as_deref();
                return Some((&entry.key, &entry.val));
            }

            if self.idx >= self.buckets.len() {
                return None;
            }

            self.cur = self.buckets[self.idx].as_deref();
            self.idx += 1;
        }
    }
}


impl <K: Hash + Eq, V> Entry<K, V> {
    fn new(key: K, val: V) -> Self {
        let hash = Self::hash_of_key(&key);
        Entry {
            key,
            val,
            hash,
            next: None,
        }
    }

    fn hash_of_key(key: &K) -> u64 {
        let mut hasher = DefaultHashBuilder::default().build_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = Entry::<K, V>::hash_of_key(key);
        let mut cur = Some(self);
        while let Some(entry) = cur {
            if entry.hash == hash && &entry.key == key {
                return Some(&mut entry.val);
            }

            cur = entry
                .next
                .as_mut()
                .map(|node| node.as_mut());
        }
        None
    }

    fn get(&self, key: &K) -> Option<&V> {
        let hash = Entry::<K, V>::hash_of_key(key);
        let mut cur = Some(self);
        while let Some(entry) = cur {
            if entry.hash == hash && &entry.key == key {
                return Some(&entry.val);
            }

            cur = entry
                .next
                .as_ref()
                .map(|node| node.as_ref());
        }
        None
    }
}
