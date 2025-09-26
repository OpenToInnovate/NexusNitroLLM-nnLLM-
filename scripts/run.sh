#!/usr/bin/env bash
set -euo pipefail
export $(grep -v '^#' .env 2>/dev/null | xargs -0 -I {} echo {} | tr '\n' ' ' || true)
RUST_LOG=${RUST_LOG:-info} cargo run --release