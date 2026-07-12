mod cli;

use clap::{Parser, Subcommand};

use cli::code_cmd::{BuildArgs, ObfuscateArgs, RevealArgs};
use cli::init_cmd::InitArgs;
use cli::seed_cmd::SeedArgs;

#[derive(Parser)]
#[command(
    name = "glyph",
    version,
    about = "Glyph — agent & human-facing glyph CLI (seed engine + store + F1 compiler)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a glyph project (generates a seed + dialect).
    Init(InitArgs),
    /// Manage the project seed (show/new/rotate/backup/recover/phrase/...).
    Seed(SeedArgs),
    /// Compile a plain `.glf` to Python.
    Build(BuildArgs),
    /// Obfuscate a real `.py` into an encrypted `.glf` (Python -> glyph).
    Obfuscate(ObfuscateArgs),
    /// Reveal a `.glf` obfuscated by `obfuscate` back to Python.
    Reveal(RevealArgs),
}

fn main() {
    let cli = Cli::parse();
    let res = match cli.command {
        Commands::Init(a) => cli::init_cmd::run(a),
        Commands::Seed(a) => cli::seed_cmd::run(a),
        Commands::Build(a) => cli::code_cmd::run_build(a),
        Commands::Obfuscate(a) => cli::code_cmd::run_obfuscate(a),
        Commands::Reveal(a) => cli::code_cmd::run_reveal(a),
    };
    if let Err(e) = res {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
