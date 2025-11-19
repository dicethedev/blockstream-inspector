use serde::{Deserialize, Serialize};
use std::fmt;

/// Complete block lifecycle analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockLifecycle {
    pub block_number: u64,
    pub block_hash: String,
    pub timestamp: u64,
    pub proposer: String,
    pub builder: Option<String>,
    
    // Timing metrics
    pub timing: TimingMetrics,
    
    // Gas analysis
    pub gas: GasMetrics,
    
    // Transaction analysis
    pub transactions: TransactionMetrics,
    
    // MEV indicators
    pub mev: MevIndicators,
    
    // PBS (Proposer-Builder Separation) data
    pub pbs: PbsMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingMetrics {
    /// Block production time (seconds since previous block)
    pub block_time: f64,
    
    /// Timestamp of this block
    pub timestamp: u64,
    
    /// Estimated propagation delay (if available)
    pub propagation_delay: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasMetrics {
    /// Gas used in this block
    pub gas_used: u64,
    
    /// Gas limit for this block
    pub gas_limit: u64,
    
    /// Utilization percentage
    pub utilization: f64,
    
    /// Base fee per gas (in gwei)
    pub base_fee_gwei: f64,
    
    /// Average priority fee (in gwei)
    pub avg_priority_fee_gwei: f64,
    
    /// Total gas fees burned (in ETH)
    pub fees_burned_eth: f64,
    
    /// Total priority fees to proposer (in ETH)
    pub priority_fees_eth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetrics {
    /// Total number of transactions
    pub total_count: usize,
    
    /// Breakdown by transaction type
    pub type_breakdown: TypeBreakdown,
    
    /// Transaction ordering analysis
    pub ordering: OrderingMetrics,
    
    /// Failed transactions
    pub failed_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeBreakdown {
    pub legacy: usize,
    pub eip2930: usize,
    pub eip1559: usize,
    pub eip4844_blob: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderingMetrics {
    /// Transactions sorted by priority fee (descending)
    pub sorted_by_priority: bool,
    
    /// Number of ordering anomalies detected
    pub anomalies: usize,
    
    /// Average position deviation
    pub avg_deviation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MevIndicators {
    /// Potential sandwich attacks detected
    pub sandwich_attacks: Vec<SandwichAttack>,
    
    /// DEX arbitrage opportunities
    pub arbitrage_ops: Vec<ArbitrageOp>,
    
    /// Liquidations detected
    pub liquidations: usize,
    
    /// Total estimated MEV value (in ETH)
    pub estimated_mev_eth: f64,
    
    /// Known MEV bot addresses in this block
    pub mev_bot_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandwichAttack {
    pub frontrun_tx: String,
    pub victim_tx: String,
    pub backrun_tx: String,
    pub estimated_profit_eth: f64,
    pub dex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOp {
    pub tx_hash: String,
    pub path: Vec<String>, // Token swap path
    pub estimated_profit_eth: f64,
    pub dexes_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbsMetrics {
    /// Was this block built via PBS?
    pub is_pbs_block: bool,
    
    /// Builder address (if identifiable)
    pub builder_address: Option<String>,
    
    /// Estimated builder payment to proposer (in ETH)
    pub builder_payment_eth: Option<f64>,
    
    /// Extra data field (often contains builder info)
    pub extra_data: String,
}

impl fmt::Display for BlockLifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::Colorize;
        
        writeln!(f, "\n{}", "═══════════════════════════════════════════════════".cyan())?;
        writeln!(f, "{} {}", "Block Number:".bold(), self.block_number.to_string().yellow())?;
        writeln!(f, "{} {}", "Block Hash:".bold(), self.block_hash)?;
        writeln!(f, "{} {}", "Timestamp:".bold(), self.timestamp)?;
        writeln!(f, "{}", "═══════════════════════════════════════════════════".cyan())?;
        
        writeln!(f, "\n{}", "TIMING METRICS".green().bold())?;
        writeln!(f, "  Block Time: {:.2}s", self.timing.block_time)?;
        
        writeln!(f, "\n{}", "GAS METRICS".green().bold())?;
        writeln!(f, "  Gas Used: {} / {} ({:.1}%)", 
            self.gas.gas_used, 
            self.gas.gas_limit, 
            self.gas.utilization
        )?;
        writeln!(f, "  Base Fee: {:.2} gwei", self.gas.base_fee_gwei)?;
        writeln!(f, "  Avg Priority Fee: {:.2} gwei", self.gas.avg_priority_fee_gwei)?;
        writeln!(f, "  Fees Burned: {:.4} ETH", self.gas.fees_burned_eth)?;
        writeln!(f, "  Priority Fees: {:.4} ETH", self.gas.priority_fees_eth)?;
        
        writeln!(f, "\n{}", "TRANSACTIONS".green().bold())?;
        writeln!(f, "  Total: {}", self.transactions.total_count)?;
        writeln!(f, "  Failed: {}", self.transactions.failed_count)?;
        writeln!(f, "  Types: Legacy({}), EIP-2930({}), EIP-1559({}), EIP-4844({})",
            self.transactions.type_breakdown.legacy,
            self.transactions.type_breakdown.eip2930,
            self.transactions.type_breakdown.eip1559,
            self.transactions.type_breakdown.eip4844_blob,
        )?;
        
        writeln!(f, "\n{}", "MEV INDICATORS".green().bold())?;
        writeln!(f, "  Sandwich Attacks: {}", self.mev.sandwich_attacks.len())?;
        writeln!(f, "  Arbitrage Ops: {}", self.mev.arbitrage_ops.len())?;
        writeln!(f, "  Liquidations: {}", self.mev.liquidations)?;
        writeln!(f, "  Estimated MEV: {:.4} ETH", self.mev.estimated_mev_eth)?;
        
        writeln!(f, "\n{}", "PBS METRICS".green().bold())?;
        writeln!(f, "  PBS Block: {}", if self.pbs.is_pbs_block { "Yes" } else { "No" })?;
        if let Some(builder) = &self.pbs.builder_address {
            writeln!(f, "  Builder: {}", builder)?;
        }
        
        Ok(())
    }
}