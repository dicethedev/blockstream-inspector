#!/bin/bash
# Enhanced Example Usage Script for Blockstream Inspector
# Uses Alchemy RPC by default and supports colors, environment loading, OS detection, and better UX

set -e

###############################################
# Colors
###############################################
RED="\e[31m"
GREEN="\e[32m"
YELLOW="\e[33m"
BLUE="\e[34m"
RESET="\e[0m"

log_info()    { echo -e "${BLUE}[INFO]${RESET} $1"; }
log_success() { echo -e "${GREEN}[✓]${RESET} $1"; }
log_warn()    { echo -e "${YELLOW}[WARN]${RESET} $1"; }
log_error()   { echo -e "${RED}[ERROR]${RESET} $1"; }

###############################################
# Auto-load .env if exists
###############################################
if [ -f ".env" ]; then
    log_info "Loading environment variables from .env..."
    set -o allexport
    source .env
    set +o allexport
fi

###############################################
# Determine OS and correct binary name
###############################################
case "$(uname -s)" in
    Linux*)     OS="linux" ;;
    Darwin*)    OS="macos" ;;
    *)          OS="unknown" ;;
esac

###############################################
# Config
###############################################
# Use RPC from environment, falling back to Alchemy mainnet URL
RPC_URL=${ALCHEMY_RPC_URL:-${RPC_URL:-"https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY"}}
BLOCKSTREAM_INSPECTOR_BIN="./target/release/blockstream-inspector"

###############################################
# Binary Checks
###############################################
if [ ! -f "$BLOCKSTREAM_INSPECTOR_BIN" ]; then
    log_error "Binary not found at $BLOCKSTREAM_INSPECTOR_BIN"
    echo "Run:"
    echo "    cargo build --release"
    exit 1
else
    log_success "Binary found: $BLOCKSTREAM_INSPECTOR_BIN"
fi

log_info "Detected OS: $OS"
log_info "Using RPC endpoint: $RPC_URL"
echo ""

pause() {
    read -p "$(echo -e "${YELLOW}Press Enter to continue...${RESET}")"
    echo ""
}

echo -e "${BLUE}================================================${RESET}"
echo -e "${BLUE}Blockstream Inspector - Examples${RESET}"
echo -e "${BLUE}================================================${RESET}"
echo ""

###############################################
# Example 1: Analyze latest block
###############################################
log_info "Example 1: Analyzing latest block"
echo "--------------------------------------------"
$BLOCKSTREAM_INSPECTOR_BIN --rpc "$RPC_URL" block --number latest
pause

###############################################
# Example 2: Detailed analysis for block 18,000,000
###############################################
log_info "Example 2: Detailed analysis of block 18,000,000"
echo "--------------------------------------------------"
$BLOCKSTREAM_INSPECTOR_BIN --rpc "$RPC_URL" block --number 18000000 --verbose
pause

###############################################
# Example 3: Analyze range and export to CSV
###############################################
log_info "Example 3: Analyze 100 recent blocks & export CSV"
echo "--------------------------------------------------"

LATEST=$($BLOCKSTREAM_INSPECTOR_BIN --rpc "$RPC_URL" block --number latest 2>/dev/null | grep "Block Number" | awk '{print $3}')
START=$((LATEST - 100))
END=$LATEST

$BLOCKSTREAM_INSPECTOR_BIN --rpc "$RPC_URL" range \
    --start "$START" \
    --end "$END" \
    --output blocks_analysis.csv

log_success "CSV exported → blocks_analysis.csv"
pause

###############################################
# Example 4: Python analysis
###############################################
log_info "Example 4: Running Python analysis"
echo "--------------------------------------------"

if [ -f "blocks_analysis.csv" ]; then
    python3 scripts/analyze.py blocks_analysis.csv --all
    log_success "Python analysis complete! Generated charts:"
    echo "  - gas_analysis.png"
    echo "  - timing_analysis.png"
    echo "  - transaction_types.png"
    echo "  - mev_analysis.png"
else
    log_warn "blocks_analysis.csv not found — skipping Python analysis"
fi
pause

###############################################
# Example 5: Live block monitoring
###############################################
log_info "Example 5: Live monitoring (10 blocks)"
echo "--------------------------------------------"
$BLOCKSTREAM_INSPECTOR_BIN --rpc "$RPC_URL" live --count 10 --output live_blocks.csv
log_success "Live monitoring complete → live_blocks.csv"
pause

###############################################
# Example 6: MEV detection
###############################################
log_info "Example 6: MEV detection (recent 50 blocks)"
echo "--------------------------------------------------"
$BLOCKSTREAM_INSPECTOR_BIN --rpc "$RPC_URL" mev --blocks 50 --threshold 0.01
pause

echo -e "${GREEN}================================================${RESET}"
echo -e "${GREEN}Examples completed successfully!${RESET}"
echo -e "${GREEN}================================================${RESET}"
echo ""
echo "Next steps:"
echo "  1. Review CSV outputs"
echo "  2. Run Python analytics:"
echo "       python3 scripts/analyze.py <csv> --all"
echo "  3. Check generated PNG charts"
echo ""
