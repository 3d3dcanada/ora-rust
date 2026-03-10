#!/bin/bash
# OrA Run Script

set -e

cd "$(dirname "$0")"

# Build if needed
if [ ! -f "./target/release/ora" ]; then
    echo "🔨 Building OrA..."
    cargo build --release
fi

# Run
echo "🚀 Starting OrA..."
./target/release/ora "$@"
