use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "merryl")]
#[command(about = "Market rotation intelligence engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        #[command(subcommand)]
        workflow: RunWorkflow,
    },
    Status,
    Doctor,
    Db {
        #[command(subcommand)]
        command: DbCommand,
    },
}

#[derive(Subcommand)]
enum RunWorkflow {
    Daily {
        #[arg(long, default_value = "latest")]
        date: String,
    },
}

#[derive(Subcommand)]
enum DbCommand {
    Migrate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { workflow } => match workflow {
            RunWorkflow::Daily { date } => {
                let result = merryl::workflows::run_daily(&date)?;
                println!("Daily market rotation run completed.");
                println!("date: {}", result.date);
                println!("database: {}", result.database.display());
                println!("report: {}", result.report.display());
                println!("sector export: {}", result.sector_export.display());
                println!("watchlist export: {}", result.watchlist_export.display());
            }
        },
        Commands::Status => {
            let status = merryl::workflows::status()?;
            println!("{status}");
        }
        Commands::Doctor => {
            let checks = merryl::workflows::doctor()?;
            for check in checks {
                println!("{check}");
            }
        }
        Commands::Db { command } => match command {
            DbCommand::Migrate => {
                let path = merryl::storage::default_db_path();
                let db = merryl::storage::Database::open(&path)?;
                db.migrate()?;
                println!("database migrated: {}", path.display());
            }
        },
    }

    Ok(())
}
