#!/bin/bash

echo "🧪 Testing WebAssembly Blockchain Simulator"
echo "=========================================="

# Start server in background
cd wasm-sim/pkg
python3 -m http.server 8000 > /dev/null 2>&1 &
SERVER_PID=$!

echo "📡 Server started with PID: $SERVER_PID"
echo "🌐 Open http://localhost:8000/ in your browser"
echo ""
echo "📋 Test Instructions:"
echo "1. Open http://localhost:8000/ in browser"
echo "2. Click 'Run Simulation' button"
echo "3. Check if it runs quickly (should be <200ms now)"
echo "4. Try both WebAssembly and JavaScript engines"
echo "5. Test different parameters"
echo ""
echo "🛑 To stop server: kill $SERVER_PID"
echo ""

# Wait a bit for server to start
sleep 2

# Optional: Auto-open browser if available
if command -v xdg-open &> /dev/null; then
    xdg-open http://localhost:8000/
elif command -v open &> /dev/null; then
    open http://localhost:8000/
fi

# Keep script running
wait $SERVER_PID