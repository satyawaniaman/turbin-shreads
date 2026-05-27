use std::collections::BTreeMap;

#[derive(Debug)]
pub struct SlotAccumulator {
    pub slot: u64,
    fec_sets: BTreeMap<u32, Vec<u8>>,
    last_fec_set_index: Option<u32>,
}

impl SlotAccumulator {
    pub fn new(slot: u64) -> Self {
        Self {
            slot,
            fec_sets: BTreeMap::new(),
            last_fec_set_index: None,
        }
    }

    pub fn add_fec_set(&mut self, fec_set_index: u32, data: Vec<u8>, last_in_slot: bool) {
        self.fec_sets.insert(fec_set_index, data);
        if last_in_slot {
            self.last_fec_set_index = Some(fec_set_index);
        }
    }

    pub fn is_complete(&self) -> bool {
        let Some(last_idx) = self.last_fec_set_index else {
            return false;
        };
        self.fec_sets.contains_key(&last_idx)
    }

    pub fn concatenate(&self) -> Vec<u8> {
        let total: usize = self.fec_sets.values().map(|v| v.len()).sum();
        let mut result = Vec::with_capacity(total);
        for data in self.fec_sets.values() {
            result.extend_from_slice(data);
        }
        result
    }

    pub fn fec_set_count(&self) -> usize {
        self.fec_sets.len()
    }

    pub fn fec_sets_ref(&self) -> &BTreeMap<u32, Vec<u8>> {
        &self.fec_sets
    }
}
