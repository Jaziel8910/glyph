use rand::{rngs::StdRng, SeedableRng};
use sha2::{Digest, Sha256};

/// A 256-bit project seed. The entire glyph dialect is derived from this.
pub type Seed = [u8; 32];

/// Generate a fresh seed from OS entropy.
pub fn from_entropy() -> Seed {
    rand::random()
}

/// Deterministically derive a seed from an arbitrary string (e.g. a shared
/// passphrase or a published seed value).
pub fn from_str(s: &str) -> Seed {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&out);
    seed
}

/// Build a deterministic PRNG from a seed. Same seed -> same stream.
pub fn rng(seed: &Seed) -> StdRng {
    StdRng::from_seed(*seed)
}
