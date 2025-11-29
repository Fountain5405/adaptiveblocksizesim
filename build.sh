#!/bin/bash

# Adaptive Blocksize Simulator - WebAssembly Build Script
# This script builds the Rust WebAssembly module and serves the HTML

set -e  # Exit on error

echo "🔨 Building WebAssembly module..."

# Change to the wasm-sim directory
cd wasm-sim

# Build the WebAssembly module
wasm-pack build --target web --out-dir pkg

if [ $? -ne 0 ]; then
    echo "❌ WebAssembly build failed"
    exit 1
fi

echo "✅ WebAssembly module built successfully!"

# Copy the HTML file to the pkg directory for serving
cp ../index.html pkg/

echo "📁 HTML file copied to pkg directory"

# Check if Python simple HTTP server is available
if command -v python3 &> /dev/null; then
    echo "🌐 Starting Python HTTP server..."
    echo "📂 Open http://localhost:8000/pkg/ in your browser"
    echo "📂 WebAssembly module will be served from: http://localhost:8000/pkg/"
    
    # Change to pkg directory and start server
    cd pkg
    python3 -m http.server 8000
else
    echo "⚠️  Python3 not found. Please install Python3 or use a different server."
    echo "💡 You can serve the files manually with any HTTP server:"
    echo "   cd wasm-sim/pkg && python -m http.server 8000"
    echo "   or use Node.js: npx serve -s pkg"
    echo "   or use Live Server extension in VS Code"
fi