use anyhow::{Context, Result};
use colored::Colorize;
use ethers::types::{Block, Transaction, U64, U256};
use std::collections::HashMap;

use crate::rpc::{EthereumRpcClient, wei_to_eth, wei_to_gwei};
use crate::types::*;

pub struct BlockAnalyzer {
    client: EthereumRpcClient,
}

impl BlockAnalyzer {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        println!("Connecting to Ethereum node at {}...", rpc_url);
        let client = EthereumRpcClient::new(rpc_url).await?;
        println!("✓ Connected successfully!\n");
        Ok(Self { client })
    }

    /// Analyze a single block with detailed output
    pub async fn analyze_single_block(&self, block_id: &str, verbose: bool) -> Result<()> {
        let block = self
            .client
            .get_block(block_id)
            .await?
            .context("Block not found")?;

        let lifecycle = self.analyze_block(&block).await?;
        println!("{}", lifecycle);

        if verbose {
            self.print_transaction_details(&block, &lifecycle).await?;
        }

        Ok(())
    }

    /// Analyze a range of blocks
    pub async fn analyze_range(&self, start: u64, end: u64) -> Result<Vec<BlockLifecycle>> {
        println!(
            "Analyzing blocks {} to {} ({} blocks)...\n",
            start,
            end,
            end - start + 1
        );

        let mut results = Vec::new();

        for block_num in start..=end {
            print!("  Block {}: ", block_num);

            match self.client.get_block(&block_num.to_string()).await? {
                Some(block) => {
                    let lifecycle = self.analyze_block(&block).await?;
                    println!(
                        "✓ {} txs, {:.2} gwei base fee",
                        lifecycle.transactions.total_count, lifecycle.gas.base_fee_gwei
                    );
                    results.push(lifecycle);
                }
                None => {
                    println!("✗ Not found");
                }
            }
        }

        println!("\n✓ Analysis complete!");
        Ok(results)
    }

    /// Monitor live blocks
    pub async fn monitor_live(&self, count: u64, output: Option<String>) -> Result<()> {
        println!("Monitoring live blocks...\n");

        let mut results = Vec::new();
        let mut last_block = self.client.get_latest_block_number().await?;

        let iterations = if count == 0 { u64::MAX } else { count };

        for i in 0..iterations {
            let current = self.client.get_latest_block_number().await?;

            if current > last_block {
                for block_num in (last_block + 1)..=current {
                    if let Some(block) = self.client.get_block(&block_num.to_string()).await? {
                        let lifecycle = self.analyze_block(&block).await?;
                        println!("{}", lifecycle);
                        results.push(lifecycle);
                    }
                }
                last_block = current;
            }

            if i < iterations - 1 {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        }

        if let Some(path) = output {
            crate::exporter::Exporter::export_to_csv(&results, &path)?;
            println!("\n✓ Exported {} blocks to {}", results.len(), path);
        }

        Ok(())
    }

    /// Detect MEV in recent blocks
    pub async fn detect_mev(&self, blocks: u64, threshold: f64) -> Result<()> {
        let latest = self.client.get_latest_block_number().await?;
        let start = latest.saturating_sub(blocks);

        println!(
            "Analyzing {} blocks for MEV (threshold: {} ETH)...\n",
            blocks, threshold
        );

        let mut total_mev = 0.0;
        let mut blocks_with_mev = 0;

        for block_num in start..=latest {
            if let Some(block) = self.client.get_block(&block_num.to_string()).await? {
                let lifecycle = self.analyze_block(&block).await?;

                if lifecycle.mev.estimated_mev_eth >= threshold {
                    blocks_with_mev += 1;
                    total_mev += lifecycle.mev.estimated_mev_eth;

                    println!(
                        "{} Block {}: {:.4} ETH MEV detected",
                        "s".yellow(),
                        block_num,
                        lifecycle.mev.estimated_mev_eth
                    );

                    if !lifecycle.mev.sandwich_attacks.is_empty() {
                        println!(
                            "   └─ {} sandwich attacks",
                            lifecycle.mev.sandwich_attacks.len()
                        );
                    }
                    if !lifecycle.mev.arbitrage_ops.is_empty() {
                        println!(
                            "   └─ {} arbitrage opportunities",
                            lifecycle.mev.arbitrage_ops.len()
                        );
                    }
                }
            }
        }

        println!("\n{}", "═══════════════════════════════════════".cyan());
        println!("Blocks analyzed: {}", blocks);
        println!("Blocks with MEV: {}", blocks_with_mev);
        println!("Total MEV extracted: {:.4} ETH", total_mev);
        println!(
            "Average MEV per block: {:.4} ETH",
            total_mev / blocks as f64
        );
        println!("{}", "═══════════════════════════════════════".cyan());

        Ok(())
    }

    /// Core block analysis logic
    async fn analyze_block(&self, block: &Block<Transaction>) -> Result<BlockLifecycle> {
        let block_number = block.number.unwrap_or_default().as_u64();
        let block_hash = format!("{:?}", block.hash.unwrap_or_default());
        let timestamp = block.timestamp.as_u64();

        // Get previous block for timing
        let prev_block = self.client.get_previous_block(block_number).await?;
        let block_time = if let Some(prev) = &prev_block {
            (timestamp - prev.timestamp.as_u64()) as f64
        } else {
            0.0
        };

        // Timing metrics
        let timing = TimingMetrics {
            block_time,
            timestamp,
            propagation_delay: None, // Would need network data
        };

        // Gas metrics
        let gas = self.calculate_gas_metrics(block);

        // Transaction metrics
        let transactions = self.analyze_transactions(block);

        // MEV indicators
        let mev = self.detect_mev_indicators(block);

        // PBS metrics
        let pbs = self.analyze_pbs(block);

        Ok(BlockLifecycle {
            block_number,
            block_hash,
            timestamp,
            proposer: format!("{:?}", block.author.unwrap_or_default()),
            builder: pbs.builder_address.clone(),
            timing,
            gas,
            transactions,
            mev,
            pbs,
        })
    }

    fn calculate_gas_metrics(&self, block: &Block<Transaction>) -> GasMetrics {
        let gas_used = block.gas_used.as_u64();
        let gas_limit = block.gas_limit.as_u64();
        let utilization = (gas_used as f64 / gas_limit as f64) * 100.0;

        let base_fee_gwei = block
            .base_fee_per_gas
            .map(|bf| wei_to_gwei(bf))
            .unwrap_or(0.0);

        // Calculate average priority fee
        let mut total_priority_fee = U256::zero();
        let mut priority_fee_count = 0;

        for tx in &block.transactions {
            if let Some(max_priority) = tx.max_priority_fee_per_gas {
                total_priority_fee += max_priority;
                priority_fee_count += 1;
            }
        }

        let avg_priority_fee_gwei = if priority_fee_count > 0 {
            wei_to_gwei(total_priority_fee / priority_fee_count)
        } else {
            0.0
        };

        // Calculate fees burned (base fee * gas used)
        let fees_burned_eth = if let Some(base_fee) = block.base_fee_per_gas {
            wei_to_eth(base_fee * gas_used)
        } else {
            0.0
        };

        // Calculate priority fees to proposer
        let priority_fees_eth = wei_to_eth(total_priority_fee);

        GasMetrics {
            gas_used,
            gas_limit,
            utilization,
            base_fee_gwei,
            avg_priority_fee_gwei,
            fees_burned_eth,
            priority_fees_eth,
        }
    }

    fn analyze_transactions(&self, block: &Block<Transaction>) -> TransactionMetrics {
        let total_count = block.transactions.len();
        let mut type_breakdown = TypeBreakdown {
            legacy: 0,
            eip2930: 0,
            eip1559: 0,
            eip4844_blob: 0,
        };

        let failed_count = 0;

        for tx in &block.transactions {
            match tx.transaction_type {
                Some(t) if t == U64::from(0) => type_breakdown.legacy += 1,
                Some(t) if t == U64::from(1) => type_breakdown.eip2930 += 1,
                Some(t) if t == U64::from(2) => type_breakdown.eip1559 += 1,
                Some(t) if t == U64::from(3) => type_breakdown.eip4844_blob += 1,
                _ => type_breakdown.legacy += 1,
            }
        }

        // Analyze transaction ordering
        let ordering = self.analyze_tx_ordering(&block.transactions);

        TransactionMetrics {
            total_count,
            type_breakdown,
            ordering,
            failed_count, // Would need receipts to determine
        }
    }

    fn analyze_tx_ordering(&self, transactions: &[Transaction]) -> OrderingMetrics {
        // Check if transactions are sorted by priority fee
        let mut sorted_by_priority = true;
        let mut anomalies = 0;

        for i in 1..transactions.len() {
            if let (Some(prev_fee), Some(curr_fee)) = (
                transactions[i - 1].max_priority_fee_per_gas,
                transactions[i].max_priority_fee_per_gas,
            ) {
                if curr_fee > prev_fee {
                    sorted_by_priority = false;
                    anomalies += 1;
                }
            }
        }

        OrderingMetrics {
            sorted_by_priority,
            anomalies,
            avg_deviation: 0.0, // Simplified
        }
    }

    fn detect_mev_indicators(&self, block: &Block<Transaction>) -> MevIndicators {
        let sandwich_attacks = Vec::new();
        let arbitrage_ops = Vec::new();
        let liquidations = 0;
        let mut estimated_mev_eth = 0.0;
        let mut mev_bot_addresses = Vec::new();

        // Simple heuristics for MEV detection
        let txs = &block.transactions;

        // Detect potential sandwich attacks (same address appears at different positions)
        let mut address_positions: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, tx) in txs.iter().enumerate() {
            let addr = format!("{:?}", tx.from);
            address_positions
                .entry(addr)
                .or_insert_with(Vec::new)
                .push(i);
        }

        for (addr, positions) in address_positions {
            if positions.len() >= 2 {
                // Potential sandwich if address appears multiple times
                if self.client.is_known_mev_bot(&addr) {
                    mev_bot_addresses.push(addr.clone());
                }
            }
        }

        // Estimate MEV based on priority fees of potential MEV transactions
        for tx in txs {
            if let Some(priority_fee) = tx.max_priority_fee_per_gas {
                let addr = format!("{:?}", tx.from);
                if self.client.is_known_mev_bot(&addr) {
                    estimated_mev_eth += wei_to_eth(priority_fee * tx.gas);
                }
            }
        }

        MevIndicators {
            sandwich_attacks,
            arbitrage_ops,
            liquidations,
            estimated_mev_eth,
            mev_bot_addresses,
        }
    }

    fn analyze_pbs(&self, block: &Block<Transaction>) -> PbsMetrics {
        let extra_data = String::from_utf8_lossy(&block.extra_data.0).to_string();

        // Detect PBS builders from extra_data
        let known_builders = vec!["flashbots", "builder0x69", "rsync", "beaverbuild"];
        let is_pbs_block = known_builders
            .iter()
            .any(|b| extra_data.to_lowercase().contains(b));

        let builder_address = if is_pbs_block {
            Some(extra_data.clone())
        } else {
            None
        };

        PbsMetrics {
            is_pbs_block,
            builder_address,
            builder_payment_eth: None, // Would need to parse coinbase tx
            extra_data,
        }
    }

    #[allow(dead_code)]
    async fn print_transaction_details(
        &self,
        block: &Block<Transaction>,
        lifecycle: &BlockLifecycle,
    ) -> Result<()> {
        println!("\n{}", "TRANSACTION DETAILS".green().bold());
        println!("{}", "─".repeat(50));

        for (i, tx) in block.transactions.iter().take(10).enumerate() {
            println!("\nTx #{}: {}", i + 1, format!("{:?}", tx.hash).yellow());
            println!("  From: {:?}", tx.from);
            println!("  To: {:?}", tx.to);
            println!("  Value: {} ETH", wei_to_eth(tx.value));
            if let Some(max_fee) = tx.max_fee_per_gas {
                println!("  Max Fee: {} gwei", wei_to_gwei(max_fee));
            }
            if let Some(priority) = tx.max_priority_fee_per_gas {
                println!("  Priority Fee: {} gwei", wei_to_gwei(priority));
            }
        }

        if block.transactions.len() > 10 {
            println!(
                "\n... and {} more transactions",
                block.transactions.len() - 10
            );
        }

        Ok(())
    }
}
