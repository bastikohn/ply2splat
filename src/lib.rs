//! # ply2splat
//!
//! `ply2splat` is a high-performance library and CLI tool for converting Gaussian Splatting PLY files
//! into a compact, optimized binary format suitable for real-time rendering.
//!
//! This crate re-exports all functionality from `ply2splat-core` for backward compatibility.
//! For new code, consider using `ply2splat-core` directly.
//!
//! ## Features
//!
//! - **Fast Parsing**: Uses `ply-rs` for robust PLY parsing.
//! - **Parallel Processing**: Leverages `rayon` for multi-threaded conversion and sorting
//!   (enabled by default via the `parallel` feature in `ply2splat-core`).
//! - **Optimized Output**: Produces a dense, memory-efficient binary format (32 bytes per splat).
//! - **Sorting**: Automatically sorts splats by importance (volume * opacity) and spatial position for deterministic rendering order.

// Re-export everything from the core crate
pub use ply2splat_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reexport_works() {
        let p = PlyGaussian {
            opacity: 0.0,
            scale_0: 0.0,
            scale_1: 0.0,
            scale_2: 0.0,
            rot_0: 1.0,
            rot_1: 0.0,
            rot_2: 0.0,
            rot_3: 0.0,
            f_dc_0: 0.0,
            f_dc_1: 0.0,
            f_dc_2: 0.0,
            ..Default::default()
        };

        let splats = ply_to_splat(vec![p], false);
        assert_eq!(splats.len(), 1);
    }
}
