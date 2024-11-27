use crate::app::ListenerApp;
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Command {
    #[command(about = "Starts listening to solana for new events")]
    Listen {
        #[arg(long, help = "Config path")]
        config: String,
    },
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub(super) struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub(super) async fn execute(args: impl Iterator<Item = String>) {
        let mut parsed_cli = Self::parse_from(args);
        match &mut parsed_cli.command {
            Command::Listen { config } => ListenerApp::execute(config).await,
        }
    }
}
