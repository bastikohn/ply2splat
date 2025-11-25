# ply2splat

A Rust crate and CLI tool for converting Gaussian Splatting `.ply` files to the `.splat` format.

## Features

- **High Performance**: Utilizes parallel processing (via `rayon`) for conversion and sorting.
- **Fast I/O**: Uses zero-copy serialization and large buffers for maximum throughput.
- **Correctness**: Implements the standard conversion logic including Spherical Harmonics (SH) to color conversion and geometric transformations.
- **Python Bindings**: Use the library directly from Python via PyO3.

## Installation

### CLI (From Source)

```bash
git clone https://github.com/bastikohn/ply2splat.git
cd ply2splat
cargo build --release
```

The binary will be available at `target/release/ply2splat`.

### Python Package

Install from source using [maturin](https://github.com/PyO3/maturin):

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
