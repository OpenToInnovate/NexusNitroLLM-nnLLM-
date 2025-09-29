#!/bin/bash

# Persistent server runner for NexusNitroLLM
# Automatically restarts the server on crashes with a maximum retry limit

set -e

# Configuration
BINARY_PATH="./target/release/nnllm"
MAX_RESTARTS=50
RESTART_DELAY=2
LOG_FILE="server_persistent.log"
PID_FILE="server_persistent.pid"

# Store the PID of this script
echo $$ > "$PID_FILE"

# Cleanup function
cleanup() {
    echo "$(date): Received interrupt signal, cleaning up..." | tee -a "$LOG_FILE"
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "$(date): Killing server process $SERVER_PID" | tee -a "$LOG_FILE"
        kill "$SERVER_PID"
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    rm -f "$PID_FILE"
    exit 0
}

# Set up signal handling
trap cleanup SIGTERM SIGINT

# Load environment variables
if [ -f ".env" ]; then
    set -a  # automatically export all variables
    source .env
    set +a
    echo "$(date): Loaded environment from .env" | tee -a "$LOG_FILE"
else
    echo "$(date): Warning: .env file not found, using defaults" | tee -a "$LOG_FILE"
fi

# Validate binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "$(date): Error: Server binary not found at $BINARY_PATH" | tee -a "$LOG_FILE"
    echo "$(date): Run 'cargo build --release' to build the server first" | tee -a "$LOG_FILE"
    exit 1
fi

echo "$(date): Starting persistent server with maximum $MAX_RESTARTS restarts" | tee -a "$LOG_FILE"
echo "$(date): Server binary: $BINARY_PATH" | tee -a "$LOG_FILE"
echo "$(date): Port: ${PORT:-8080}" | tee -a "$LOG_FILE"
echo "$(date): Model: ${nnLLM_MODEL:-not set}" | tee -a "$LOG_FILE"

restart_count=0

while [ $restart_count -lt $MAX_RESTARTS ]; do
    echo "$(date): Starting server (attempt $((restart_count + 1))/$MAX_RESTARTS)" | tee -a "$LOG_FILE"

    # Start the server in background and capture its PID
    "$BINARY_PATH" --port "${PORT:-8080}" &
    SERVER_PID=$!

    echo "$(date): Server started with PID $SERVER_PID" | tee -a "$LOG_FILE"

    # Wait for the server process to complete
    wait "$SERVER_PID"
    exit_code=$?

    echo "$(date): Server process $SERVER_PID exited with code $exit_code" | tee -a "$LOG_FILE"

    # If exit code is 0, it was a clean shutdown (probably manual)
    if [ $exit_code -eq 0 ]; then
        echo "$(date): Server shut down cleanly, exiting persistent runner" | tee -a "$LOG_FILE"
        break
    fi

    # If we get a signal (like SIGTERM), exit gracefully
    if [ $exit_code -eq 130 ] || [ $exit_code -eq 143 ]; then
        echo "$(date): Server received interrupt signal, exiting persistent runner" | tee -a "$LOG_FILE"
        break
    fi

    restart_count=$((restart_count + 1))

    if [ $restart_count -lt $MAX_RESTARTS ]; then
        echo "$(date): Server crashed, restarting in ${RESTART_DELAY}s..." | tee -a "$LOG_FILE"
        sleep $RESTART_DELAY
    else
        echo "$(date): Maximum restart limit ($MAX_RESTARTS) reached, giving up" | tee -a "$LOG_FILE"
        echo "$(date): Check the logs for recurring issues" | tee -a "$LOG_FILE"
    fi
done

# Cleanup
rm -f "$PID_FILE"
echo "$(date): Persistent server runner exited" | tee -a "$LOG_FILE"