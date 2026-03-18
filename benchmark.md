# Benchmarking Memory (Anvil vs forge-persist)

This runbook definitively bounds the architectural flaws within `anvil` regarding extensive long-term state persistence and visually charts why `forge-persist` (utilizing Reth's MDBX persistence layer) guarantees month-long testnet uptimes without crashing out of memory.

## The Telemetry Experiment

### 1. Test Anvil (The Memory Leak)
Open 3 separate terminals cleanly.

**Terminal A (Boot Anvil natively):**
```bash
anvil --port 4000
```

**Terminal B (Telemetry Loop):**
```bash
bash ./bin/monitor-mem.sh 4000
```

**Terminal C (Solidity Stress Flooder):**
```bash
forge script script/StressTest.s.sol:StressTest --rpc-url http://localhost:4000 --broadcast
```

*Observation Requirement:* Watch Terminal B's `benchmark.csv` terminal proxy. As the 1,000 independent transactions forcefully flood the Anvil mempool, its RSS memory will dynamically slope upwards continuously without GC relief, permanently retaining the mapping tree overhead indefinitely inside RAM.

---

### 2. Test forge-persist (The Solution)
Terminate the previous Anvil process safely. Delete your `.forge-persist` directory payload if demanding an isolated absolute fresh block test.

**Terminal A (Boot forge-persist):**
```bash
forge-persist --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY --port 4001
# If explicitly resuming an existing state map later:
# forge-persist --resume --port 4001
```

**Terminal B (Telemetry Loop):**
```bash
bash ./bin/monitor-mem.sh 4001
```

**Terminal C (Solidity Stress Flooder):**
```bash
forge script script/StressTest.s.sol:StressTest --rpc-url http://localhost:4001 --broadcast
```

*Observation Requirement:* Watch Terminal B precisely. Even as 1,000 structurally independent transactions bloat the node sequentially, the underlying `reth` binary asserts a perfectly flat Resident Set Size (RSS) profile limit, mechanically flushing state allocations flush to the underlying local MDBX database persistence disk natively.
