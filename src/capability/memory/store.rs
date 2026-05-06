//! Deterministic in-memory index for prior run facts.

use crate::kernel::mix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryFact {
    pub key: u64,
    pub value_hash: u64,
    pub weight: u8,
    pub source_seq: u64,
}

impl MemoryFact {
    pub const fn new(key: u64, value_hash: u64, weight: u8, source_seq: u64) -> Self {
        Self {
            key,
            value_hash,
            weight,
            source_seq,
        }
    }

    pub fn is_valid(self) -> bool {
        self.key != 0 && self.value_hash != 0 && self.weight != 0 && self.source_seq != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryLookupRecord {
    pub query_hash: u64,
    pub matches: Vec<MemoryFact>,
    pub aggregate_hash: u64,
}

impl MemoryLookupRecord {
    pub fn is_valid(&self) -> bool {
        self.query_hash != 0
            && self.aggregate_hash == aggregate_memory_hash(self.query_hash, &self.matches)
            && self.matches.iter().copied().all(MemoryFact::is_valid)
    }

    pub fn match_count(&self) -> u8 {
        self.matches.len().min(u8::MAX as usize) as u8
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryIndex {
    facts: Vec<MemoryFact>,
}

impl MemoryIndex {
    pub fn insert(&mut self, fact: MemoryFact) -> bool {
        if !fact.is_valid() {
            return false;
        }

        if let Some(existing) = self
            .facts
            .iter_mut()
            .find(|existing| existing.key == fact.key && existing.source_seq == fact.source_seq)
        {
            *existing = fact;
        } else {
            self.facts.push(fact);
        }

        self.sort_facts();
        true
    }

    pub fn lookup(&self, query_hash: u64, limit: usize) -> MemoryLookupRecord {
        let mut matches = self
            .facts
            .iter()
            .copied()
            .filter(|fact| fact.key == query_hash)
            .collect::<Vec<_>>();

        matches.sort_by(|a, b| {
            b.weight
                .cmp(&a.weight)
                .then_with(|| a.source_seq.cmp(&b.source_seq))
                .then_with(|| a.value_hash.cmp(&b.value_hash))
        });
        matches.truncate(limit);

        MemoryLookupRecord {
            query_hash,
            aggregate_hash: aggregate_memory_hash(query_hash, &matches),
            matches,
        }
    }

    pub fn facts(&self) -> &[MemoryFact] {
        &self.facts
    }

    fn sort_facts(&mut self) {
        self.facts.sort_by(|a, b| {
            a.key
                .cmp(&b.key)
                .then_with(|| b.weight.cmp(&a.weight))
                .then_with(|| a.source_seq.cmp(&b.source_seq))
                .then_with(|| a.value_hash.cmp(&b.value_hash))
        });
    }
}

fn aggregate_memory_hash(query_hash: u64, matches: &[MemoryFact]) -> u64 {
    let mut h = 0x510e527fade682d1u64 ^ query_hash;
    for fact in matches {
        h = mix(h, fact.key);
        h = mix(h, fact.value_hash);
        h = mix(h, fact.weight as u64);
        h = mix(h, fact.source_seq);
    }
    h.max(1)
}
