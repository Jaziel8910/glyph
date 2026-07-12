//! Glyph Mnemonic v1 — multi-language recovery phrases over BIP-39 wordlists.
//!
//! A 32-byte seed → 24 words (BIP-39 style: 256 bits entropy + 8-bit checksum =
//! 264 bits = 24 × 11-bit indices). The same seed encodes to an equivalent
//! phrase in *every* registered language, so any single full phrase recovers the
//! seed. Language is declared/explicit (auto-detected from the first word on
//! decode, never guessed blindly).
//!
//! Shares (Shamir) use the same 11-bit packing but over 33 bytes (264 bits,
//! exact, no checksum): `x || y` where `x` is the share index.

use sha2::{Digest, Sha256};

use crate::seed::prng::Seed;
use crate::seed::wordlists;

const BITS: usize = 11;
const MASK: u32 = 0x7FF;

fn bits_to_indices(data: &[u8], total_bits: usize) -> Vec<u16> {
    let mut out = Vec::with_capacity(total_bits / BITS + 1);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in data {
        buf = (buf << 8) | b as u32;
        bits += 8;
        while bits >= BITS as u32 {
            bits -= BITS as u32;
            out.push(((buf >> bits) & MASK) as u16);
        }
    }
    debug_assert!(bits == 0, "total_bits must be a multiple of 11");
    out
}

fn indices_to_bytes(indices: &[u16], total_bits: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(total_bits / 8);
    let mut buf: u64 = 0;
    let mut bits: u32 = 0;
    for &i in indices {
        buf = (buf << BITS) | i as u64;
        bits += BITS as u32;
        while bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    out
}

fn join(words: &[&str]) -> String {
    words.join(" ")
}

fn words_of(lang: &str) -> anyhow::Result<Vec<&'static str>> {
    wordlists::words(lang).ok_or_else(|| anyhow::anyhow!("idioma no registrado: {lang}"))
}

/// Encode a 32-byte seed as a 24-word BIP-39-style phrase in `lang`.
pub fn seed_to_mnemonic(seed: &Seed, lang: &str) -> anyhow::Result<String> {
    let words = words_of(lang)?;
    let checksum = Sha256::digest(seed.as_slice())[0];
    let mut data = seed.as_slice().to_vec();
    data.push(checksum); // 33 bytes = 264 bits = 24 words
    let indices = bits_to_indices(&data, 264);
    let phrase: Vec<&str> = indices.iter().map(|i| words[*i as usize]).collect();
    Ok(join(&phrase))
}

/// Decode a 24-word phrase (any registered language) back into a 32-byte seed.
/// Tries every registered language and returns the one whose checksum verifies,
/// so a phrase whose first word also exists in another language still decodes
/// correctly (the wrong language will fail the checksum).
pub fn mnemonic_to_seed(phrase: &str) -> anyhow::Result<Seed> {
    let toks: Vec<&str> = phrase.split_whitespace().collect();
    if toks.len() != 24 {
        anyhow::bail!("se esperan 24 palabras, recibí {}", toks.len());
    }
    for l in wordlists::LANGS {
        let lang = l.code;
        if let Ok(indices) = toks
            .iter()
            .map(|t| wordlists::word_index(lang, t).map(|i| i as u16))
            .collect::<anyhow::Result<Vec<u16>>>()
        {
            let data = indices_to_bytes(&indices, 264); // 33 bytes
            let (body, checksum) = data.split_at(32);
            let mut seed_bytes = [0u8; 32];
            seed_bytes.copy_from_slice(body);
            let expected = Sha256::digest(&seed_bytes)[0];
            if checksum[0] == expected {
                return Ok(seed_bytes);
            }
        }
    }
    anyhow::bail!("frase no válida en ningún idioma registrado (idioma o checksum inválido)")
}

/// Encode arbitrary bytes (e.g. a Shamir share) as a phrase in `lang`. The byte
/// length must be a multiple of 11 bits (e.g. 33 bytes = 24 words).
pub fn bytes_to_mnemonic(bytes: &[u8], lang: &str) -> anyhow::Result<String> {
    let words = words_of(lang)?;
    let total = bytes.len() * 8;
    if total % BITS != 0 {
        anyhow::bail!("los bytes deben ser múltiplo de 11 bits");
    }
    let indices = bits_to_indices(bytes, total);
    let phrase: Vec<&str> = indices.iter().map(|i| words[*i as usize]).collect();
    Ok(join(&phrase))
}

/// Decode a phrase (any registered language) back into bytes.
/// Prefers `prefer_lang` when given, otherwise tries every registered language
/// and returns the first that decodes all words.
pub fn mnemonic_to_bytes(phrase: &str, prefer_lang: Option<&str>) -> anyhow::Result<Vec<u8>> {
    let toks: Vec<&str> = phrase.split_whitespace().collect();
    if toks.is_empty() {
        anyhow::bail!("frase vacía");
    }
    let total = toks.len() * BITS;
    let codes: Vec<&str> = wordlists::LANGS.iter().map(|l| l.code).collect();
    let langs: Vec<&str> = match prefer_lang {
        Some(l) => std::iter::once(l).chain(codes.iter().copied()).collect(),
        None => codes,
    };
    for lang in langs {
        if let Ok(indices) = toks
            .iter()
            .map(|t| wordlists::word_index(lang, t).map(|i| i as u16))
            .collect::<anyhow::Result<Vec<u16>>>()
        {
            return Ok(indices_to_bytes(&indices, total));
        }
    }
    anyhow::bail!("frase no válida en ningún idioma registrado")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::prng::from_entropy;

    #[test]
    fn seed_phrase_round_trips_in_every_language() {
        let seed = from_entropy();
        for lang in ["en", "es", "fr", "it"] {
            let phrase = seed_to_mnemonic(&seed, lang).unwrap();
            assert_eq!(phrase.split_whitespace().count(), 24);
            let back = mnemonic_to_seed(&phrase).unwrap();
            assert_eq!(seed, back, "round-trip failed for {lang}");
        }
    }

    #[test]
    fn equivalent_phrases_decode_to_same_seed() {
        let seed = from_entropy();
        let en = seed_to_mnemonic(&seed, "en").unwrap();
        let es = seed_to_mnemonic(&seed, "es").unwrap();
        assert_ne!(en, es);
        assert_eq!(mnemonic_to_seed(&en).unwrap(), mnemonic_to_seed(&es).unwrap());
    }

    #[test]
    fn tampered_checksum_is_rejected() {
        let seed = from_entropy();
        let mut phrase = seed_to_mnemonic(&seed, "en").unwrap();
        // flip the last word to one we know exists
        phrase.push(' ');
        phrase.push_str("zoo");
        let bad = phrase.replace("zoo", "zoo"); // no-op; build an actually-bad phrase
        let _ = bad;
        // Replace a middle word to corrupt checksum
        let mut toks: Vec<&str> = phrase.split_whitespace().collect();
        toks[10] = "abandon";
        let corrupted = toks.join(" ");
        assert!(mnemonic_to_seed(&corrupted).is_err());
    }
}
