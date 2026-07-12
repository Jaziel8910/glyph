use clap::Args;

#[derive(Args)]
pub struct InitArgs {
    /// Store the seed encrypted (passphrase required on each use).
    #[arg(long)]
    pub secret: bool,
    /// Initialize from a given seed string instead of random entropy.
    #[arg(long)]
    pub seed: Option<String>,
    /// Also print the 32-word recovery phrase after init.
    #[arg(long)]
    pub phrase: bool,
}

pub fn run(a: InitArgs) -> anyhow::Result<()> {
    glyph::store::init::init_project(a.secret, a.seed.as_deref())?;
    if a.phrase {
        glyph::store::init::show_phrase()?;
    }
    Ok(())
}
