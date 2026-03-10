#!/bin/bash
# OrA Build Script

set -e

echo "🔨 Building OrA..."

cd "$(dirname "$0")"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install it from https://rustup.rs/"
    exit 1
fi

# Build in release mode
echo "📦 Compiling..."
cargo build --release

echo "✅ Build complete!"
echo ""
echo "Run with: ./target/release/ora"
