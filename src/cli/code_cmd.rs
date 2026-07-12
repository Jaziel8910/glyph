use clap::Args;

use glyph::compiler;

#[derive(Args)]
pub struct BuildArgs {
    /// Glyph source file (`.glf`).
    pub input: std::path::PathBuf,
    /// Output path (default: <input>.py).
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct ObfuscateArgs {
    /// Real source file to obfuscate (e.g. a `.py`).
    pub input: std::path::PathBuf,
    /// Output path (default: <input>.glf).
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct RevealArgs {
    /// Obfuscated `.glf` file to reveal.
    pub input: std::path::PathBuf,
    /// Output path (default: <input>.py).
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

fn out_path(input: &std::path::Path, ext: &str, given: &Option<std::path::PathBuf>) -> std::path::PathBuf {
    given
        .clone()
        .unwrap_or_else(|| input.with_extension(ext))
}

/// Compile a plain `.glf` to Python.
pub fn run_build(a: BuildArgs) -> anyhow::Result<()> {
    let out = out_path(&a.input, "py", &a.output);
    compiler::build(&a.input, &out)
}

/// Obfuscate a real `.py` into an encrypted `.glf` (Python -> IR -> glyph -> AES).
pub fn run_obfuscate(a: ObfuscateArgs) -> anyhow::Result<()> {
    let out = out_path(&a.input, "glf", &a.output);
    compiler::obfuscate(&a.input, &out)
}

/// Reveal an encrypted `.glf` back to Python (AES -> glyph -> IR -> Python).
pub fn run_reveal(a: RevealArgs) -> anyhow::Result<()> {
    let out = out_path(&a.input, "py", &a.output);
    compiler::reveal(&a.input, &out)
}
