# Quick Start Guide

## Prerequisites

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should show rust 1.70+
```

### 2. Install Python Dependencies
```bash
pip install -r requirements.txt
# Or with pip3
pip3 install pandas matplotlib seaborn numpy
```

### 3. Get RPC Access

**Option A: Free Hosted Node (Recommended for Quick Start)**
1. Sign up at https://www.alchemy.com/ or https://www.infura.io/
2. Create a new app
3. Copy your API endpoint URL
4. Export as environment variable:
```bash
export RPC_URL="https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY"
```

**Option B: Local Node (For Advanced Users)**
```bash
# Install Reth (Rust Ethereum client)
cargo install --git https://github.com/paradigmxyz/reth reth

# Run node (requires ~2TB storage)
reth node --http --http.addr 0.0.0.0
```

## Building BlockStream Inspector

### Development Build
```bash
cd blockstream-inspector
cargo build
./target/debug/blockstream-inspector --help
```

### Release Build (Optimized)
```bash
cargo build --release
./target/release/blockstream-inspector --help
```

### Run Tests
```bash
cargo test
```

## Quick Usage Examples

### 1. Analyze Latest Block
```bash
./target/release/blockstream-inspector --rpc $RPC_URL block --number latest
```

Expected output:
```
🔌 Connecting to Ethereum node at https://eth-mainnet...
✓ Connected successfully!

═══════════════════════════════════════════════════
Block Number: 18500123
Block Hash: 0x1234...
Timestamp: 1698765432
═══════════════════════════════════════════════════

⏱️  TIMING METRICS
  Block Time: 12.05s

⛽ GAS METRICS
  Gas Used: 29834521 / 30000000 (99.4%)
  Base Fee: 25.34 gwei
  ...
```

### 2. Analyze Specific Block with Details
```bash
./target/release/blockstream-inspector --rpc $RPC_URL block --number 18000000 --verbose
```

### 3. Analyze Block Range and Export
```bash
# Analyze 100 blocks
./target/release/blockstream-inspector --rpc $RPC_URL range \
    --start 18000000 \
    --end 18000100 \
    --output blocks.csv

# Output: blocks.csv created
```

### 4. Live Monitoring
```bash
# Monitor next 10 blocks
./target/release/blockstream-inspector --rpc $RPC_URL live --count 10 --output data/live.csv
```

### 5. MEV Detection
```bash
# Analyze last 100 blocks for MEV
./target/release/blockstream-inspector --rpc $RPC_URL mev --blocks 100 --threshold 0.1
```

### 6. Python Analysis
```bash
# Run comprehensive analysis
python3 scripts/analyze.py data/blocks.csv --all

# Run specific analyses
python3 scripts/analyze.py data/blocks.csv --gas     # Gas metrics only
python3 scripts/analyze.py data/blocks.csv --mev     # MEV analysis only
python3 scripts/analyze.py data/blocks.csv --timing  # Block timing only
```

## Troubleshooting

### Error: "Failed to connect to Ethereum node"
- Check RPC_URL is correct
- Verify internet connection
- Try a different RPC provider
- Check if you've hit rate limits

### Error: "cargo: command not found"
- Rust not installed or not in PATH
- Run: `source $HOME/.cargo/env`
- Or add to .bashrc/.zshrc

### Error: "Failed to fetch block"
- Block might not exist yet
- RPC provider might be down
- Try with "latest" instead of number

### Python Import Errors
```bash
# Install missing packages
pip3 install pandas matplotlib seaborn numpy

# Or use virtual environment
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### Rate Limiting
If you hit rate limits on free RPC providers:
- Use smaller block ranges
- Add delays between requests
- Upgrade to paid tier
- Run local node

## Performance Tips

### For Large Block Ranges
```bash
# Process in chunks
for i in {0..9}; do
    start=$((18000000 + i * 1000))
    end=$((18000000 + (i + 1) * 1000 - 1))
    ./target/release/blockstream-inspector --rpc $RPC_URL range \
        --start $start --end $end \
        --output blocks_chunk_$i.csv
done

# Combine CSV files
head -n 1 blocks_chunk_0.csv > all_blocks.csv
for f in blocks_chunk_*.csv; do
    tail -n +2 $f >> all_blocks.csv
done
```

### Parallel Processing
```bash
# Run multiple instances with different ranges
./target/release/blockstream-inspector --rpc $RPC_URL range --start 18000000 --end 18000500 --output p1.csv &
./target/release/blockstream-inspector --rpc $RPC_URL range --start 18000500 --end 18001000 --output p2.csv &
wait
```

## Common Workflows

### Daily MEV Monitoring

```bash
#!/bin/bash
# Get last 7200 blocks (approximately 24 hours)
LATEST=$(./target/release/blockstream-inspector --rpc block --number latest 2>/dev/null | grep "Block Number" | awk '{print $3}')
START=$((LATEST - 7200))

./target/release/blockstream-inspector --rpc range \
    --start $START \
    --end $LATEST \
    --output daily_$(date +%Y%m%d).csv

python3 scripts/analyze.py data/daily_$(date +%Y%m%d).csv --mev
```

### Historical Analysis
```bash
# Analyze specific period (e.g., post-Dencun upgrade)
./target/release/blockstream-inspector --rpc range \
    --start 19426587 \
    --end 19426687 \
    --output data/post_dencun.csv

python3 scripts/analyze.py data/post_dencun.csv --all
```

### Builder Competition Analysis
```bash
# Focus on PBS metrics
./target/release/blockstream-inspector --rpc range \
    --start 18000000 \
    --end 18001000 \
    --output data/builder_analysis.csv

python3 scripts/analyze.py data/builder_analysis.csv --pbs
```

## Next Steps

1. **Understand the Code**
   - Read `src/main.rs` - CLI entry point
   - Review `src/analyzer.rs` - Core analysis logic
   - Study `src/types.rs` - Data structures

2. **Experiment**
   - Try different block ranges
   - Modify detection heuristics
   - Add new metrics

3. **Contribute**
   - Add new MEV detection patterns
   - Improve visualization
   - Optimize performance

4. **Learn More**
   - Read `docs/TECHNICAL.md` for protocol deep-dive
   - Check Ethereum documentation

## Getting Help

- **Documentation**: Check `README.md` and `docs/`
- **Code Comments**: Read inline documentation
- **Ethereum Docs**: https://ethereum.org/developers
- **Rust Book**: https://doc.rust-lang.org/book/

## Useful Resources

- **Ethereum RPC Spec**: https://ethereum.org/en/developers/docs/apis/json-rpc/
- **Alloy Documentation**: https://github.com/alloy-rs/alloy
- **Ethers-rs Guide**: https://docs.rs/ethers/latest/ethers/
- **EIP-1559**: https://eips.ethereum.org/EIPS/eip-1559
- **EIP-4844**: https://eips.ethereum.org/EIPS/eip-4844

---

**Need help?** Open an issue on GitHub or refer to the documentation.

Happy analyzing! 🚀