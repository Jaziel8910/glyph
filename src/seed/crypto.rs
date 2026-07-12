use aes_gcm::aead::{Aead, AeadCore, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;

use crate::seed::prng::Seed;

const NONCE_LEN: usize = 12;

/// Derive a 256-bit AES key from a passphrase via HKDF.
fn key_from_pass(pass: &str) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, pass.as_bytes());
    let mut o = [0u8; 32];
    hk.expand(b"glyph-seed-v1", &mut o)
        .expect("hkdf expand to 32 bytes");
    o
}

/// Derive a 256-bit AES key directly from a seed (used for code obfuscation,
/// where the seed *is* the key — no passphrase involved).
fn key_from_seed(seed: &Seed) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, seed);
    let mut o = [0u8; 32];
    hk.expand(b"glyph-obfuscate-v1", &mut o)
        .expect("hkdf expand to 32 bytes");
    o
}

/// Encrypt a seed with a passphrase. Output: `<12-byte nonce>||ciphertext`.
pub fn encrypt_seed(seed: &Seed, pass: &str) -> anyhow::Result<Vec<u8>> {
    let key = key_from_pass(pass);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ct = cipher
        .encrypt(&nonce, seed.as_slice())
        .map_err(|e| anyhow::anyhow!("encrypt failed: {e}"))?;
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Decrypt a seed produced by [`encrypt_seed`].
pub fn decrypt_seed(data: &[u8], pass: &str) -> anyhow::Result<Seed> {
    if data.len() < NONCE_LEN {
        anyhow::bail!("ciphertext too short");
    }
    let (nonce, ct) = data.split_at(NONCE_LEN);
    let key = key_from_pass(pass);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let n = Nonce::from_slice(nonce);
    let pt = cipher
        .decrypt(n, ct)
        .map_err(|e| anyhow::anyhow!("decrypt failed (wrong passphrase?): {e}"))?;
    let mut s = [0u8; 32];
    if pt.len() != 32 {
        anyhow::bail!("decrypted seed has wrong length");
    }
    s.copy_from_slice(&pt);
    Ok(s)
}

/// Obfuscate arbitrary bytes using the seed as the key (human-facing code
/// obfuscation). Output layout: `<12-byte nonce>||ciphertext`.
pub fn encrypt_bytes(seed: &Seed, pt: &[u8]) -> anyhow::Result<Vec<u8>> {
    let key = key_from_seed(seed);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ct = cipher
        .encrypt(&nonce, pt)
        .map_err(|e| anyhow::anyhow!("obfuscate failed: {e}"))?;
    let mut out = nonce.to_vec();
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Reveal bytes produced by [`encrypt_bytes`].
pub fn decrypt_bytes(seed: &Seed, data: &[u8]) -> anyhow::Result<Vec<u8>> {
    if data.len() < NONCE_LEN {
        anyhow::bail!("ciphertext too short");
    }
    let (nonce, ct) = data.split_at(NONCE_LEN);
    let key = key_from_seed(seed);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let n = Nonce::from_slice(nonce);
    let pt = cipher
        .decrypt(n, ct)
        .map_err(|e| anyhow::anyhow!("reveal failed (wrong seed?): {e}"))?;
    Ok(pt)
}
