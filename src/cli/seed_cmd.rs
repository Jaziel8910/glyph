use clap::{Args, Subcommand};

use glyph::seed::dialect::dialect_id;
use glyph::seed::mnemonic;
use glyph::seed::permute::build;
use glyph::seed::shamir;
use glyph::store::init::{
    backup_seed, current_dialect_id, export_seed, import_seed, load_seed, lock as lock_seed,
    prompt_secret, recover_seed, recover_seed_phrase, regenerate, rotate,     set_current_seed,
    show_phrase_lang, show_phrase_langs, unlock,
};

#[derive(Args)]
pub struct SeedArgs {
    #[command(subcommand)]
    pub cmd: SeedSub,
}

#[derive(Subcommand)]
pub enum SeedSub {
    /// Show the dialect id and the op→glyph map (pass --public for id only).
    Show(ShowArgs),
    /// Regenerate the current dialect's seed.
    New(NewArgs),
    /// Export the seed blob to a file.
    Export(ExportArgs),
    /// Import a seed blob into the current dialect.
    Import(ImportArgs),
    /// Create a new dialect and make it current (old seeds preserved).
    Rotate(RotateArgs),
    /// Verify the seed matches the stored dialect id.
    Verify(VerifyArgs),
    /// Encrypt a currently-public seed.
    Lock(LockArgs),
    /// Decrypt a currently-secret seed.
    Unlock(UnlockArgs),
    /// Write an encrypted emergency backup, or print equivalent phrases.
    Backup(BackupArgs),
    /// Restore the seed from a backup file or a recovery phrase.
    Recover(RecoverArgs),
    /// Print the recovery phrase for the current seed.
    Phrase(PhraseArgs),
    /// Split the seed into Shamir shares (write one phrase file per share).
    Split(SplitArgs),
    /// Recombine Shamir shares into the seed.
    Combine(CombineArgs),
}

#[derive(Args)]
pub struct ShowArgs {
    #[arg(long)]
    pub public: bool,
}
#[derive(Args)]
pub struct NewArgs {}
#[derive(Args)]
pub struct ExportArgs {}
#[derive(Args)]
pub struct ImportArgs {
    pub file: String,
}
#[derive(Args)]
pub struct RotateArgs {}
#[derive(Args)]
pub struct VerifyArgs {}
#[derive(Args)]
pub struct LockArgs {}
#[derive(Args)]
pub struct UnlockArgs {}
#[derive(Args)]
pub struct BackupArgs {
    /// Print an equivalent recovery phrase for each of these languages.
    #[arg(long)]
    pub equivalent: Option<String>,
    #[arg(short, long)]
    pub out: Option<std::path::PathBuf>,
    #[arg(short, long)]
    pub pass: Option<String>,
}
#[derive(Args)]
pub struct RecoverArgs {
    #[arg(short, long)]
    pub input: Option<std::path::PathBuf>,
    #[arg(short, long)]
    pub pass: Option<String>,
    #[arg(long)]
    pub phrase: Option<String>,
}
#[derive(Args)]
pub struct PhraseArgs {
    /// Language code for the phrase (en, es, fr, it). Default: en.
    #[arg(long, default_value = "en")]
    pub lang: String,
}
#[derive(Args)]
pub struct SplitArgs {
    /// Total number of shares to create.
    #[arg(long, default_value_t = 5)]
    pub shares: u8,
    /// Shares required to reconstruct.
    #[arg(long, default_value_t = 3)]
    pub threshold: u8,
    /// Language for the share phrases (en, es, fr, it). Default: en.
    #[arg(long, default_value = "en")]
    pub lang: String,
    /// Directory to write share files into.
    #[arg(short, long)]
    pub out: Option<std::path::PathBuf>,
}
#[derive(Args)]
pub struct CombineArgs {
    /// Share phrase files (any `threshold` of them).
    pub files: Vec<String>,
}

pub fn run(a: SeedArgs) -> anyhow::Result<()> {
    match a.cmd {
        SeedSub::Show(a) => cmd_show(a),
        SeedSub::New(_) => regenerate(None),
        SeedSub::Export(_) => cmd_export(),
        SeedSub::Import(a) => {
            let data = std::fs::read(&a.file)?;
            import_seed(&data)
        }
        SeedSub::Rotate(_) => rotate(),
        SeedSub::Verify(_) => cmd_verify(),
        SeedSub::Lock(_) => {
            let pass = prompt_secret("backup passphrase: ")?;
            lock_seed(&pass)
        }
        SeedSub::Unlock(_) => {
            let pass = prompt_secret("seed passphrase: ")?;
            unlock(&pass)
        }
        SeedSub::Backup(a) => {
            if let Some(langs) = a.equivalent {
                let list: Vec<String> =
                    langs.split(',').map(|s| s.trim().to_string()).collect();
                show_phrase_langs(&list)
            } else {
                let pass = match a.pass {
                    Some(p) => p,
                    None => prompt_secret("backup passphrase: ")?,
                };
                let out = a
                    .out
                    .unwrap_or_else(|| std::path::PathBuf::from("glyph-seed-backup.enc"));
                backup_seed(&out, &pass)
            }
        }
        SeedSub::Recover(a) => {
            if let Some(phrase) = a.phrase {
                recover_seed_phrase(&phrase)
            } else {
                let input = a
                    .input
                    .ok_or_else(|| anyhow::anyhow!("provide --input <file> or --phrase"))?;
                let pass = match a.pass {
                    Some(p) => p,
                    None => prompt_secret("backup passphrase: ")?,
                };
                recover_seed(&input, &pass)
            }
        }
        SeedSub::Phrase(a) => show_phrase_lang(&a.lang),
        SeedSub::Split(a) => cmd_split(a),
        SeedSub::Combine(a) => cmd_combine(a),
    }
}

fn cmd_show(a: ShowArgs) -> anyhow::Result<()> {
    let (seed, _) = load_seed()?;
    let id = dialect_id(&seed);
    if a.public {
        println!("{id}");
        return Ok(());
    }
    let d = build(&seed);
    println!("dialect id : {id}");
    println!("glyphs     : {}", d.op_to_glyph.len());
    for (op, g) in &d.op_to_glyph {
        println!("  {g}  {op}");
    }
    Ok(())
}

fn cmd_export() -> anyhow::Result<()> {
    let data = export_seed()?;
    let path = glyph::store::init::export_path();
    std::fs::write(&path, &data)?;
    println!("✓ seed exported to {}", path.display());
    Ok(())
}

fn cmd_verify() -> anyhow::Result<()> {
    let (seed, _) = load_seed()?;
    let id = current_dialect_id()?;
    let expected = dialect_id(&seed);
    if id == expected {
        println!("✓ seed matches dialect id {id}");
        Ok(())
    } else {
        anyhow::bail!("seed does NOT match stored dialect id {id} (got {expected})")
    }
}

fn cmd_split(a: SplitArgs) -> anyhow::Result<()> {
    let (seed, _) = load_seed()?;
    let mut rng = rand::thread_rng();
    let shares = shamir::split(seed.as_slice(), a.shares, a.threshold, &mut rng)?;
    let dir = a
        .out
        .unwrap_or_else(|| std::path::PathBuf::from("glyph-shares"));
    std::fs::create_dir_all(&dir)?;
    for (x, y) in &shares {
        let mut blob = vec![*x];
        blob.extend_from_slice(y);
        let phrase = mnemonic::bytes_to_mnemonic(&blob, &a.lang)?;
        let path = dir.join(format!("share-{x}.txt"));
        let content = format!(
            "# Glyph Shamir share — GLYPH-SHARE v1\n# x={x} lang={}\n{phrase}\n",
            a.lang
        );
        std::fs::write(&path, content)?;
        println!("  wrote {}", path.display());
    }
    println!(
        "✓ {} shares ({}/{} threshold) written to {}",
        a.shares, a.threshold, a.shares, dir.display()
    );
    println!("  any {} shares reconstruct the seed; fewer reveal nothing", a.threshold);
    Ok(())
}

fn cmd_combine(a: CombineArgs) -> anyhow::Result<()> {
    if a.files.is_empty() {
        anyhow::bail!("provide at least one share file");
    }
    let mut shares = Vec::new();
    for f in &a.files {
        let text = std::fs::read_to_string(f)?;
        let mut prefer_lang: Option<String> = None;
        let phrase: String = text
            .lines()
            .filter_map(|l| {
                let t = l.trim_start();
                if let Some(rest) = t.strip_prefix("#") {
                    if let Some(idx) = rest.find("lang=") {
                        let lang = rest[idx + 5..].trim().split_whitespace().next().unwrap_or("");
                        if !lang.is_empty() {
                            prefer_lang = Some(lang.to_string());
                        }
                    }
                    None
                } else if !l.trim().is_empty() {
                    Some(l.trim())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let blob = mnemonic::mnemonic_to_bytes(&phrase, prefer_lang.as_deref())?;
        if blob.len() != 33 {
            anyhow::bail!("share {} no tiene 33 bytes (formato inválido)", f);
        }
        let x = blob[0];
        let y = blob[1..].to_vec();
        shares.push((x, y));
    }
    let secret = shamir::combine(&shares)?;
    if secret.len() != 32 {
        anyhow::bail!("el secreto reconstruido no tiene 32 bytes");
    }
    let mut s = [0u8; 32];
    s.copy_from_slice(&secret);
    set_current_seed(&s)?;
    println!("✓ reconstructed seed from {} shares", shares.len());
    Ok(())
}
