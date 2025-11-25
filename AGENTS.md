# AI Assistance Context for ply2splat

This project is a high-performance Rust crate for converting Gaussian Splatting `.ply` files (from the original paper) to the `.splat` binary format.

## Project Structure

*   `src/lib.rs`: Core library logic.
    *   `PlyGaussian`: Struct representing input PLY format (SH coefficients, scaling, rotation).
    *   `SplatPoint`: Zero-copy `#[repr(C)]` struct (32 bytes) for the output SPLAT format.
    *   `load_ply`: Reads PLY files using `ply-rs`.
    *   `ply_to_splat`: Parallel conversion and sorting logic using `rayon`.
    *   `save_splat`: Writes raw bytes to disk.
*   `src/bin/ply2splat.rs`: CLI entry point with progress bars (`indicatif`).
*   `tests/integration_test.rs`: CLI integration tests.
*   `fuzz/`: Fuzzing targets using `cargo-fuzz` / `libfuzzer`.
*   `.github/workflows/ci.yml`: CI pipeline (Test, Fuzz, Build, Release on Tag).
*   `flake.nix`: Nix development environment.

## Key Design Decisions

1.  **Performance**: 
    *   Uses `rayon` for parallel iteration and sorting.
    *   Uses `bytemuck` for zero-copy serialization of the `SplatPoint` struct.
    *   Buffered I/O (10MB buffer) for reading PLY files.
2.  **Correctness**:
    *   Opacity is calculated via Sigmoid function.
    *   Scale is calculated via Exponential function.
    *   Color is derived from Spherical Harmonics (SH) coefficients (specifically DC components).
    *   Points are sorted by a specific key: `-volume * alpha` (approximated).
3.  **Safety**:
    *   Uses `anyhow` for error handling.
    *   Fuzzing targets ensure the converter doesn't crash on malformed inputs.

## Development Workflow

*   **Build**: `cargo build --release`
*   **Test**: `cargo test`
*   **Fuzz**: `cargo fuzz run fuzz_conversion`
*   **Environment**: Use `nix develop` or the provided `.devcontainer`.

## Common Tasks

*   **Updating Logic**: If modifying `SplatPoint`, ensure it remains `#[repr(C)]` and 32-byte aligned to maintain compatibility with splat renderers.
*   **Adding Features**: When adding new CLI flags, update `src/bin/ply2splat.rs` using `clap`.
