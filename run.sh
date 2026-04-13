#!/bin/bash
# OrA Run Script

set -e

cd "$(dirname "$0")"

if [ -f "../config/ora.env" ]; then
    # Load the product's default runtime environment when launched from this repo.
    set -a
    # shellcheck disable=SC1091
    source "../config/ora.env"
    set +a
fi

# Build if needed
if [ ! -f "./target/release/ora" ]; then
    echo "🔨 Building OrA..."
    cargo build --release
fi

# Run
echo "🚀 Starting OrA..."
./target/release/ora "$@"
