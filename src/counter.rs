use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Default)]
pub struct Counter<K> {
    elems: HashMap<K, usize>,
    size: usize,
}

impl<K> Counter<K>
where
    K: Eq + Hash + Clone,
{
    pub fn inc(self: &mut Self, k: K) {
        self.elems.entry(k).and_modify(|e| *e += 1).or_insert(1);
        self.size += 1;
    }

    // Remove any elements that occur less than 10% of the times.
    pub fn remove_rare_elements(self: &mut Self) {
        self.elems.retain(|_, value| value >= &mut (self.size / 10));
        self.size = self.elems.iter().map(|pair| pair.1).sum();
    }

    pub fn values(self: &Self) -> std::collections::hash_map::Keys<K, usize> {
        self.elems.keys()
    }

    pub fn len(self: &Self) -> usize {
        self.elems.len()
    }
}

pub fn longest(counter: &Counter<String>) -> Result<&String> {
    counter
        .values()
        .sorted_by(|a, b| Ord::cmp(&a.len(), &b.len()))
        .next()
        .ok_or(anyhow!("Got unexpectedly no value!"))
}
