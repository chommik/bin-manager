use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Don't check file checksums, skip if target files exist
    #[arg(long)]
    pub skip_checksum: bool,

    /// Skip actions on filesystem, just tell what's done
    #[arg(long, short = 'n')]
    pub dry_run: bool,

    /// Path to binary list
    #[arg(long)]
    pub config_dir: Option<String>,

    /// Path to target directory
    #[arg(long)]
    pub target_dir: Option<String>,
}