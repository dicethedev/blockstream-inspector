use blockstream_inspector::rpc::{wei_to_eth, wei_to_gwei};
use ethers::{
    types::{U256},
};

#[test]
fn test_wei_to_eth_conversion() {
    // 1 ETH = 10^18 wei
    let one_eth = U256::from(1_000_000_000_000_000_000u64);
    let result = wei_to_eth(one_eth);
    assert!((result - 1.0).abs() < 0.001);

    // 0.5 ETH
    let half_eth = U256::from(500_000_000_000_000_000u64);
    let result = wei_to_eth(half_eth);
    assert!((result - 0.5).abs() < 0.001);

    // Zero
    let zero = U256::zero();
    let result = wei_to_eth(zero);
    assert_eq!(result, 0.0);
}

#[test]
fn test_wei_to_gwei_conversion() {
    // 1 gwei = 10^9 wei
    let one_gwei = U256::from(1_000_000_000u64);
    let result = wei_to_gwei(one_gwei);
    assert!((result - 1.0).abs() < 0.001);

    // 50 gwei
    let fifty_gwei = U256::from(50_000_000_000u64);
    let result = wei_to_gwei(fifty_gwei);
    assert!((result - 50.0).abs() < 0.001);

    // Zero
    let zero = U256::zero();
    let result = wei_to_gwei(zero);
    assert_eq!(result, 0.0);
}

#[test]
fn test_is_known_mev_bot() {
    let client_mock = MockRpcClient::new();

    // Known MEV bot
    assert!(client_mock.is_known_mev_bot("0x0000000000007f150bd6f54c40a34d7c3d5e9f56"));
    assert!(client_mock.is_known_mev_bot("0xa57bd00134b2850b2a1c55860c9e9ea100fdd6cf"));

    // Unknown address
    assert!(!client_mock.is_known_mev_bot("0x1234567890abcdef1234567890abcdef12345678"));

    // Case insensitive
    assert!(client_mock.is_known_mev_bot("0X0000000000007F150BD6F54C40A34D7C3D5E9F56"));
}

#[test]
fn test_large_wei_amounts() {
    // Test with very large amounts
    let large_amount = U256::from_dec_str("1000000000000000000000").unwrap(); // 1000 ETH
    let result = wei_to_eth(large_amount);
    assert!((result - 1000.0).abs() < 0.001);
}

// Mock client for testing MEV bot detection
struct MockRpcClient;

impl MockRpcClient {
    fn new() -> Self {
        MockRpcClient
    }

    fn is_known_mev_bot(&self, address: &str) -> bool {
        let known_bots = vec![
            "0x0000000000007f150bd6f54c40a34d7c3d5e9f56",
            "0xa57bd00134b2850b2a1c55860c9e9ea100fdd6cf",
            "0x00000000003b3cc22af3ae1eac0440bcee416b40",
        ];
        known_bots.contains(&address.to_lowercase().as_str())
    }
}
