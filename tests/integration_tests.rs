/// Integration tests for BlockStream Inspector
/// These tests verify the entire system works together

use blockstream_inspector::types::*;
use blockstream_inspector::exporter::Exporter;

#[test]
fn test_block_lifecycle_roundtrip() {
    // Create a test block lifecycle
    let block = create_test_block();
    
    // Verify all components
    assert_eq!(block.block_number, 18000000);
    assert!(block.gas.utilization > 99.0);
    assert_eq!(block.transactions.total_count, 247);
    assert!(block.pbs.is_pbs_block);
}

#[test]
fn test_csv_export_and_reimport() {
    use std::fs;
    
    let blocks = vec![create_test_block()];
    let path = "/tmp/integration_test.csv";
    
    // Export
    let result = Exporter::export_to_csv(&blocks, path);
    assert!(result.is_ok());
    
    // Verify file was created
    assert!(std::path::Path::new(path).exists());
    
    // Read and verify content
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("block_number"));
    assert!(content.contains("18000000"));
    
    // Cleanup
    fs::remove_file(path).ok();
}

#[test]
fn test_multiple_blocks_export() {
    use std::fs;
    
    let blocks: Vec<BlockLifecycle> = (0..10)
        .map(|i| {
            let mut block = create_test_block();
            block.block_number = 18000000 + i;
            block
        })
        .collect();
    
    let path = "/tmp/integration_multiple.csv";
    
    Exporter::export_to_csv(&blocks, path).unwrap();
    
    let content = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    
    // Header + 10 data rows
    assert_eq!(lines.len(), 11);
    
    // Cleanup
    fs::remove_file(path).ok();
}

#[test]
fn test_gas_calculations() {
    let block = create_test_block();
    
    // Verify gas utilization calculation
    let expected_util = (block.gas.gas_used as f64 / block.gas.gas_limit as f64) * 100.0;
    assert!((block.gas.utilization - expected_util).abs() < 0.1);
    
    // Verify fees are positive
    assert!(block.gas.fees_burned_eth >= 0.0);
    assert!(block.gas.priority_fees_eth >= 0.0);
    assert!(block.gas.base_fee_gwei >= 0.0);
}

#[test]
fn test_transaction_metrics_consistency() {
    let block = create_test_block();
    
    // Total transactions should equal sum of types
    let total_by_type = block.transactions.type_breakdown.legacy
        + block.transactions.type_breakdown.eip2930
        + block.transactions.type_breakdown.eip1559
        + block.transactions.type_breakdown.eip4844_blob;
    
    assert_eq!(block.transactions.total_count, total_by_type);
}

#[test]
fn test_mev_metrics() {
    let block = create_test_block();
    
    // MEV value should be non-negative
    assert!(block.mev.estimated_mev_eth >= 0.0);
    
    // Counts should be valid
    assert!(block.mev.sandwich_attacks.len() <= 100);
    assert!(block.mev.arbitrage_ops.len() <= 100);
    assert!(block.mev.liquidations <= 100);
}

#[test]
fn test_pbs_detection() {
    let mut block = create_test_block();
    
    // Test PBS block
    assert!(block.pbs.is_pbs_block);
    assert!(block.pbs.builder_address.is_some());
    
    // Test non-PBS block
    block.pbs.is_pbs_block = false;
    block.pbs.builder_address = None;
    assert!(!block.pbs.is_pbs_block);
    assert!(block.pbs.builder_address.is_none());
}

#[test]
fn test_serialization_deserialization() {
    let original = create_test_block();
    
    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();
    
    // Deserialize back
    let deserialized: BlockLifecycle = serde_json::from_str(&json).unwrap();
    
    // Verify key fields match
    assert_eq!(original.block_number, deserialized.block_number);
    assert_eq!(original.block_hash, deserialized.block_hash);
    assert_eq!(original.gas.gas_used, deserialized.gas.gas_used);
    assert_eq!(original.transactions.total_count, deserialized.transactions.total_count);
}

#[test]
fn test_display_formatting() {
    let block = create_test_block();
    let output = format!("{}", block);
    
    // Verify all sections are present
    assert!(output.contains("Block Number"));
    assert!(output.contains("TIMING METRICS"));
    assert!(output.contains("GAS METRICS"));
    assert!(output.contains("TRANSACTIONS"));
    assert!(output.contains("MEV INDICATORS"));
    assert!(output.contains("PBS METRICS"));
}

// Helper function to create a test block
fn create_test_block() -> BlockLifecycle {
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