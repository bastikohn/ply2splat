# ply2splat-core

Core library for the `ply2splat` project.

This crate provides the logic for parsing Gaussian Splatting PLY files and converting them into the optimized SPLAT format.

## Features

- **Fast Parsing**: Uses `ply-rs` for robust PLY parsing.
- **Parallel Processing**: Leverages `rayon` for multi-threaded conversion and sorting.
- **Optimized Output**: Produces a dense, memory-efficient binary format.
- **Sorting**: Automatically sorts splats by importance and spatial position.

For more details, see the [main repository](https://github.com/bastikohn/ply2splat).
