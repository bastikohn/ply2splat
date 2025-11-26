# ply2splat

[![Crates.io](https://img.shields.io/crates/v/ply2splat.svg)](https://crates.io/crates/ply2splat)
[![docs.rs](https://docs.rs/ply2splat/badge.svg)](https://docs.rs/ply2splat)
[![PyPI](https://img.shields.io/pypi/v/ply2splat.svg)](https://pypi.org/project/ply2splat/)
[![npm](https://img.shields.io/npm/v/ply2splat.svg)](https://www.npmjs.com/package/ply2splat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust crate and CLI tool for converting Gaussian Splatting `.ply` files to the `.splat` format.

Available on [crates.io](https://crates.io/crates/ply2splat) for Rust, [PyPI](https://pypi.org/project/ply2splat/) for Python, and [npm](https://www.npmjs.com/package/ply2splat) for JavaScript/TypeScript.

## Features

- **High Performance**: Utilizes parallel processing (via `rayon`) for conversion and sorting.
- **Fast I/O**: Uses zero-copy serialization and large buffers for maximum throughput.
- **Correctness**: Implements the standard conversion logic including Spherical Harmonics (SH) to color conversion and geometric transformations.
- **Python Bindings**: Use the library directly from Python via PyO3.
- **WebAssembly Support**: Run in browsers and Node.js via the npm package.

## Installation

### Rust Crate

Add `ply2splat` to your `Cargo.toml`:

```toml
[dependencies]
ply2splat = "0.1"
```

Or install via cargo:

```bash
cargo add ply2splat
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

### npm Package (WebAssembly)

Install from [npm](https://www.npmjs.com/package/ply2splat):

```bash
npm install ply2splat
```

## Usage

### CLI

```bash
ply2splat --input input.ply --output output.splat
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

```javascript
import init, { convert, parseSplatData, getSplatCount } from 'ply2splat';

// Initialize the WASM module
await init();

// Read PLY file (e.g., from a file input or fetch)
const plyData = new Uint8Array(await file.arrayBuffer());

// Convert PLY to SPLAT format
const result = convert(plyData, true);  // true = sort by importance
console.log(`Converted ${result.count} splats`);

// Get the raw SPLAT data as Uint8Array
const splatData = result.data;

// Parse SPLAT data into individual splat objects
const splats = parseSplatData(splatData);
for (const splat of splats) {
    console.log('Position:', splat.position);  // [x, y, z]
    console.log('Scale:', splat.scale);        // [sx, sy, sz]
    console.log('Color:', splat.color);        // [r, g, b, a]
    console.log('Rotation:', splat.rotation);  // [r0, r1, r2, r3]
}

// Get the count of splats from raw data
const count = getSplatCount(splatData);
console.log(`SPLAT data contains ${count} splats`);
```

#### TypeScript Types

The package includes TypeScript definitions. Key types:

```typescript
interface ConversionResult {
    readonly data: Uint8Array;  // Raw SPLAT binary data (32 bytes per splat)
    readonly count: number;     // Number of splats
}

// Functions
function convert(ply_data: Uint8Array, sort?: boolean | null): ConversionResult;
function parseSplatData(splat_data: Uint8Array): Array<any>;
function getSplatCount(splat_data: Uint8Array): number;
```

## Development

### Requirements

- Rust (latest stable)
- Nix (optional, for reproducible environment)
- wasm-pack (for WASM builds)

### Running Tests

```bash
cargo test
```

### Building WASM

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web (browsers)
wasm-pack build --target web --features wasm

# Build for bundlers (webpack, etc.)
wasm-pack build --target bundler --features wasm
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
