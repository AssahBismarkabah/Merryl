use anyhow::Result;
use clap::{Parser, Subcommand};
use merryl::config::dashboard::DEFAULT_PORT;

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
    Dashboard {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
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
    Backtest {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
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
                println!("Daily market rotation run:");
                println!("date: {}", result.date);
                println!("database: {}", result.database.display());
                println!("report: {}", result.report.display());
                println!("sector export: {}", result.sector_export.display());
                println!("watchlist export: {}", result.watchlist_export.display());
                println!("historical score dates: {}", result.historical_score_dates);
                println!("macro observations: {}", result.macro_observations);
                println!("news events: {}", result.news_events);
                println!("earnings events: {}", result.earnings_events);
                println!("filing events: {}", result.filing_events);
            }
            RunWorkflow::Backtest { from, to } => {
                let result = merryl::workflows::run_backtest(&from, &to)?;
                println!("Backtest run:");
                println!("from: {}", result.from_date);
                println!("to: {}", result.to_date);
                println!("database: {}", result.database.display());
                println!("report: {}", result.report.display());
                println!("summary export: {}", result.summary_export.display());
                println!(
                    "macro regime validation report: {}",
                    result.macro_regime_validation_report.display()
                );
                println!(
                    "macro regime validation export: {}",
                    result.macro_regime_validation_export.display()
                );
                println!("sector observations: {}", result.sector_observation_count);
                println!(
                    "sector component observations: {}",
                    result.sector_component_observation_count
                );
                println!("stock observations: {}", result.stock_observation_count);
                println!(
                    "industry validation observations: {}",
                    result.industry_stock_observation_count
                );
                println!(
                    "macro regime snapshots: {}",
                    result.macro_regime_snapshot_count
                );
                println!("backtest result id: {}", result.backtest_result_id);
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
        Commands::Dashboard { port } => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(merryl::dashboard::run_dashboard(
                merryl::dashboard::DashboardServerConfig::local(port),
            ))?;
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
