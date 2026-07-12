//! Official BIP-39 wordlists from `bitcoin/bips`, vendored into the binary with
//! `include_str!`. Each file is exactly 2,048 words, one per line, with a stable
//! index `0..=2047` (11 bits per word). This gives us a deterministic
//! index→word mapping for recovery phrases.
//!
//! Glyph *reuses the BIP-39 word lists* (their quality and format), but the
//! recovery system is **Glyph Mnemonic v1**, not a BIP-39 wallet: seed,
//! checksum, KDF and file format are proprietary. A Glyph phrase is NOT valid
//! for any wallet and must never be.
//!
//! Only `en`, `es`, `fr`, `it` are registered for F0. `ja` / `zh-*` are vendored
//! but disabled until Unicode NFKD normalization and ideographic-space
//! separators are handled.

pub const ENGLISH: &str = include_str!("../../assets/wordlists/english.txt");
pub const SPANISH: &str = include_str!("../../assets/wordlists/spanish.txt");
pub const FRENCH: &str = include_str!("../../assets/wordlists/french.txt");
pub const ITALIAN: &str = include_str!("../../assets/wordlists/italian.txt");

// Vendored, disabled pending NFKD / ideographic-space handling:
// pub const JAPANESE: &str = include_str!("../../assets/wordlists/japanese.txt");
// pub const CHINESE_SIMPLIFIED: &str = include_str!("../../assets/wordlists/chinese_simplified.txt");
// pub const CHINESE_TRADITIONAL: &str = include_str!("../../assets/wordlists/chinese_traditional.txt");

pub struct Lang {
    pub code: &'static str,
    pub name: &'static str,
    pub raw: &'static str,
}

/// Languages available for recovery phrases in this build.
pub const LANGS: &[Lang] = &[
    Lang { code: "en", name: "English", raw: ENGLISH },
    Lang { code: "es", name: "Español", raw: SPANISH },
    Lang { code: "fr", name: "Français", raw: FRENCH },
    Lang { code: "it", name: "Italiano", raw: ITALIAN },
];

/// Split a raw wordlist into its words (trimmed, non-empty lines).
pub fn parse_wordlist(raw: &'static str) -> Vec<&'static str> {
    raw.lines().map(str::trim).filter(|w| !w.is_empty()).collect()
}

/// Return the words for a language code, if it is registered.
pub fn words(code: &str) -> Option<Vec<&'static str>> {
    LANGS.iter().find(|l| l.code == code).map(|l| parse_wordlist(l.raw))
}

/// Fold Latin diacritics + case so that e.g. "Árbol" matches "arbol". BIP-39
/// allows treating Spanish accents as equivalent on input; we extend this to
/// the other Latin languages for forgiving decode. Handles both precomposed
/// (`á`) and decomposed (`a` + combining acute) forms.
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            let l = c.to_ascii_lowercase();
            match l {
                'á' | 'à' | 'â' | 'ä' | 'ã' => vec!['a'],
                'é' | 'è' | 'ê' | 'ë' => vec!['e'],
                'í' | 'ì' | 'î' | 'ï' => vec!['i'],
                'ó' | 'ò' | 'ô' | 'ö' | 'õ' => vec!['o'],
                'ú' | 'ù' | 'û' | 'ü' => vec!['u'],
                'ñ' => vec!['n'],
                'ç' => vec!['c'],
                // combining diacritical marks (decomposed accents): drop them
                c if ('\u{0300}'..='\u{036f}').contains(&c) => vec![],
                _ => vec![l],
            }
        })
        .collect()
}

/// Detect which registered language a phrase belongs to, by its first word.
/// Accent/case-insensitive, so an Élève-style misspelling still resolves.
pub fn detect_lang(first_word: &str) -> Option<&'static str> {
    let fw = fold(first_word);
    for l in LANGS {
        let found = parse_wordlist(l.raw).iter().any(|w| fold(w) == fw);
        if found {
            return Some(l.code);
        }
    }
    None
}

/// Map a phrase word to its index within `lang`'s list (accent/case-insensitive).
pub fn word_index(lang: &str, token: &str) -> anyhow::Result<usize> {
    let words = words(lang).ok_or_else(|| anyhow::anyhow!("idioma no registrado: {lang}"))?;
    let t = fold(token);
    words
        .iter()
        .position(|w| fold(w) == t)
        .ok_or_else(|| anyhow::anyhow!("palabra no reconocida en '{lang}': '{token}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registered_list_has_2048_words() {
        for l in LANGS {
            assert_eq!(parse_wordlist(l.raw).len(), 2048, "lista {} no tiene 2048", l.code);
        }
    }

    #[test]
    fn known_indices_are_stable() {
        let en = words("en").unwrap();
        assert_eq!(en[0], "abandon");
        assert_eq!(en[2047], "zoo");
        // Spanish accent folding resolves despite input accents.
        assert_eq!(detect_lang("ábaco"), Some("es"));
    }
}
