use std::path::PathBuf;

/// Root of the per-project glyph store (mirrors `.git/`).
pub fn root() -> PathBuf {
    PathBuf::from(".glyph")
}

pub fn dialects_dir() -> PathBuf {
    root().join("dialects")
}

/// Directory holding everything for one dialect id.
pub fn dialect_dir(id: &str) -> PathBuf {
    dialects_dir().join(id)
}

/// Raw (public) seed for a dialect.
pub fn dialect_seed_file(id: &str) -> PathBuf {
    dialect_dir(id).join("seed")
}

/// Encrypted (secret) seed for a dialect.
pub fn dialect_seed_enc(id: &str) -> PathBuf {
    dialect_dir(id).join("seed.enc")
}

pub fn config_file() -> PathBuf {
    root().join("config.toml")
}
