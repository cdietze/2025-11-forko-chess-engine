use crate::r#move::Move;
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
    pub score: i32,
    pub flag: NodeType,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    pub map: HashMap<u64, TTEntry>,
    pub max_entries: usize,
}

impl TranspositionTable {
    pub fn new(max_entries: usize) -> Self {
        Self {
            map: HashMap::with_capacity(max_entries.min(1 << 20)),
            max_entries,
        }
    }

    #[inline]
    pub fn get(&self, key: u64) -> Option<&TTEntry> {
        self.map.get(&key)
    }

    #[inline]
    pub fn store(&mut self, entry: TTEntry) {
        if self.map.len() >= self.max_entries {
            // Very simple aging: clear the table when full.
            self.map.clear();
        }
        // Replace only if deeper or not present
        match self.map.get(&entry.key) {
            Some(old) if old.depth > entry.depth => (),
            _ => {
                self.map.insert(entry.key, entry);
            }
        }
    }
}
