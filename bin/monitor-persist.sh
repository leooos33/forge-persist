#!/bin/bash
PORT=${1:-4001}
PID=$(lsof -ti:$PORT | head -n 1)
if [ -z "$PID" ]; then exit 1; fi
echo "Time,Memory(MB)" > benchmark_persist.csv
while true; do
    if ! kill -0 $PID 2>/dev/null; then break; fi
    RSS_KB=$(ps -o rss= -p $PID 2>/dev/null | xargs)
    if [ -n "$RSS_KB" ]; then
        MB=$(echo "scale=2; $RSS_KB / 1024" | bc)
        TIMESTAMP=$(date '+%H:%M:%S')
        echo "$TIMESTAMP,$MB" >> benchmark_persist.csv
    fi
    sleep 1
done
