use anyhow::{Context, Result};
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Block, Transaction, H256, U256},
};
use std::sync::Arc;

pub struct EthereumRpcClient {
    provider: Arc<Provider<Http>>,
}

impl EthereumRpcClient {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .context("Failed to create provider")?;
        
        // Test connection
        provider
            .get_block_number()
            .await
            .context("Failed to connect to Ethereum node")?;
        
        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    /// Fetch a block by number or latest
    pub async fn get_block(&self, block_id: &str) -> Result<Option<Block<Transaction>>> {
        let block = if block_id == "latest" {
            self.provider
                .get_block_with_txs(ethers::types::BlockNumber::Latest)
                .await
                .context("Failed to fetch latest block")?
        } else {
            let block_number: u64 = block_id.parse().context("Invalid block number")?;
            self.provider
                .get_block_with_txs(block_number)
                .await
                .context("Failed to fetch block")?
        };

        Ok(block)
    }

    #[allow(dead_code)]
    /// Fetch multiple blocks in a range
    pub async fn get_blocks_range(
        &self,
        start: u64,
        end: u64,
    ) -> Result<Vec<Block<Transaction>>> {
        let mut blocks = Vec::new();

        for block_num in start..=end {
            if let Some(block) = self
                .provider
                .get_block_with_txs(block_num)
                .await
                .context(format!("Failed to fetch block {}", block_num))?
            {
                blocks.push(block);
            }
        }

        Ok(blocks)
    }

    /// Get the latest block number
    pub async fn get_latest_block_number(&self) -> Result<u64> {
        Ok(self
            .provider
            .get_block_number()
            .await
            .context("Failed to get latest block number")?
            .as_u64())
    }

    #[allow(dead_code)]
    /// Get transaction receipt for detailed gas analysis
    pub async fn get_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> Result<Option<ethers::types::TransactionReceipt>> {
        Ok(self
            .provider
            .get_transaction_receipt(tx_hash)
            .await
            .context("Failed to fetch transaction receipt")?)
    }

    /// Get previous block for timing comparison
    pub async fn get_previous_block(&self, current: u64) -> Result<Option<Block<Transaction>>> {
        if current == 0 {
            return Ok(None);
        }

        Ok(self
            .provider
            .get_block_with_txs(current - 1)
            .await
            .context("Failed to fetch previous block")?)
    }

    /// Estimate if address is a known MEV bot
    pub fn is_known_mev_bot(&self, address: &str) -> bool {
        // Known MEV bot addresses (partial list for demonstration)
        let known_bots = vec![
            "0x0000000000007f150bd6f54c40a34d7c3d5e9f56", // MEV Bot
            "0xa57bd00134b2850b2a1c55860c9e9ea100fdd6cf", // MEV Bot
            "0x00000000003b3cc22af3ae1eac0440bcee416b40", // MEV Bot
            // Add more known addresses
        ];

        known_bots.contains(&address.to_lowercase().as_str())
    }
}

/// Helper function to convert U256 to f64 ETH
pub fn wei_to_eth(wei: U256) -> f64 {
    let eth_string = ethers::utils::format_units(wei, "ether").unwrap_or_else(|_| "0".to_string());
    eth_string.parse::<f64>().unwrap_or(0.0)
}

/// Helper function to convert U256 to gwei
pub fn wei_to_gwei(wei: U256) -> f64 {
    let gwei_string = ethers::utils::format_units(wei, "gwei").unwrap_or_else(|_| "0".to_string());
    gwei_string.parse::<f64>().unwrap_or(0.0)
}

