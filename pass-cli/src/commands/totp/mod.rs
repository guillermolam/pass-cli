mod generate;

use crate::commands::OutputFormat;
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TotpCommands {
    #[command(about = "Generate a TOTP token from a secret or URI")]
    Generate {
        #[arg(help = "TOTP secret (base32) or URI (otpauth://...)")]
        secret_or_uri: String,
        #[arg(long, default_value = "human")]
        output: OutputFormat,
    },
}

pub async fn run(command: &TotpCommands) -> Result<()> {
    match command {
        TotpCommands::Generate {
            secret_or_uri,
            output,
        } => generate::run(secret_or_uri, output).await,
    }
}
