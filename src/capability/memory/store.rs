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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryLookupReceipt {
    pub query_hash: u64,
    pub requested_limit: u16,
    pub returned_count: u16,
    pub index_fingerprint: u64,
    pub aggregate_hash: u64,
    pub receipt_hash: u64,
}

impl MemoryLookupReceipt {
    pub fn from_lookup(
        query_hash: u64,
        requested_limit: usize,
        index_fingerprint: u64,
        lookup: &MemoryLookupRecord,
    ) -> Self {
        let requested_limit = requested_limit.min(u16::MAX as usize) as u16;
        let returned_count = lookup.matches.len().min(u16::MAX as usize) as u16;
        let mut receipt = Self {
            query_hash,
            requested_limit,
            returned_count,
            index_fingerprint,
            aggregate_hash: lookup.aggregate_hash,
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt
    }

    pub fn is_valid_for(&self, lookup: &MemoryLookupRecord) -> bool {
        self.query_hash != 0
            && self.index_fingerprint != 0
            && lookup.is_valid()
            && self.query_hash == lookup.query_hash
            && self.returned_count == lookup.matches.len().min(u16::MAX as usize) as u16
            && self.aggregate_hash == lookup.aggregate_hash
            && self.receipt_hash == self.expected_receipt_hash()
    }

    pub fn expected_receipt_hash(&self) -> u64 {
        let mut h = 0x4d45_4d4f_5259_5255u64;
        h = mix(h, self.query_hash);
        h = mix(h, u64::from(self.requested_limit));
        h = mix(h, u64::from(self.returned_count));
        h = mix(h, self.index_fingerprint);
        h = mix(h, self.aggregate_hash);
        h.max(1)
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

    pub fn lookup_with_receipt(
        &self,
        query_hash: u64,
        limit: usize,
    ) -> (MemoryLookupRecord, MemoryLookupReceipt) {
        let lookup = self.lookup(query_hash, limit);
        let receipt =
            MemoryLookupReceipt::from_lookup(query_hash, limit, self.fingerprint(), &lookup);
        (lookup, receipt)
    }

    pub fn fingerprint(&self) -> u64 {
        aggregate_index_hash(&self.facts)
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
    fold_memory_facts_hash(0x510e527fade682d1u64 ^ query_hash, matches)
}

fn aggregate_index_hash(facts: &[MemoryFact]) -> u64 {
    fold_memory_facts_hash(0x4d45_4d49_4e44_4558u64, facts)
}

fn fold_memory_facts_hash(mut h: u64, facts: &[MemoryFact]) -> u64 {
    for fact in facts {
        h = mix(h, fact.key);
        h = mix(h, fact.value_hash);
        h = mix(h, fact.weight as u64);
        h = mix(h, fact.source_seq);
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_lookup_receipt_binds_limit_index_and_result() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(7, 70, 2, 2)));
        assert!(index.insert(MemoryFact::new(7, 90, 5, 1)));
        assert!(index.insert(MemoryFact::new(8, 80, 9, 3)));

        let (lookup, receipt) = index.lookup_with_receipt(7, 1);

        assert_eq!(lookup.matches, vec![MemoryFact::new(7, 90, 5, 1)]);
        assert_eq!(receipt.requested_limit, 1);
        assert_eq!(receipt.returned_count, 1);
        assert_eq!(receipt.index_fingerprint, index.fingerprint());
        assert!(receipt.is_valid_for(&lookup));
    }

    #[test]
    fn memory_lookup_receipt_rejects_tampered_result() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(11, 110, 4, 1)));
        assert!(index.insert(MemoryFact::new(11, 111, 3, 2)));

        let (mut lookup, receipt) = index.lookup_with_receipt(11, 2);
        lookup.matches.pop();

        assert!(!receipt.is_valid_for(&lookup));
    }

    #[test]
    fn memory_fingerprint_changes_when_index_changes() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(5, 50, 1, 1)));
        let before = index.fingerprint();
        assert!(index.insert(MemoryFact::new(5, 51, 1, 2)));

        assert_ne!(before, index.fingerprint());
    }

    #[test]
    fn memory_hash_fold_helper_preserves_lookup_and_index_boundaries() {
        let facts = vec![MemoryFact::new(7, 70, 2, 2), MemoryFact::new(7, 90, 5, 1)];
        let lookup_aggregate = aggregate_memory_hash(7, &facts);
        let index_fingerprint = aggregate_index_hash(&facts);

        assert_ne!(lookup_aggregate, 0);
        assert_ne!(index_fingerprint, 0);
        assert_ne!(lookup_aggregate, index_fingerprint);

        assert_ne!(aggregate_memory_hash(8, &facts), lookup_aggregate);
        assert_eq!(aggregate_index_hash(&facts), index_fingerprint);

        for changed in [
            vec![MemoryFact::new(8, 70, 2, 2), facts[1]],
            vec![MemoryFact::new(7, 71, 2, 2), facts[1]],
            vec![MemoryFact::new(7, 70, 3, 2), facts[1]],
            vec![MemoryFact::new(7, 70, 2, 3), facts[1]],
        ] {
            assert_ne!(aggregate_memory_hash(7, &changed), lookup_aggregate);
            assert_ne!(aggregate_index_hash(&changed), index_fingerprint);
        }

        let reversed = vec![facts[1], facts[0]];
        assert_ne!(aggregate_memory_hash(7, &reversed), lookup_aggregate);
        assert_ne!(aggregate_index_hash(&reversed), index_fingerprint);

        let mut inserted_forward = MemoryIndex::default();
        assert!(inserted_forward.insert(facts[0]));
        assert!(inserted_forward.insert(facts[1]));

        let mut inserted_reverse = MemoryIndex::default();
        assert!(inserted_reverse.insert(facts[1]));
        assert!(inserted_reverse.insert(facts[0]));

        assert_eq!(inserted_forward.facts(), inserted_reverse.facts());
        assert_eq!(inserted_forward.fingerprint(), inserted_reverse.fingerprint());

        let (lookup, receipt) = inserted_forward.lookup_with_receipt(7, 2);
        assert_eq!(lookup.aggregate_hash, aggregate_memory_hash(7, &lookup.matches));
        assert_eq!(receipt.aggregate_hash, lookup.aggregate_hash);
        assert_eq!(receipt.index_fingerprint, inserted_forward.fingerprint());
        assert_eq!(receipt.index_fingerprint, aggregate_index_hash(inserted_forward.facts()));
        assert!(receipt.is_valid_for(&lookup));
    }
}
