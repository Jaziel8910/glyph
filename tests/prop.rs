use glyph::seed::CANONICAL_OPS;
use glyph::seed::permute::{alphabet, build};
use glyph::seed::prng::{from_str, Seed};
use rand::Rng;

#[test]
fn alphabet_covers_ops() {
    assert!(
        alphabet().len() >= CANONICAL_OPS.len(),
        "alphabet {} must cover {} ops",
        alphabet().len(),
        CANONICAL_OPS.len()
    );
}

#[test]
fn permutation_is_bijective() {
    let mut rng = rand::thread_rng();
    for _ in 0..300 {
        let seed: Seed = rng.gen();
        let d = build(&seed);
        assert_eq!(d.op_to_glyph.len(), CANONICAL_OPS.len());
        assert_eq!(d.glyph_to_op.len(), d.op_to_glyph.len());
        for (op, g) in &d.op_to_glyph {
            assert_eq!(d.glyph_to_op.get(g).unwrap(), op, "round-trip broken");
        }
    }
}

#[test]
fn glyphs_are_distinct_chars() {
    let d = build(&from_str("distinct-check"));
    let mut seen = std::collections::HashSet::new();
    for g in d.op_to_glyph.values() {
        assert!(seen.insert(*g), "duplicate glyph char {g}");
    }
}

#[test]
fn determinism() {
    let s = from_str("fixed-seed-value");
    assert_eq!(build(&s).op_to_glyph, build(&s).op_to_glyph);
}
