# ply2splat

[![Crates.io](https://img.shields.io/crates/v/ply2splat.svg)](https://crates.io/crates/ply2splat)
[![docs.rs](https://docs.rs/ply2splat/badge.svg)](https://docs.rs/ply2splat)
[![PyPI](https://img.shields.io/pypi/v/ply2splat.svg)](https://pypi.org/project/ply2splat/)
[![npm](https://img.shields.io/npm/v/ply2splat.svg)](https://www.npmjs.com/package/ply2splat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust crate and CLI tool for converting Gaussian Splatting `.ply` files to the `.splat` format.

Available on [crates.io](https://crates.io/crates/ply2splat) for Rust, [PyPI](https://pypi.org/project/ply2splat/) for Python, and [npm](https://www.npmjs.com/package/ply2splat) for JavaScript/TypeScript.

## Workspace Architecture

This repository is organized as a Cargo workspace:

```
.                      # Core library (ply2splat) and CLI
├── bindings/
│   ├── ply2splat-wasm/    # WASM bindings for browser/Node.js
│   ├── ply2splat-napi/    # Native Node.js bindings via NAPI-RS
│   └── ply2splat-python/  # Python bindings via PyO3
└── packages/
    └── ply2splat/         # Unified npm package (WASM + optional native)
```

## Features

- **High Performance**: Utilizes parallel processing (via `rayon`) for conversion and sorting.
- **Fast I/O**: Uses zero-copy serialization and large buffers for maximum throughput.
- **Correctness**: Implements the standard conversion logic including Spherical Harmonics (SH) to color conversion and geometric transformations.
- **Python Bindings**: Use the library directly from Python via PyO3.
- **WebAssembly Support**: Run in browsers and Node.js via the npm package.
- **Native Node.js Bindings**: For maximum performance via NAPI-RS.

## Installation

### Rust Crate

Add `ply2splat` to your `Cargo.toml`:

```toml
[dependencies]
ply2splat = "0.2"
```

### CLI

Install the CLI tool directly from [crates.io](https://crates.io/crates/ply2splat):

```bash
cargo install ply2splat
```

Or build from source:

```bash
git clone https://github.com/bastikohn/ply2splat.git
cd ply2splat
cargo build --release
```

The binary will be available at `target/release/ply2splat`.

### Python Package

Install from [PyPI](https://pypi.org/project/ply2splat/):

```bash
pip install ply2splat
```

Or install from source using [maturin](https://github.com/PyO3/maturin):

```bash
pip install maturin
git clone https://github.com/bastikohn/ply2splat.git
cd ply2splat
maturin develop --release
```

Or build a wheel:

```bash
maturin build --release
pip install target/wheels/ply2splat-*.whl
```

### npm Package (Combined WASM + Native)

The `ply2splat` npm package provides a unified interface that seamlessly combines WebAssembly (WASM) for browser support and high-performance native bindings for Node.js.

Install from [npm](https://www.npmjs.com/package/ply2splat):

```bash
npm install ply2splat
```

When installed in a Node.js environment, it will attempt to download and use the native bindings (`@ply2splat/native`) for maximum performance and multi-threading. If the native bindings are unavailable or the platform is unsupported, it gracefully falls back to the WASM implementation.

## Usage

### CLI

#### Standard Installation (Rust)

```bash
ply2splat --input input.ply --output output.splat
```

#### Native CLI via Node.js

You can also run the high-performance Rust CLI directly via Node.js without installing Rust or compiling the binary manually. This uses the pre-compiled native bindings.

```bash
# Run once without installing
npx @ply2splat/native --input input.ply --output output.splat

# Or install globally
npm install -g @ply2splat/native
ply2splat --input input.ply
```

### Python

```python
import ply2splat

# Convert a PLY file to SPLAT format
count = ply2splat.convert("input.ply", "output.splat")
print(f"Converted {count} splats")

# Convert without sorting (faster, but may affect rendering quality)
count = ply2splat.convert("input.ply", "output.splat", sort=False)

# Load PLY file and access individual splats
data = ply2splat.load_ply_file("input.ply")
print(f"Loaded {len(data)} splats")

for splat in data:
    print(f"Position: {splat.position}")
    print(f"Scale: {splat.scale}")
    print(f"Color (RGBA): {splat.color}")
    print(f"Rotation: {splat.rotation}")

# Access splats by index
first_splat = data[0]
last_splat = data[-1]

# Load existing SPLAT file
data = ply2splat.load_splat_file("output.splat")
print(f"Loaded {len(data)} splats from SPLAT file")

# Get raw bytes for custom processing
raw_bytes = data.to_bytes()

# Load and convert to bytes (for in-memory processing)
data, count = ply2splat.load_and_convert("input.ply")
print(f"Loaded {count} splats, {len(data)} bytes")
```

### JavaScript/TypeScript (Browser/Node.js)

The npm package provides full TypeScript support with helper functions for working with various input types. It automatically selects the best backend (Native or WASM) for your environment.

#### Browser Usage

```typescript
import { init, convertFromFile, convertFromUrl, downloadSplat } from 'ply2splat';

// Initialize the WASM module
await init();

// Convert from a File input
const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
fileInput.addEventListener('change', async (e) => {
  const file = fileInput.files![0];
  const result = await convertFromFile(file);
  console.log(`Converted ${result.count} splats`);
  
  // Get typed Splat objects
  const splats = result.toSplats();
  console.log(splats[0].position);  // [x, y, z]
  
  // Download the result
  downloadSplat(result.data, 'output.splat');
});

// Convert from a URL
const result = await convertFromUrl('https://example.com/model.ply');
```

#### Node.js Usage

```typescript
import { init, convertFromBuffer, getBackend } from 'ply2splat';
import { readFileSync } from 'fs';

// Initialize (loads native bindings if available, otherwise WASM)
await init();

console.log(`Using backend: ${getBackend()}`); // 'native' or 'wasm'

// Convert from a Node.js Buffer
const plyBuffer = readFileSync('model.ply');
const result = convertFromBuffer(plyBuffer);
console.log(`Converted ${result.count} splats`);

// Get typed splat data
const splats = result.toSplats();
```

The package includes full TypeScript definitions. See the [API documentation](https://github.com/bastikohn/ply2splat/blob/main/packages/ply2splat/README.md) for detailed type information and all available helper functions.

## Development

### Requirements

- Rust (latest stable)
- Nix (optional, for reproducible environment)
- wasm-pack (for WASM builds)

### Running Tests

```bash
# Test the entire workspace
cargo test --workspace

# Test a specific crate
cargo test -p ply2splat
```

### Building WASM

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web (browsers)
wasm-pack build bindings/ply2splat-wasm --target web --out-dir ../../packages/ply2splat/wasm

# Build for bundlers (webpack, etc.)
wasm-pack build bindings/ply2splat-wasm --target bundler --out-dir ../../packages/ply2splat/wasm
```

### Building the npm Package

```bash
# Build WASM first
wasm-pack build bindings/ply2splat-wasm --target web --out-dir ../../packages/ply2splat/wasm

# Build TypeScript
cd packages/ply2splat
npm install
npm run build
```

### Fuzzing

The crate includes fuzzing targets to ensure stability against malformed inputs.

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzing target
cargo fuzz run fuzz_conversion
```

### Development Environment

This project supports both **Nix** and **Devcontainers** for a reproducible development environment.

- **Nix**: `nix develop` will enter a shell with Rust and dependencies configured.
- **Devcontainer**: Open the folder in VS Code and accept the prompt to reopen in container.

### License

MIT
