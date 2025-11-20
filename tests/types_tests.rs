
use blockstream_inspector::types::*;

// Helper function to create a test block
fn create_test_block_lifecycle() -> BlockLifecycle {
    BlockLifecycle {
        block_number: 18000000,
        block_hash: "0x1234567890abcdef".to_string(),
        timestamp: 1698765432,
        proposer: "0xabcdef".to_string(),
        builder: Some("flashbots".to_string()),
        timing: TimingMetrics {
            block_time: 12.05,
            timestamp: 1698765432,
            propagation_delay: None,
        },
        gas: GasMetrics {
            gas_used: 29834521,
            gas_limit: 30000000,
            utilization: 99.45,
            base_fee_gwei: 25.34,
            avg_priority_fee_gwei: 1.52,
            fees_burned_eth: 0.7563,
            priority_fees_eth: 0.0453,
        },
        transactions: TransactionMetrics {
            total_count: 247,
            type_breakdown: TypeBreakdown {
                legacy: 12,
                eip2930: 5,
                eip1559: 225,
                eip4844_blob: 5,
            },
            ordering: OrderingMetrics {
                sorted_by_priority: false,
                anomalies: 3,
                avg_deviation: 0.5,
            },
            failed_count: 3,
        },
        mev: MevIndicators {
            sandwich_attacks: vec![],
            arbitrage_ops: vec![],
            liquidations: 2,
            estimated_mev_eth: 2.3451,
            mev_bot_addresses: vec!["0x123".to_string()],
        },
        pbs: PbsMetrics {
            is_pbs_block: true,
            builder_address: Some("flashbots".to_string()),
            builder_payment_eth: Some(0.05),
            extra_data: "flashbots".to_string(),
        },
    }
}

#[test]
fn test_block_lifecycle_creation() {
    let block = create_test_block_lifecycle();
    assert_eq!(block.block_number, 18000000);
    assert_eq!(block.block_hash, "0x1234567890abcdef");
    assert_eq!(block.timestamp, 1698765432);
}

#[test]
fn test_gas_metrics_utilization() {
    let block = create_test_block_lifecycle();
    assert!(block.gas.utilization > 99.0);
    assert!(block.gas.utilization < 100.0);
    assert_eq!(block.gas.gas_used, 29834521);
    assert_eq!(block.gas.gas_limit, 30000000);
}

#[test]
fn test_transaction_type_breakdown() {
    let block = create_test_block_lifecycle();
    let total = block.transactions.type_breakdown.legacy
        + block.transactions.type_breakdown.eip2930
        + block.transactions.type_breakdown.eip1559
        + block.transactions.type_breakdown.eip4844_blob;

    assert_eq!(total, 247);
    assert_eq!(block.transactions.total_count, 247);
}

#[test]
fn test_pbs_metrics() {
    let block = create_test_block_lifecycle();
    assert!(block.pbs.is_pbs_block);
    assert_eq!(block.pbs.builder_address, Some("flashbots".to_string()));
    assert_eq!(block.pbs.extra_data, "flashbots");
}

#[test]
fn test_mev_indicators() {
    let block = create_test_block_lifecycle();
    assert_eq!(block.mev.liquidations, 2);
    assert!(block.mev.estimated_mev_eth > 2.0);
    assert_eq!(block.mev.mev_bot_addresses.len(), 1);
}

#[test]
fn test_serialization() {
    let block = create_test_block_lifecycle();
    let json = serde_json::to_string(&block).unwrap();
    let deserialized: BlockLifecycle = serde_json::from_str(&json).unwrap();

    assert_eq!(block.block_number, deserialized.block_number);
    assert_eq!(block.gas.gas_used, deserialized.gas.gas_used);
}

#[test]
fn test_timing_metrics() {
    let timing = TimingMetrics {
        block_time: 12.5,
        timestamp: 1698765432,
        propagation_delay: Some(0.3),
    };

    assert_eq!(timing.block_time, 12.5);
    assert!(timing.propagation_delay.is_some());
    assert_eq!(timing.propagation_delay.unwrap(), 0.3);
}

#[test]
fn test_sandwich_attack() {
    let sandwich = SandwichAttack {
        frontrun_tx: "0xaaa".to_string(),
        victim_tx: "0xbbb".to_string(),
        backrun_tx: "0xccc".to_string(),
        estimated_profit_eth: 0.5,
        dex: "Uniswap".to_string(),
    };

    assert_eq!(sandwich.frontrun_tx, "0xaaa");
    assert_eq!(sandwich.dex, "Uniswap");
    assert!(sandwich.estimated_profit_eth > 0.0);
}

#[test]
fn test_arbitrage_op() {
    let arb = ArbitrageOp {
        tx_hash: "0x123".to_string(),
        path: vec!["ETH".to_string(), "USDC".to_string(), "ETH".to_string()],
        estimated_profit_eth: 0.2,
        dexes_involved: vec!["Uniswap".to_string(), "Sushiswap".to_string()],
    };

    assert_eq!(arb.path.len(), 3);
    assert_eq!(arb.dexes_involved.len(), 2);
    assert!(arb.estimated_profit_eth > 0.0);
}

#[test]
fn test_ordering_metrics() {
    let ordering = OrderingMetrics {
        sorted_by_priority: false,
        anomalies: 5,
        avg_deviation: 1.2,
    };

    assert!(!ordering.sorted_by_priority);
    assert_eq!(ordering.anomalies, 5);
}

#[test]
fn test_display_formatting() {
    let block = create_test_block_lifecycle();
    let display_str = format!("{}", block);

    // Check that key information is in the display output
    assert!(display_str.contains("18000000"));
    assert!(display_str.contains("TIMING METRICS"));
    assert!(display_str.contains("GAS METRICS"));
    assert!(display_str.contains("TRANSACTIONS"));
    assert!(display_str.contains("MEV INDICATORS"));
    assert!(display_str.contains("PBS METRICS"));
}
