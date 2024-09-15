use super::object::Object;
use nohash_hasher::BuildNoHashHasher;
use std::collections::{HashMap, HashSet};

type BnhU64 = BuildNoHashHasher<u64>;

pub type YsetlSet = HashSet<Object, BnhU64>;

pub fn new_y_set_with_capacity(capacity: usize) -> YsetlSet {
    HashSet::with_capacity_and_hasher(capacity, BuildNoHashHasher::default())
}

pub fn new_y_set_from_vec(elements: Vec<Object>) -> YsetlSet {
    let mut set: YsetlSet =
        HashSet::with_capacity_and_hasher(elements.len(), BuildNoHashHasher::default());
    set.extend(elements);
    set
}

pub type YsetlMap = HashMap<Object, Object, BnhU64>;

pub fn new_y_map_with_capacity(capacity: usize) -> YsetlMap {
    HashMap::with_capacity_and_hasher(capacity, BuildNoHashHasher::default())
}
