#!/bin/bash
set -e

# Build wasm template
wasm-pack build wasm --out-name search --release --target web

# Minify JavaScript
npx esbuild --bundle wasm/pkg/search.js --format=esm --minify --outfile=wasm/pkg/search.js --allow-overwrite

# Then build CLI
cargo build --release -p cli
