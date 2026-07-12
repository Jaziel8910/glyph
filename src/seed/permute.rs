use std::collections::HashMap;

use rand::seq::SliceRandom;

use crate::seed::prng::{rng, Seed};
use crate::seed::CANONICAL_OPS;

/// A bijective mapping between canonical ops and glyph characters for one
/// dialect. Forward (`op_to_glyph`) and inverse (`glyph_to_op`) are kept in
/// sync so decoding is unambiguous.
#[derive(Debug, Clone)]
pub struct DialectMap {
    pub op_to_glyph: HashMap<String, char>,
    pub glyph_to_op: HashMap<char, String>,
}

/// Curated source of rare Unicode glyphs. Duplicates are removed at runtime
/// and the list is shuffled per-seed, so this only needs to be "long enough"
/// and visually distinct from normal prose.
const GLYPH_SOURCE: &str = "\
◆◇◈◉○●◌◍◎⊙⊚⊛⊜⊝⊞⊟⊠⊡⊢⊣⊤⊥⊦⊧⊨⊩⊪⊫⊬⊭⊮⊯⊰⊱⊲⊳⊴⊵⊶⊷⊸⊹⊺⊻⊼⊽⊾⊿\
⋀⋁⋂⋃⋄∗∙∅∇∆∓±∔∸≀⊻⊼\
↦↣↤↢↧↯↰↱↲↳↴↵↶↷↸↹↺↻↼↽↾↿⇀⇁⇂⇃⇄⇅⇆⇇⇈⇉⇊⇋⇌⇍⇎⇏⇐⇑⇒⇓⇔⇕⇖⇗⇘⇙⇚⇛⇜⇝⇞⇟\
⌘⌙⌚⌛⌜⌝⌞⌟⌠⌡⌢⌣⌤⌥⌦⌧⌨⌫⌬⌭⌮⌯⌰⌱⌲⌳⌴⌵⌶⌷⌸⌹⌺⌻⌼⌽⌾⌿⏀⏁⏂⏃⏄⏅⏆⏇⏈⏉⏊⏋⏌⏍⏎⏏\
✦✧✩✪✫✬✭✮✯✰✱✲✳✴✵✶✷✸✹✺✻✼✽✾✿❀❁❂❃❄❅❆❇❈❉❊❋\
▢▣▤▥▦▧▨▩▪▫▬▭▮▯▰▱▲▼▴▵▶▷▸▹►▻◀◁◂◃◄◅■□▬";

/// The working alphabet: de-duplicated glyph source (all duplicates removed,
/// not just consecutive ones).
pub fn alphabet() -> Vec<char> {
    let mut seen = std::collections::HashSet::new();
    let mut v: Vec<char> = GLYPH_SOURCE
        .chars()
        .filter(|c| !c.is_whitespace())
        .filter(|c| seen.insert(*c))
        .collect();
    v
}

/// Build the dialect map for a seed. Uses Fisher–Yates over the alphabet so
/// that the same seed always yields the same glyph assignment.
pub fn build(seed: &Seed) -> DialectMap {
    let ops = CANONICAL_OPS;
    let alpha = alphabet();
    assert!(
        alpha.len() >= ops.len(),
        "glyph alphabet too small: {} glyphs < {} ops",
        alpha.len(),
        ops.len()
    );

    let mut idx: Vec<usize> = (0..alpha.len()).collect();
    let mut r = rng(seed);
    idx.shuffle(&mut r);

    let mut op_to_glyph = HashMap::new();
    let mut glyph_to_op = HashMap::new();
    for (i, op) in ops.iter().enumerate() {
        let g = alpha[idx[i]];
        op_to_glyph.insert((*op).to_string(), g);
        glyph_to_op.insert(g, (*op).to_string());
    }

    DialectMap {
        op_to_glyph,
        glyph_to_op,
    }
}
