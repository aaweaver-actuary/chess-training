use blake3::Hasher;

const SCHEMA_VERSION: u8 = 1;
const HASH_NAMESPACE: u64 = 0x0123_4567_89ab_cdef;

/// Computes a deterministic hash for the given input string, namespaced and versioned.
///
/// # Panics
/// Panics if the hash output cannot be converted to a u64 (should never happen with blake3).
#[must_use]
pub fn hash_with_seed(input: &str) -> u64 {
    let mut seed_bytes = HASH_NAMESPACE.to_le_bytes().to_vec();
    seed_bytes.push(SCHEMA_VERSION);
    let mut hasher = Hasher::new();
    hasher.update(&seed_bytes);
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap())
}
