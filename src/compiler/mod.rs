//! F1 compiler orchestration: ties the glyph codegen and Python front-end to
//! the seed/dialect store and the encryption layer.

pub mod glyph;
pub mod ir;
pub mod op;
pub mod py;

use anyhow::Context;

use crate::seed::crypto;
use crate::seed::dialect::{dialect_id, parse_header, HEADER_PREFIX};
use crate::seed::permute::build as permute_build;
use crate::store::init::{load_seed, load_seed_by_id};

/// Compile a plain `.glf` (header + glyph body) to Python using the dialect in
/// its header. Output is written to `out`.
pub fn build(input: &std::path::Path, out: &std::path::Path) -> anyhow::Result<()> {
    let data = std::fs::read_to_string(input)?;
    let (id, body) = split_header(&data)?;
    let (seed, _) = load_seed_by_id(&id).context("cannot load dialect seed for this .glf")?;
    let map = permute_build(&seed);
    let stmts = glyph::parse_glyph(body, &map)?;
    let py = py::emit_python(&stmts);
    std::fs::write(out, py)?;
    println!("✓ built {} -> {}", input.display(), out.display());
    Ok(())
}

/// Obfuscate a real `.py` into an encrypted `.glf`: parse Python -> IR -> glyph
/// text -> AES-GCM with the current seed. Output is the `.glf`.
pub fn obfuscate(input: &std::path::Path, out: &std::path::Path) -> anyhow::Result<()> {
    let src = std::fs::read_to_string(input)?;
    let stmts = py::parse_python(&src).context("failed to parse Python source")?;
    let (seed, _) = load_seed()?;
    let map = permute_build(&seed);
    let glyph_text = glyph::emit_glyph(&stmts, &map);
    let blob = crypto::encrypt_bytes(&seed, glyph_text.as_bytes())?;
    let id = dialect_id(&seed);
    let mut out_bytes = Vec::new();
    out_bytes.extend_from_slice(HEADER_PREFIX.as_bytes());
    out_bytes.extend_from_slice(id.as_bytes());
    out_bytes.push(b'\n');
    out_bytes.extend_from_slice(&blob);
    std::fs::write(out, &out_bytes)?;
    println!("✓ obfuscated {} -> {}", input.display(), out.display());
    println!("  glyphs: {}", stmts.len());
    Ok(())
}

/// Reveal an encrypted `.glf` back to Python: decrypt with the header's dialect
/// seed -> glyph text -> IR -> Python.
pub fn reveal(input: &std::path::Path, out: &std::path::Path) -> anyhow::Result<()> {
    let data = std::fs::read(input)?;
    let nl = data
        .iter()
        .position(|&b| b == b'\n')
        .context("missing glyph header")?;
    let header = std::str::from_utf8(&data[..nl]).context("header is not UTF-8")?;
    let id = parse_header(header).context("missing glyph header (expected '#!glyph1 <id>')")?;
    let blob = &data[nl + 1..];
    let (seed, _) = load_seed_by_id(&id).context("cannot load dialect seed for this .glf")?;
    let glyph_text = crypto::decrypt_bytes(&seed, blob).context("decrypt failed (wrong seed?)")?;
    let map = permute_build(&seed);
    let stmts = glyph::parse_glyph(std::str::from_utf8(&glyph_text)?, &map)?;
    let py = py::emit_python(&stmts);
    std::fs::write(out, py)?;
    println!("✓ revealed {} -> {}", input.display(), out.display());
    Ok(())
}

fn split_header(data: &str) -> anyhow::Result<(String, &str)> {
    let first = data.lines().next().unwrap_or("");
    if let Some(id) = parse_header(first) {
        let body = data.split_once('\n').map(|(_, b)| b).unwrap_or("");
        Ok((id, body))
    } else {
        anyhow::bail!("missing glyph header (expected '#!glyph1 <id>')")
    }
}
