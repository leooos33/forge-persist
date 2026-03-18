#!/bin/bash
# forge-persist: Telemetry Monitor for RSS Memory Profiling

PORT=${1:-8545}

echo "Looking for designated active process bound to port $PORT..."

# Poka-Yoke: Use robust lsof rather than loose pgrep matching to avoid zombie metric pollution
PID=$(lsof -ti:$PORT | head -n 1)

if [ -z "$PID" ]; then
    echo "Error: No process is currently bound to port $PORT."
    echo "Please spin up anvil or forge-persist prior to tracking."
    exit 1
fi

NAME=$(ps -p $PID -o comm=)
echo "Tracking $NAME (PID: $PID) bound to port $PORT..."
echo "Time,Memory(MB)" | tee benchmark.csv

while true; do
    if ! kill -0 $PID 2>/dev/null; then
        echo "Process $PID ($NAME) has cleanly terminated."
        break
    fi

    # Extract raw Resident Set Size in KB and strictly convert mathematically to MB
    RSS_KB=$(ps -o rss= -p $PID 2>/dev/null | xargs)
    if [ -n "$RSS_KB" ]; then
        MB=$(echo "scale=2; $RSS_KB / 1024" | bc)
        TIMESTAMP=$(date '+%H:%M:%S')
        echo "$TIMESTAMP,$MB" | tee -a benchmark.csv
    fi

    sleep 1
done
