#!/bin/bash
set -x
lsof -ti:8546 | xargs kill -9 2>/dev/null || true
lsof -ti:4001 | xargs kill -9 2>/dev/null || true
pkill -f "forge-persist" 2>/dev/null || true
pkill -f reth 2>/dev/null || true
lsof -ti:4000 | xargs kill -9 2>/dev/null || true

echo "Booting temporary native anvil for the fork..."
anvil --port 4000 > /dev/null 2>&1 &
ANV_PID=$!
sleep 2

echo "Booting forge-persist on port 4001"
./target/release/forge-persist --fork-url http://localhost:4000 --port 4001 > persist_bench.log 2>&1 &
PERSIST_PID=$!

echo "Waiting for MDBX..."
for i in {1..30}; do
    if lsof -ti:4001 > /dev/null 2>&1; then
        echo "MDBX Bound!"
        break
    fi
    if ! kill -0 $PERSIST_PID 2>/dev/null; then
        echo "Forge Persist fatally bounded!"
        cat persist_bench.log
        exit 1
    fi
    sleep 2
done

echo "Attaching telemetry..."
./bin/monitor-persist.sh 4001 > /dev/null 2>&1 &
MON_PID=$!

echo "Flooding Reth..."
forge script script/StressTest.s.sol:StressTest --rpc-url http://localhost:4001 --broadcast > forge_persist_out.log 2>&1

echo "Teardown..."
kill $MON_PID 2>/dev/null || true
kill $PERSIST_PID 2>/dev/null || true
kill $ANV_PID 2>/dev/null || true
pkill -f "forge-persist" 2>/dev/null || true
pkill -f reth 2>/dev/null || true

echo "--- FINAL METRICS ---"
head -n 3 benchmark_persist.csv
echo "..."
tail -n 3 benchmark_persist.csv
