#!/bin/bash
set -e

echo "Building demo WASM files from static/documents.json..."

# Build the docfind CLI first if needed
if [ ! -f "target/release/docfind" ]; then
    echo "Building docfind CLI..."
    ./scripts/build.sh
fi

# Generate WASM files from documents.json
echo "Generating WASM files..."
./target/release/docfind static/documents.json static/

# Compress WASM with Brotli
echo "Compressing WASM with Brotli..."
brotli -k -f static/docfind_bg.wasm

echo "Demo build completed successfully!"
echo ""
echo "Generated files:"
ls -lh static/docfind.js static/docfind_bg.wasm static/docfind_bg.wasm.br
