pub mod fec_set;
pub mod reed_solomon;
pub mod slot_accumulator;

use crate::shred::ParsedShred;
use fec_set::FecSet;
use slot_accumulator::SlotAccumulator;
use std::collections::{HashMap, HashSet};

pub struct CompletedSlot {
    pub slot: u64,
    pub data: Vec<u8>,
    pub fec_set_count: usize,
}

pub struct FecTracker {
    sets: HashMap<(u64, u32), FecSet>,
    completed: HashSet<(u64, u32)>,
    slots: HashMap<u64, SlotAccumulator>,
    completed_slots: HashSet<u64>,
    max_slot: u64,
    eviction_threshold: u64,
}

pub enum IngestResult {
    Pending,
    FirstFecSet {
        slot: u64,
        data: Vec<u8>,
    },
    SlotComplete(CompletedSlot),
}

impl FecTracker {
    pub fn new(eviction_threshold: u64) -> Self {
        Self {
            sets: HashMap::new(),
            completed: HashSet::new(),
            slots: HashMap::new(),
            completed_slots: HashSet::new(),
            max_slot: 0,
            eviction_threshold,
        }
    }

    pub fn ingest(&mut self, shred: &ParsedShred) -> IngestResult {
        let slot = shred.slot();
        let fec_idx = shred.fec_set_index();
        let key = (slot, fec_idx);

        if slot > self.max_slot {
            self.max_slot = slot;
            self.evict_stale();
        }

        if self.completed.contains(&key) || self.completed_slots.contains(&slot) {
            return IngestResult::Pending;
        }

        let set = self
            .sets
            .entry(key)
            .or_insert_with(|| FecSet::new(slot, fec_idx));

        if !set.insert(shred) {
            return IngestResult::Pending;
        }

        self.completed.insert(key);
        let mut set = match self.sets.remove(&key) {
            Some(s) => s,
            None => return IngestResult::Pending,
        };

        let last_in_slot = set.last_in_slot;

        let data = match set.reassemble() {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(slot, fec_idx, error = %e, "FEC reassembly failed");
                return IngestResult::Pending;
            }
        };

        let acc = self
            .slots
            .entry(slot)
            .or_insert_with(|| SlotAccumulator::new(slot));

        let is_first = acc.fec_set_count() == 0 && fec_idx == 0;

        acc.add_fec_set(fec_idx, data, last_in_slot);

        if acc.is_complete() {
            let acc = self.slots.remove(&slot).unwrap();
            self.completed_slots.insert(slot);
            let full_data = acc.concatenate();

            tracing::debug!(
                slot,
                fec_sets = acc.fec_set_count(),
                total_bytes = full_data.len(),
                "Slot complete"
            );

            IngestResult::SlotComplete(CompletedSlot {
                slot,
                data: full_data,
                fec_set_count: acc.fec_set_count(),
            })
        } else if is_first {
            // Return a clone of the first FEC set's data for greedy parsing.
            // Only the first FEC set (fec_idx=0) has the Vec<Entry> length header,
            // so only it is usable for greedy deserialization.
            let first_data = acc.fec_sets_ref().get(&0).unwrap().clone();
            tracing::debug!(
                slot,
                fec_idx,
                fec_sets_so_far = acc.fec_set_count(),
                "First FEC set complete, slot incomplete"
            );
            IngestResult::FirstFecSet {
                slot,
                data: first_data,
            }
        } else {
            tracing::debug!(
                slot,
                fec_idx,
                fec_sets_so_far = acc.fec_set_count(),
                "FEC set complete, slot incomplete"
            );
            IngestResult::Pending
        }
    }

    fn evict_stale(&mut self) {
        if self.max_slot < self.eviction_threshold {
            return;
        }
        let cutoff = self.max_slot - self.eviction_threshold;
        self.sets.retain(|&(slot, _), _| slot >= cutoff);
        self.completed.retain(|&(slot, _)| slot >= cutoff);
        self.slots.retain(|&slot, _| slot >= cutoff);
        self.completed_slots.retain(|&slot| slot >= cutoff);
    }

    pub fn active_sets(&self) -> usize {
        self.sets.len()
    }

    pub fn active_slots(&self) -> usize {
        self.slots.len()
    }
}
