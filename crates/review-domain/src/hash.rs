//! Deterministic hashing utilities shared across review domain types.

use blake3::Hasher;

/// Deterministic 64-bit hash for identifiers backed by truncated BLAKE3 output.
///
/// Using a cryptographic hash reduces the risk of accidental collisions when compared
/// to simple FNV-based hashes while keeping identifier generation deterministic.
#[must_use = "the returned hash should be used as a stable identifier"]
pub fn hash64(parts: &[&[u8]]) -> u64 {
    let mut hasher = Hasher::new();
    for part in parts {
        hasher.update(part);
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
    u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash64_changes_when_input_changes() {
        let base = hash64(&[b"abc"]);
        assert_ne!(base, hash64(&[b"abcd"]));
        assert_ne!(base, hash64(&[b"ab", b"c"]));
    }
}
