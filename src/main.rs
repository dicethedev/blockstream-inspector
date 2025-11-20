use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::env;

mod analyzer;
mod exporter;
mod rpc;
mod types;

use analyzer::BlockAnalyzer;
use exporter::Exporter;

#[derive(Parser)]
#[command(name = "BlockStream Inspector")]
#[command(about = "BlockStream Inspector — An advanced analytics tool for examining Ethereum block production, propagation timing, transaction ordering, and MEV extraction trends", long_about = None)]

struct Cli {
    //NB I will make use of my own RPC Later in this project
    /// RPC endpoint URL (e.g., http://localhost:8545 or Infura/Alchemy URL)
    // #[arg(
    //     short,
    //     long,
    //     env = "ALCHEMY_RPC_URL",
    //     default_value = "http://localhost:8545"
    // )]
    /// RPC endpoint URL (e.g., Alchemy/Infura URL)
    #[arg(short, long)]
    rpc: Option<String>, // optional so build from ALCHEMY_API_KEY if missing

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a single block
    Block {
        /// Block number or 'latest'
        #[arg(short, long)]
        number: String,

        /// Show detailed transaction analysis
        #[arg(short, long)]
        verbose: bool,
    },

    /// Analyze a range of blocks
    Range {
        /// Start block number
        #[arg(short, long)]
        start: u64,

        /// End block number
        #[arg(short, long)]
        end: u64,

        /// Export to CSV
        #[arg(short = 'o', long)]
        output: Option<String>,
    },

    /// Live monitoring mode
    Live {
        /// Number of blocks to monitor (0 = infinite)
        #[arg(short, long, default_value = "10")]
        count: u64,

        /// Export to CSV
        #[arg(short = 'o', long)]
        output: Option<String>,
    },

    /// Detect MEV opportunities in recent blocks
    Mev {
        /// Number of recent blocks to analyze
        #[arg(short, long, default_value = "100")]
        blocks: u64,

        /// Minimum profit threshold in ETH
        #[arg(short, long, default_value = "0.1")]
        threshold: f64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {

    dotenv().ok();

    let cli = Cli::parse();

    // Determine the RPC URL
    let rpc_url = cli.rpc.clone()
    .or_else(|| env::var("ALCHEMY_RPC_URL").ok())
    .or_else(|| env::var("ALCHEMY_API_KEY").ok().map(|key| format!("https://eth-mainnet.g.alchemy.com/v2/{}", key)))
    .unwrap_or_else(|| {
        eprintln!("ERROR: No valid Alchemy RPC key found. Please set ALCHEMY_RPC_URL or ALCHEMY_API_KEY.");
        std::process::exit(1);
    });

    println!("Using RPC URL: {}", rpc_url);

    let analyzer = BlockAnalyzer::new(&rpc_url).await?;

    match cli.command {
        Commands::Block { number, verbose } => {
            analyzer.analyze_single_block(&number, verbose).await?;
        }
        Commands::Range { start, end, output } => {
            let results = analyzer.analyze_range(start, end).await?;

            if let Some(path) = output {
                Exporter::export_to_csv(&results, &path)?;
                println!("✓ Exported {} blocks to {}", results.len(), path);
            }
        }
        Commands::Live { count, output } => {
            analyzer.monitor_live(count, output).await?;
        }
        Commands::Mev { blocks, threshold } => {
            analyzer.detect_mev(blocks, threshold).await?;
        }
    }

    Ok(())
}
