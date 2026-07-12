#![no_main]
use libfuzzer_sys::fuzz_target;

use glyph::seed::CANONICAL_OPS;
use glyph::seed::permute::build;
use glyph::seed::prng::Seed;

fuzz_target!(|data: &[u8]| {
    // Derive a 32-byte seed from the (possibly short) fuzz input.
    let mut seed = [0u8; 32];
    if data.is_empty() {
        return;
    }
    for (i, b) in data.iter().cycle().take(32).enumerate() {
        seed[i] = *b;
    }
    let d = build(&seed);
    // Core invariant the lexer/parser will rely on: bijective + round-trip.
    assert_eq!(d.op_to_glyph.len(), CANONICAL_OPS.len());
    assert_eq!(d.glyph_to_op.len(), d.op_to_glyph.len());
    for (op, g) in &d.op_to_glyph {
        assert_eq!(d.glyph_to_op.get(g).unwrap(), op);
    }
});
