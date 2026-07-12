use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::seed::crypto;
use crate::seed::dialect::dialect_id;
use crate::seed::mnemonic;
use crate::seed::permute;
use crate::seed::prng::{from_entropy, from_str, Seed};

use crate::store::layout;

#[derive(Serialize, Deserialize)]
struct Config {
    dialect_id: String,
    secret: bool,
}

/// Create `.glyph/` for a new project. Generates (or imports) a seed, derives
/// its dialect id, and writes the per-dialect seed plus the registry entry.
pub fn init_project(secret: bool, seed_str: Option<&str>) -> anyhow::Result<()> {
    let root = layout::root();
    if root.exists() {
        anyhow::bail!(".glyph already initialized in this directory");
    }
    fs::create_dir_all(layout::dialects_dir())?;

    let seed: Seed = match seed_str {
        Some(s) => from_str(s),
        None => from_entropy(),
    };
    let id = dialect_id(&seed);

    write_seed(&id, &seed, secret)?;

    fs::write(
        layout::config_file(),
        toml::to_string(&Config {
            dialect_id: id.clone(),
            secret,
        })?,
    )?;

    println!("✓ glyph initialized");
    println!("  dialect id : {id}");
    println!("  seed mode  : {}", if secret { "secret (encrypted)" } else { "public" });
    println!("  glyphs     : {}", permute::build(&seed).op_to_glyph.len());
    Ok(())
}

/// Write a dialect's seed (raw or encrypted) under its registry directory.
fn write_seed(id: &str, seed: &Seed, secret: bool) -> anyhow::Result<()> {
    let dir = layout::dialect_dir(id);
    fs::create_dir_all(&dir)?;
    if secret {
        let pass = read_pass("seed passphrase: ")?;
        let enc = crypto::encrypt_seed(seed, &pass)?;
        fs::write(layout::dialect_seed_enc(id), enc)?;
        // remove any stale raw seed
        let _ = fs::remove_file(layout::dialect_seed_file(id));
    } else {
        fs::write(layout::dialect_seed_file(id), &seed[..])?;
        let _ = fs::remove_file(layout::dialect_seed_enc(id));
    }
    Ok(())
}

fn read_pass(prompt: &str) -> anyhow::Result<String> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

/// Prompt the user for a secret on stdin (used by CLI commands).
pub fn prompt_secret(prompt: &str) -> anyhow::Result<String> {
    read_pass(prompt)
}

/// Load the seed of the *current* dialect (from config). Returns the seed and
/// whether it was stored encrypted.
pub fn load_seed() -> anyhow::Result<(Seed, bool)> {
    let id = current_dialect_id()?;
    let enc = layout::dialect_seed_enc(&id);
    let raw = layout::dialect_seed_file(&id);
    if enc.exists() {
        let pass = read_pass("seed passphrase: ")?;
        let data = fs::read(&enc)?;
        let seed = crypto::decrypt_seed(&data, &pass)?;
        Ok((seed, true))
    } else if raw.exists() {
        let data = fs::read(&raw)?;
        let mut s = [0u8; 32];
        if data.len() < 32 {
            anyhow::bail!("corrupt seed file");
        }
        s.copy_from_slice(&data[..32]);
        Ok((s, false))
    } else {
        anyhow::bail!("no seed for current dialect '{id}'; run `glyph init`")
    }
}

pub fn current_dialect_id() -> anyhow::Result<String> {
    let cfg = layout::config_file();
    if !cfg.exists() {
        anyhow::bail!("not a glyph project (missing .glyph/config.toml)");
    }
    let text = fs::read_to_string(&cfg)?;
    let cfg: Config = toml::from_str(&text)?;
    Ok(cfg.dialect_id)
}

/// Load the seed for a *specific* dialect id (not just the current one). Used by
/// `glyph build` / `reveal` when a `.glf` file carries its own dialect id in the
/// header. Falls back to the current dialect if `id` is the current one.
pub fn load_seed_by_id(id: &str) -> anyhow::Result<(Seed, bool)> {
    let enc = layout::dialect_seed_enc(id);
    let raw = layout::dialect_seed_file(id);
    if enc.exists() {
        let pass = read_pass("seed passphrase: ")?;
        let data = fs::read(&enc)?;
        let seed = crypto::decrypt_seed(&data, &pass)?;
        Ok((seed, true))
    } else if raw.exists() {
        let data = fs::read(&raw)?;
        let mut s = [0u8; 32];
        if data.len() < 32 {
            anyhow::bail!("corrupt seed file");
        }
        s.copy_from_slice(&data[..32]);
        Ok((s, false))
    } else {
        anyhow::bail!("no seed for dialect '{id}'; run `glyph init` or `glyph seed rotate`")
    }
}

pub fn is_secret() -> anyhow::Result<bool> {
    let cfg = layout::config_file();
    let text = fs::read_to_string(&cfg)?;
    let cfg: Config = toml::from_str(&text)?;
    Ok(cfg.secret)
}

/// Replace the current dialect's seed with a new one (from entropy or a
/// provided string). Keeps the same dialect id.
pub fn regenerate(seed_str: Option<&str>) -> anyhow::Result<()> {
    let id = current_dialect_id()?;
    let secret = is_secret()?;
    let seed: Seed = match seed_str {
        Some(s) => from_str(s),
        None => from_entropy(),
    };
    write_seed(&id, &seed, secret)?;
    println!("✓ seed regenerated for dialect {id}");
    Ok(())
}

/// Create a brand-new dialect and make it current. The previous dialect's
/// seed and registry entry are preserved, so older `.glf` files (which carry
/// their own dialect id in the header) keep decoding correctly.
pub fn rotate() -> anyhow::Result<()> {
    let new_id = dialect_id(&from_entropy());
    let seed = from_entropy();
    write_seed(&new_id, &seed, is_secret()?)?;
    fs::write(
        layout::config_file(),
        toml::to_string(&Config {
            dialect_id: new_id.clone(),
            secret: is_secret()?,
        })?,
    )?;
    println!("✓ rotated to new dialect {new_id}");
    println!("  (previous dialect seeds are preserved under .glyph/dialects/)");
    Ok(())
}

/// Encrypt a currently-public seed (prompts for passphrase).
pub fn lock(pass: &str) -> anyhow::Result<()> {
    let id = current_dialect_id()?;
    let (seed, secret) = load_seed()?;
    if secret {
        anyhow::bail!("seed is already encrypted");
    }
    write_seed(&id, &seed, true)?;
    // persist secret flag
    fs::write(
        layout::config_file(),
        toml::to_string(&Config {
            dialect_id: id,
            secret: true,
        })?,
    )?;
    let _ = pass;
    println!("✓ seed locked (encrypted)");
    Ok(())
}

/// Decrypt a currently-secret seed back to raw.
pub fn unlock(pass: &str) -> anyhow::Result<()> {
    let id = current_dialect_id()?;
    let _ = pass;
    let (seed, secret) = load_seed()?;
    if !secret {
        anyhow::bail!("seed is not encrypted");
    }
    write_seed(&id, &seed, false)?;
    fs::write(
        layout::config_file(),
        toml::to_string(&Config {
            dialect_id: id,
            secret: false,
        })?,
    )?;
    println!("✓ seed unlocked (raw)");
    Ok(())
}

/// Export the current seed as base64 (encrypted blob if secret).
pub fn export_seed() -> anyhow::Result<Vec<u8>> {
    let id = current_dialect_id()?;
    let enc = layout::dialect_seed_enc(&id);
    let raw = layout::dialect_seed_file(&id);
    if enc.exists() {
        Ok(fs::read(&enc)?)
    } else if raw.exists() {
        Ok(fs::read(&raw)?)
    } else {
        anyhow::bail!("no seed to export");
    }
}

/// Import a seed blob (32 raw bytes, or 44+ encrypted bytes) as the current
/// dialect's seed.
pub fn import_seed(data: &[u8]) -> anyhow::Result<()> {
    let id = current_dialect_id()?;
    let secret = is_secret()?;
    if data.len() == 32 {
        let mut s = [0u8; 32];
        s.copy_from_slice(&data[..32]);
        write_seed(&id, &s, secret)?;
        println!("✓ imported raw seed for dialect {id}");
    } else {
        // treat as encrypted blob
        fs::write(layout::dialect_seed_enc(&id), data)?;
        let _ = fs::remove_file(layout::dialect_seed_file(&id));
        println!("✓ imported encrypted seed for dialect {id}");
    }
    Ok(())
}

/// Path for export/import files.
pub fn export_path() -> PathBuf {
    PathBuf::from("glyph-seed-export.bin")
}

/// Write an encrypted, passphrase-protected backup of the current seed to
/// `out`. This is the human "emergency backup": store it offline so a lost or
/// forgotten seed can always be recovered.
pub fn backup_seed(out: &Path, pass: &str) -> anyhow::Result<()> {
    let (seed, _) = load_seed()?;
    let enc = crypto::encrypt_seed(&seed, pass)?;
    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(out, enc)?;
    println!("✓ seed backup written to {}", out.display());
    println!("  keep this file + passphrase safe and offline");
    Ok(())
}

/// Restore the current dialect's seed from an encrypted backup file.
pub fn recover_seed(inp: &Path, pass: &str) -> anyhow::Result<()> {
    let data = fs::read(inp)?;
    let seed = crypto::decrypt_seed(&data, pass)?;
    let id = current_dialect_id()?;
    write_seed(&id, &seed, true)?;
    fs::write(
        layout::config_file(),
        toml::to_string(&Config {
            dialect_id: id.clone(),
            secret: true,
        })?,
    )?;
    println!("✓ recovered seed for dialect {id}");
    Ok(())
}

/// Write a seed as the current dialect's seed, marked secret. Shared by all
/// recovery paths (phrase, backup file, Shamir combine).
fn apply_seed(seed: &Seed) -> anyhow::Result<()> {
    let id = current_dialect_id()?;
    write_seed(&id, seed, true)?;
    fs::write(
        layout::config_file(),
        toml::to_string(&Config {
            dialect_id: id.clone(),
            secret: true,
        })?,
    )?;
    println!("✓ seed aplicado para dialect {id}");
    Ok(())
}

/// Restore the current dialect's seed from a Glyph Mnemonic v1 phrase (any
/// registered language; auto-detected, checksum-verified).
pub fn recover_seed_phrase(phrase: &str) -> anyhow::Result<()> {
    let seed = mnemonic::mnemonic_to_seed(phrase)?;
    apply_seed(&seed)
}

/// Restore the current dialect's seed from raw bytes (used by Shamir combine).
pub fn set_current_seed(seed: &Seed) -> anyhow::Result<()> {
    apply_seed(seed)
}

/// Print the current seed as a recovery phrase in `lang`.
pub fn show_phrase_lang(lang: &str) -> anyhow::Result<()> {
    let (seed, _) = load_seed()?;
    let phrase = mnemonic::seed_to_mnemonic(&seed, lang)?;
    println!("recovery phrase ({lang}):");
    println!("  {phrase}");
    Ok(())
}

/// Print the current seed as an equivalent phrase in each requested language.
pub fn show_phrase_langs(langs: &[String]) -> anyhow::Result<()> {
    let (seed, _) = load_seed()?;
    for lang in langs {
        let phrase = mnemonic::seed_to_mnemonic(&seed, lang)?;
        println!("{lang}:");
        println!("  {phrase}");
    }
    Ok(())
}

/// Print the current seed as an English recovery phrase.
pub fn show_phrase() -> anyhow::Result<()> {
    show_phrase_lang("en")
}
