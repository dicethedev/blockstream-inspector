use anyhow::Result;
use csv::Writer;
use std::fs::File;

use crate::types::BlockLifecycle;

pub struct Exporter;

impl Exporter {
    /// Export block lifecycle data to CSV for Python analysis
    pub fn export_to_csv(results: &[BlockLifecycle], path: &str) -> Result<()> {
        let file = File::create(path)?;
        let mut wtr = Writer::from_writer(file);

        // Write header
        wtr.write_record(&[
            "block_number",
            "block_hash",
            "timestamp",
            "proposer",
            "builder",
            "block_time",
            "gas_used",
            "gas_limit",
            "gas_utilization",
            "base_fee_gwei",
            "avg_priority_fee_gwei",
            "fees_burned_eth",
            "priority_fees_eth",
            "tx_count",
            "tx_legacy",
            "tx_eip2930",
            "tx_eip1559",
            "tx_eip4844",
            "tx_failed",
            "tx_ordering_anomalies",
            "mev_sandwich_attacks",
            "mev_arbitrage_ops",
            "mev_liquidations",
            "mev_estimated_eth",
            "mev_bot_count",
            "is_pbs_block",
            "builder_address",
            "extra_data",
        ])?;

        // Write data
        for lifecycle in results {
            wtr.write_record(&[
                lifecycle.block_number.to_string(),
                lifecycle.block_hash.clone(),
                lifecycle.timestamp.to_string(),
                lifecycle.proposer.clone(),
                lifecycle.builder.clone().unwrap_or_else(|| "".to_string()),
                lifecycle.timing.block_time.to_string(),
                lifecycle.gas.gas_used.to_string(),
                lifecycle.gas.gas_limit.to_string(),
                lifecycle.gas.utilization.to_string(),
                lifecycle.gas.base_fee_gwei.to_string(),
                lifecycle.gas.avg_priority_fee_gwei.to_string(),
                lifecycle.gas.fees_burned_eth.to_string(),
                lifecycle.gas.priority_fees_eth.to_string(),
                lifecycle.transactions.total_count.to_string(),
                lifecycle.transactions.type_breakdown.legacy.to_string(),
                lifecycle.transactions.type_breakdown.eip2930.to_string(),
                lifecycle.transactions.type_breakdown.eip1559.to_string(),
                lifecycle.transactions.type_breakdown.eip4844_blob.to_string(),
                lifecycle.transactions.failed_count.to_string(),
                lifecycle.transactions.ordering.anomalies.to_string(),
                lifecycle.mev.sandwich_attacks.len().to_string(),
                lifecycle.mev.arbitrage_ops.len().to_string(),
                lifecycle.mev.liquidations.to_string(),
                lifecycle.mev.estimated_mev_eth.to_string(),
                lifecycle.mev.mev_bot_addresses.len().to_string(),
                lifecycle.pbs.is_pbs_block.to_string(),
                lifecycle.pbs.builder_address.clone().unwrap_or_else(|| "".to_string()),
                lifecycle.pbs.extra_data.clone(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }
}