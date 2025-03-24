pub mod action;
pub mod config;
pub mod utils;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = action::AppArgs::parse();

    args.run().await
}
