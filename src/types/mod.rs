use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;

#[derive(Debug)]
pub struct TileMapping<N: Eq + Hash>(pub HashMap<N, Vec<N>>);

impl <N>TileMapping<N>
    where N: Eq + Hash {
    pub fn contains(&self, y: &N, x: &N) -> bool {
        if let Some(xs) = self.0.get(y) {
            if xs.contains(x) {
                return true
            }
        }

        return false
    }

    pub fn contains_key(&self, key: &N) -> bool {
        self.0.contains_key(key)
    }

    pub fn get(&self, key: &N) -> Option<&Vec<N>> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &N) -> Option<&mut Vec<N>> {
        self.0.get_mut(key)
    }

    pub fn insert(&mut self, key: N, value: Vec<N>) {
        self.0.insert(key, value);
    }
}