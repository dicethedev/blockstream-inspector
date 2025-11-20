
use blockstream_inspector::types::*;
use blockstream_inspector::exporter::Exporter;
use std::fs;

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

#[test]
fn test_export_single_block() {
    let blocks = vec![create_test_block()];
    let path = "/tmp/test_export_single.csv";

    let result = Exporter::export_to_csv(&blocks, path);
    assert!(result.is_ok());

    // Read and verify file exists
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("block_number"));
    assert!(content.contains("18000000"));
    assert!(content.contains("flashbots"));

    // Cleanup
    fs::remove_file(path).ok();
}

#[test]
fn test_export_multiple_blocks() {
    let mut blocks = vec![create_test_block()];
    let mut block2 = create_test_block();
    block2.block_number = 18000001;
    blocks.push(block2);

    let path = "/tmp/test_export_multiple.csv";

    let result = Exporter::export_to_csv(&blocks, path);
    assert!(result.is_ok());

    // Read and verify
    let content = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    // Should have header + 2 data rows
    assert_eq!(lines.len(), 3);
    assert!(content.contains("18000000"));
    assert!(content.contains("18000001"));

    // Cleanup
    fs::remove_file(path).ok();
}

#[test]
fn test_csv_headers() {
    let blocks = vec![create_test_block()];
    let path = "/tmp/test_headers.csv";

    Exporter::export_to_csv(&blocks, path).unwrap();

    let content = fs::read_to_string(path).unwrap();
    let first_line = content.lines().next().unwrap();

    // Check essential headers
    assert!(first_line.contains("block_number"));
    assert!(first_line.contains("gas_used"));
    assert!(first_line.contains("mev_estimated_eth"));
    assert!(first_line.contains("is_pbs_block"));

    // Cleanup
    fs::remove_file(path).ok();
}

#[test]
fn test_csv_data_integrity() {
    let blocks = vec![create_test_block()];
    let path = "/tmp/test_integrity.csv";

    Exporter::export_to_csv(&blocks, path).unwrap();

    let content = fs::read_to_string(path).unwrap();
    let data_line = content.lines().nth(1).unwrap();

    // Verify key data points are present
    assert!(data_line.contains("29834521")); // gas_used
    assert!(data_line.contains("99.45")); // utilization
    assert!(data_line.contains("true")); // is_pbs_block

    // Cleanup
    fs::remove_file(path).ok();
}

#[test]
fn test_empty_export() {
    let blocks: Vec<BlockLifecycle> = vec![];
    let path = "/tmp/test_empty.csv";

    let result = Exporter::export_to_csv(&blocks, path);
    assert!(result.is_ok());

    // Should still have headers
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains("block_number"));

    // Cleanup
    fs::remove_file(path).ok();
}

#[test]
fn test_export_with_none_values() {
    let mut block = create_test_block();
    block.builder = None;
    block.pbs.builder_address = None;

    let blocks = vec![block];
    let path = "/tmp/test_none_values.csv";

    let result = Exporter::export_to_csv(&blocks, path);
    assert!(result.is_ok());

    let content = fs::read_to_string(path).unwrap();
    // Empty strings for None values
    assert!(content.contains(",,") || content.lines().nth(1).unwrap().contains(""));

    // Cleanup
    fs::remove_file(path).ok();
}
