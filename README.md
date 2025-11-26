# ply2splat

[![Crates.io](https://img.shields.io/crates/v/ply2splat.svg)](https://crates.io/crates/ply2splat)
[![docs.rs](https://docs.rs/ply2splat/badge.svg)](https://docs.rs/ply2splat)
[![PyPI](https://img.shields.io/pypi/v/ply2splat.svg)](https://pypi.org/project/ply2splat/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust crate and CLI tool for converting Gaussian Splatting `.ply` files to the `.splat` format.

Available on [crates.io](https://crates.io/crates/ply2splat) for Rust and [PyPI](https://pypi.org/project/ply2splat/) for Python.

## Features

- **High Performance**: Utilizes parallel processing (via `rayon`) for conversion and sorting.
- **Fast I/O**: Uses zero-copy serialization and large buffers for maximum throughput.
- **Correctness**: Implements the standard conversion logic including Spherical Harmonics (SH) to color conversion and geometric transformations.
- **Python Bindings**: Use the library directly from Python via PyO3.

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

## Development

### Requirements

- Rust (latest stable)
- Nix (optional, for reproducible environment)

### Running Tests

```bash
cargo test
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
