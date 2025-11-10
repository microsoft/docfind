# docfind

A high-performance document search engine built in Rust with WebAssembly support. Combines full-text search using FST (Finite State Transducers) with FSST compression for efficient storage and fast fuzzy matching capabilities.

## Features

- **Fast Fuzzy Search**: Uses FST for efficient keyword matching with Levenshtein distance support
- **Compact Storage**: FSST compression reduces index size while maintaining fast decompression
- **RAKE Keyword Extraction**: Automatic keyword extraction from document content using the RAKE algorithm
- **WebAssembly Ready**: Compile to WASM for browser-based search with no server required
- **Standalone CLI Tool**: Self-contained CLI tool to build a .wasm file out of a collection of documents, no Rust tooling required

## Usage

### Building the CLI

```bash
./build.sh
```

### Creating a Search Index

Prepare a JSON file with your documents:

```json
[
  {
    "title": "Getting Started",
    "category": "docs",
    "href": "/docs/getting-started",
    "body": "This guide will help you get started...",
    "keywords": ["tutorial", "introduction", "setup"]
  },
  ...
]
```

Build the index and generate a WASM module:

```bash
./target/release/cli documents.json output/
```

This creates:
- `output/search.js` - JavaScript bindings
- `output/search_bg.wasm` - WebAssembly module with embedded index

### Using in the Browser

```html
<script type="module">
  import init, { search } from './output/search.js';
  
  await init();
  
  const results = search('getting started', 10);
  const documents = JSON.parse(results);
  
  console.log(documents);
</script>
```

## How It Works

1. **Indexing Phase** (CLI):
   - Extracts keywords from document titles, categories, and bodies
   - Uses RAKE algorithm to identify important multi-word phrases
   - Assigns relevance scores based on keyword source (metadata > title > body)
   - Builds an FST mapping keywords to document indices
   - Compresses all document strings using FSST
   - Serializes the index using Postcard (binary format)

2. **Embedding Phase** (CLI):
   - Parses the pre-compiled WASM module
   - Expands WASM memory to accommodate the index
   - Patches global variables (`INDEX_BASE`, `INDEX_LEN`) with actual values
   - Adds the index as a new data segment in the WASM binary

3. **Search Phase** (WASM):
   - Deserializes the embedded index on first use
   - Performs fuzzy matching using Levenshtein automaton
   - Combines results from multiple keywords with score accumulation
   - Decompresses matching document strings on demand
   - Returns ranked results as JSON

## Dependencies

- **fst**: Fast finite state transducer library with Levenshtein support
- **fsst-rs**: Fast string compression for text data
- **rake**: Rapid Automatic Keyword Extraction algorithm
- **serde/postcard**: Efficient serialization
- **wasm-bindgen**: WebAssembly bindings for Rust
- **wasm-encoder/wasmparser**: WASM manipulation tools

## Performance

The combination of FST and FSST provides:
- Sub-millisecond search times for typical queries
- 60-80% compression ratio for document storage
- Instant startup with lazy index loading
- Zero network requests after initial load
