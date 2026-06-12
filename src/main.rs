use anyhow::Result;
use clap::{Parser, Subcommand};
use merryl::config::dashboard::DEFAULT_PORT;
use std::path::PathBuf;

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
        #[arg(long, value_name = "DIR")]
        export_static: Option<PathBuf>,
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
    Intraday {
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
                for warning in result.warnings {
                    println!("warning: {warning}");
                }
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
                println!(
                    "event context validation report: {}",
                    result.event_context_validation_report.display()
                );
                println!(
                    "event context validation export: {}",
                    result.event_context_validation_export.display()
                );
                println!(
                    "actionability validation report: {}",
                    result.actionability_validation_report.display()
                );
                println!(
                    "actionability validation export: {}",
                    result.actionability_validation_export.display()
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
                println!(
                    "event context observations: {}",
                    result.event_context_observation_count
                );
                println!(
                    "actionability observations: {}",
                    result.actionability_observation_count
                );
                println!("backtest result id: {}", result.backtest_result_id);
            }
            RunWorkflow::Intraday { date } => {
                let result = merryl::workflows::run_intraday(&date)?;
                println!("Intraday execution readiness run:");
                println!("score date: {}", result.date);
                println!("database: {}", result.database.display());
                println!("profile timeframe: {}", result.profile_timeframe);
                println!("trigger timeframe: {}", result.trigger_timeframe);
                println!("candidate count: {}", result.candidate_count);
                println!("stage 1 count: {}", result.stage1_count);
                println!("stage 2 count: {}", result.stage2_count);
                println!("stage 3 trigger count: {}", result.stage3_trigger_count);
                println!("report: {}", result.report.display());
                println!("export: {}", result.export.display());
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
        Commands::Dashboard {
            port,
            export_static,
        } => {
            if let Some(output_dir) = export_static {
                let export = merryl::dashboard::export_static_dashboard(
                    &merryl::storage::default_db_path(),
                    &output_dir,
                )?;
                println!("Static dashboard export:");
                println!("output: {}", export.output_dir.display());
                println!("dates: {}", export.dates_path.display());
                println!("latest: {}", export.latest_snapshot_path.display());
                println!("snapshots: {}", export.snapshot_count);
            } else {
                let runtime = tokio::runtime::Runtime::new()?;
                runtime.block_on(merryl::dashboard::run_dashboard(
                    merryl::dashboard::DashboardServerConfig::local(port),
                ))?;
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
