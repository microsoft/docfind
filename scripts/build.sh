#!/bin/bash
set -e

# Build wasm template
wasm-pack build wasm --out-name docfind --release --target web

# Minify JavaScript
npx --yes esbuild --bundle wasm/pkg/docfind.js --format=esm --minify --outfile=wasm/pkg/docfind.js --allow-overwrite

# Then build CLI
cargo build --release -p docfind
