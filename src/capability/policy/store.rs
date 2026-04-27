//! Append-only policy store placeholder.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyEntry {
    pub version: u64,
    pub key: &'static str,
    pub value: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyStore {
    entries: Vec<PolicyEntry>,
}

impl PolicyStore {
    pub fn append(&mut self, entry: PolicyEntry) {
        self.entries.push(entry);
    }

    pub fn latest(&self, key: &str) -> Option<&PolicyEntry> {
        self.entries.iter().rev().find(|entry| entry.key == key)
    }
}

// TODO: replace in-memory storage with durable append-only policy artifacts.
