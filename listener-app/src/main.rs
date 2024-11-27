mod app;
mod cli;
mod config;
mod data;
mod error;
mod postgres_writer;
mod solana_logs_processor;

use std::env;

#[tokio::main]
async fn main() {
    env_logger::init();
    cli::Cli::execute(env::args()).await;
}
