use clap::{Args, Parser, Subcommand};

use crate::{config, utils};

#[derive(Args, Debug, Clone)]
pub struct RunMigrationsArgs {
    #[arg(short, long)]
    file: String,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Action {
    RunMigrations(RunMigrationsArgs),
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct AppArgs {
    #[command(subcommand)]
    pub action: Action,
}

impl AppArgs {
    pub async fn run(&self) -> anyhow::Result<()> {
        match &self.action {
            Action::RunMigrations(RunMigrationsArgs { file }) => {
                let db_pool = utils::setup_sqlite_db_pool(config::APP_CONFIG.is_prod()).await?;

                utils::run_migrations(&db_pool, file).await
            }
        }
    }
}
