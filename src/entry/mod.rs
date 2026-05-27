use bincode::Options;
use solana_sdk::transaction::VersionedTransaction;

#[derive(Debug, serde::Deserialize)]
pub struct Entry {
    pub num_hashes: u64,
    pub hash: [u8; 32],
    pub transactions: Vec<VersionedTransaction>,
}

pub fn deserialize_entries(data: &[u8]) -> Vec<Entry> {
    if data.len() < 8 {
        return Vec::new();
    }

    let opts = bincode::options()
        .with_fixint_encoding()
        .allow_trailing_bytes();

    if let Ok(entries) = opts.deserialize::<Vec<Entry>>(data) {
        return entries;
    }

    let mut cursor = std::io::Cursor::new(data);
    let count: u64 = match opts.deserialize_from(&mut cursor) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    if count > 100_000 {
        return Vec::new();
    }

    let mut entries = Vec::new();
    for _ in 0..count {
        match opts.deserialize_from::<_, Entry>(&mut cursor) {
            Ok(entry) => entries.push(entry),
            Err(_) => break,
        }
    }

    if !entries.is_empty() {
        tracing::debug!(
            parsed = entries.len(),
            expected = count,
            tx_total = entries.iter().map(|e| e.transactions.len()).sum::<usize>(),
            "Greedy entry deserialization"
        );
    }

    entries
}
