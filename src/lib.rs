//! Glyph — agent-facing glyph CLI.
//!
//! F0 scope: deterministic seed engine, bijective glyph-permutation,
//! on-disk store (`.glyph/`), and the `init` / `seed *` commands.

pub mod compiler;
pub mod seed;
pub mod store;
