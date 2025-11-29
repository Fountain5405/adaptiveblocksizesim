# Adaptive Blocksize Simulator

A high-performance blockchain simulator with WebAssembly acceleration, demonstrating dynamic block size algorithms and fee market mechanics.

## Overview

This project simulates blockchain growth and fee dynamics under various traffic patterns, implementing the adaptive block size algorithm used in cryptocurrencies like Monero. It features both WebAssembly (Rust) and JavaScript implementations for performance comparison.

**⚠️ WARNING**: AI GENERATED. Original human code found here: https://github.com/spackle-xmr/Dynamic_Block_Demo/blob/main/Dynamic_Blocksize_econ_draft.py

## Features

- **Dual Engine Support**: WebAssembly (Rust) for high performance + JavaScript for compatibility
- **Real-time Visualization**: Interactive charts showing block size, mempool growth, fees, and penalties
- **Performance Comparison**: Side-by-side benchmarking of WASM vs JavaScript engines
- **Configurable Parameters**: Extensive tuning options for network constants and algorithm parameters
- **Multiple Traffic Patterns**: Linear, parabolic, exponential, and sine wave transaction influx patterns

## Prerequisites

### Common Requirements
- **Rust** (latest stable version)
- **Node.js** (optional, for alternative serving)
- **Python 3** (for built-in HTTP server)

### Platform-Specific Requirements

#### Linux
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Python 3 (if not already installed)
sudo apt update
sudo apt install python3 python3-pip

# Install Node.js (optional)
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt install nodejs
```

#### Windows
```powershell
# Install Rust
# Download and run rustup-init.exe from https://rustup.rs/

# Install Python 3
# Download from https://www.python.org/downloads/windows/

# Install Node.js (optional)
# Download from https://nodejs.org/

# Or use package managers:
# Using Chocolatey:
choco install rust python nodejs

# Using Scoop:
scoop install rust python nodejs
```

## Installation

### 1. Clone the Repository
```bash
git clone <repository-url>
cd adaptiveblocksizesim
```

### 2. Install wasm-pack (Required for WebAssembly compilation)
```bash
# Linux/macOS
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Windows (PowerShell)
Invoke-WebRequest https://rustwasm.github.io/wasm-pack/installer/init.ps1 -UseBasicParsing | Invoke-Expression
```

### 3. Verify Installation
```bash
rustc --version
wasm-pack --version
python3 --version
```

## Deployment Instructions

### Linux Deployment

#### Method 1: Using the Build Script (Recommended)
```bash
# Make the build script executable
chmod +x build.sh

# Build and run the application
./build.sh
```

The script will:
1. Build the WebAssembly module
2. Copy HTML files to the output directory
3. Start a Python HTTP server on port 8000
4. Open the application in your default browser

#### Method 2: Manual Build and Deploy
```bash
# Build the WebAssembly module
cd wasm-sim
wasm-pack build --target web --out-dir pkg

# Copy HTML file to the pkg directory
cp ../index.html pkg/

# Start HTTP server
cd pkg
python3 -m http.server 8000

# Alternative: Use Node.js serve
npx serve -s . -p 8000

# Alternative: Use any web server pointing to wasm-sim/pkg/
```

#### Method 3: Production Deployment with Nginx
```bash
# Build the application
./build.sh

# Install Nginx (if not installed)
sudo apt install nginx

# Configure Nginx
sudo tee /etc/nginx/sites-available/adaptiveblocksizesim > /dev/null <<EOF
server {
    listen 80;
    server_name your-domain.com;
    root /path/to/adaptiveblocksizesim/wasm-sim/pkg;
    index index.html;
    
    location / {
        try_files \$uri \$uri/ =404;
    }
    
    # Cache static assets
    location ~* \.(wasm|js|css)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
EOF

# Enable the site
sudo ln -s /etc/nginx/sites-available/adaptiveblocksizesim /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Windows Deployment

#### Method 1: Using PowerShell (Recommended)
```powershell
# Navigate to project directory
cd adaptiveblocksizesim

# Build the WebAssembly module
cd wasm-sim
wasm-pack build --target web --out-dir pkg

# Copy HTML file to the pkg directory
Copy-Item ..\index.html pkg\

# Start HTTP server
cd pkg
python -m http.server 8000

# Or use Python 3 explicitly
python3 -m http.server 8000
```

#### Method 2: Using Command Prompt
```cmd
# Navigate to project directory
cd adaptiveblocksizesim

# Build the WebAssembly module
cd wasm-sim
wasm-pack build --target web --out-dir pkg

# Copy HTML file
copy ..\index.html pkg\

# Start HTTP server
cd pkg
python -m http.server 8000
```

#### Method 3: Using Node.js
```powershell
# Install serve globally
npm install -g serve

# Build and serve
cd wasm-sim
wasm-pack build --target web --out-dir pkg
Copy-Item ..\index.html pkg\
cd pkg
serve -s . -p 8000
```

#### Method 4: Production Deployment with IIS
1. Build the application using Method 1 or 2
2. Install IIS (if not already installed)
3. Create a new website pointing to `wasm-sim/pkg`
4. Configure MIME types for `.wasm` files:
   - Extension: `.wasm`
   - MIME Type: `application/wasm`

```powershell
# Add WASM MIME type to IIS
Add-WebConfigurationProperty -Filter "system.webServer/staticContent/mimeMap" -Name "." -Value @{fileExtension=".wasm"; mimeType="application/wasm"} -PSPath "IIS:\Sites\Default Web Site"
```

## Testing the Deployment

### Automated Testing
```bash
# Linux/macOS
chmod +x test_webpage.sh
./test_webpage.sh

# Windows
# Open PowerShell and run:
cd wasm-sim/pkg
python -m http.server 8000
# Then open http://localhost:8000 in your browser
```

### Manual Testing
1. Open your browser and navigate to `http://localhost:8000`
2. Click "Run Simulation" to test the WebAssembly engine
3. Try different traffic patterns and parameters
4. Test the performance comparison mode

## Troubleshooting

### Common Issues

#### WebAssembly Module Fails to Load
- **Symptom**: "Failed to load WebAssembly module" error
- **Solution**: Ensure you're using a secure context (HTTPS or localhost)
- **Alternative**: Use `python -m http.server` which provides the correct headers

#### CORS Issues
- **Symptom**: Cross-origin request blocked
- **Solution**: Use a proper HTTP server instead of opening HTML files directly

#### Build Failures
- **Symptom**: wasm-pack build fails
- **Solution**:
  - Update Rust: `rustup update`
  - Clear cache: `cargo clean`
  - Reinstall wasm-pack

#### Performance Issues
- **Symptom**: Slow simulation performance
- **Solution**:
  - Enable "Fast Median" and "Simple Block Building" options
  - Use WebAssembly engine for better performance
  - Reduce the number of blocks in simulation

### Platform-Specific Issues

#### Linux
- **Permission Denied**: `chmod +x build.sh test_webpage.sh`
- **Python 3 Not Found**: Install with `sudo apt install python3`
- **Port 8000 in Use**: Use a different port: `python3 -m http.server 8080`

#### Windows
- **PowerShell Execution Policy**: `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser`
- **Python Not in PATH**: Add Python installation directory to PATH
- **Firewall Blocking**: Allow Python through Windows Firewall when prompted

## Development

### Building for Development
```bash
# Build with debug information
cd wasm-sim
wasm-pack build --target web --dev --out-dir pkg

# Watch for changes (requires additional setup)
cargo watch -x build
```

### Testing CLI Version
```bash
# Run the standalone CLI version
cd wasm-sim
cargo run --release
```

## Performance Optimization

### WebAssembly Optimization
- The release build is optimized for size and speed
- LTO (Link Time Optimization) is enabled
- Use `--release` flag for production builds

### Browser Optimization
- Modern browsers with WebAssembly support required
- Chrome/Edge: Best performance
- Firefox: Good performance
- Safari: Supported but may be slower

## License

This project maintains the same license as the original implementation.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly on both Linux and Windows
5. Submit a pull request
