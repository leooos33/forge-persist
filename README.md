# forge-persist

Persistent mainnet forks in one command. Free, self-hosted replacement for Anvil (leaks memory) and Tenderly Virtual Testnets ($450/mo).

## Installation

Install `forge-persist` via our automated cross-platform script:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leooos33/forge-persist/releases/latest/download/forge-persist-installer.sh | sh
```

*(For from-source installation methods, see the [Documentation](#).)*

## Usage

```bash
# Fork mainnet — that's it
forge-persist --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# Fork + deploy + persist (Phase 2 tested!)
forge-persist --fork-url $ETH_RPC_URL \
  --deploy "forge script script/Deploy.s.sol --broadcast --rpc-url http://localhost:8545"

# Resume later without re-fetching state
forge-persist --resume
```

## Comparisons

| Feature | Anvil | Tenderly | forge-persist |
|---------|-------|----------|---------------|
| Cost | Free | $450/mo | **Free** |
| Persistent | ✗ | ✓ | **✓** |
| Memory stable | ✗ (OOMs) | ✓ | **✓** (MDBX) |
| Latency | <1ms | 50-200ms | **<1ms** |

## License
MIT
