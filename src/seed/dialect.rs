use blake3::Hash;

use crate::seed::prng::Seed;

/// Short, stable identifier for a dialect. Derived from the seed so the same
/// seed always produces the same id. Stored in every `.glf` file header so a
/// file is self-describing about which dialect encoded it.
pub fn dialect_id(seed: &Seed) -> String {
    let h: Hash = blake3::hash(seed);
    h.to_hex()[..12].to_string()
}

/// Magic prefix written at the top of every `.glf` file. Followed by the
/// dialect id and a newline, e.g. `#!glyph1 a1b2c3d4e5f6\n`.
pub const HEADER_PREFIX: &str = "#!glyph1 ";

/// Parse a dialect id out of a file's first line, if present.
pub fn parse_header(line: &str) -> Option<String> {
    line.strip_prefix(HEADER_PREFIX)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
