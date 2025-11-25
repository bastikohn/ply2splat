//! # ply2splat
//!
//! `ply2splat` is a high-performance library and CLI tool for converting Gaussian Splatting PLY files
//! into a compact, optimized binary format suitable for real-time rendering.
//!
//! ## Features
//!
//! - **Fast Parsing**: Uses `ply-rs` for robust PLY parsing.
//! - **Parallel Processing**: Leverages `rayon` for multi-threaded conversion and sorting.
//! - **Optimized Output**: Produces a dense, memory-efficient binary format (32 bytes per splat).
//! - **Sorting**: Automatically sorts splats by importance (volume * opacity) and spatial position for deterministic rendering order.

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use ply_rs::parser::Parser;
use ply_rs::ply::{Property, PropertyAccess};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const SH_C0: f32 = 0.282_094_8;

/// Represents a raw Gaussian Splat read from a PLY file.
///
/// This struct holds the properties directly as they appear in standard Gaussian Splatting PLY files.
/// - `x`, `y`, `z`: Position
/// - `f_dc_*`: Spherical Harmonics (DC component, representing color)
/// - `opacity`: Logit opacity (needs sigmoid)
/// - `scale_*`: Log-scale (needs exp)
/// - `rot_*`: Quaternion rotation (w, x, y, z order usually, but handled as raw floats here)
#[derive(Debug, Clone, Default)]
pub struct PlyGaussian {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub f_dc_0: f32,
    pub f_dc_1: f32,
    pub f_dc_2: f32,
    pub opacity: f32,
    pub scale_0: f32,
    pub scale_1: f32,
    pub scale_2: f32,
    pub rot_0: f32,
    pub rot_1: f32,
    pub rot_2: f32,
    pub rot_3: f32,
}

impl PropertyAccess for PlyGaussian {
    fn new() -> Self {
        Self::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_str(), property) {
            ("x", Property::Float(v)) => self.x = v,
            ("y", Property::Float(v)) => self.y = v,
            ("z", Property::Float(v)) => self.z = v,
            ("f_dc_0", Property::Float(v)) => self.f_dc_0 = v,
            ("f_dc_1", Property::Float(v)) => self.f_dc_1 = v,
            ("f_dc_2", Property::Float(v)) => self.f_dc_2 = v,
            ("opacity", Property::Float(v)) => self.opacity = v,
            ("scale_0", Property::Float(v)) => self.scale_0 = v,
            ("scale_1", Property::Float(v)) => self.scale_1 = v,
            ("scale_2", Property::Float(v)) => self.scale_2 = v,
            ("rot_0", Property::Float(v)) => self.rot_0 = v,
            ("rot_1", Property::Float(v)) => self.rot_1 = v,
            ("rot_2", Property::Float(v)) => self.rot_2 = v,
            ("rot_3", Property::Float(v)) => self.rot_3 = v,
            _ => {} // Ignore other properties
        }
    }
}

/// Represents a processed Gaussian Splat ready for serialization.
/// Layout is exactly 32 bytes packed: 3 floats, 3 floats, 4 u8, 4 u8.
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct SplatPoint {
    /// Position (x, y, z)
    pub pos: [f32; 3],
    /// Scale (x, y, z) - already exponentiated
    pub scale: [f32; 3],
    /// Color (R, G, B, A) - 8-bit quantization
    pub color: [u8; 4], // R, G, B, A
    /// Rotation (Quaternion) - 8-bit quantization mapping [-1, 1] to [0, 255]
    pub rot: [u8; 4],
}

impl SplatPoint {
    /// Converts a raw `PlyGaussian` into a `SplatPoint`.
    ///
    /// This process involves:
    /// 1. Converting SH DC components to RGB colors.
    /// 2. Applying the sigmoid activation to opacity.
    /// 3. Applying the exponential activation to scale.
    /// 4. Normalizing and quantizing the rotation quaternion.
    /// 5. Packing everything into the compact 32-byte format.
    ///
    /// Returns a tuple of `(SplatPoint, sort_key)`, where `sort_key` is used for sorting splats
    /// (usually by volume/opacity importance) to optimize rendering.
    pub fn from_ply(p: &PlyGaussian) -> (Self, f32) {
        // Color
        let r = ((0.5 + SH_C0 * p.f_dc_0).clamp(0.0, 1.0) * 255.0) as u8;
        let g = ((0.5 + SH_C0 * p.f_dc_1).clamp(0.0, 1.0) * 255.0) as u8;
        let b = ((0.5 + SH_C0 * p.f_dc_2).clamp(0.0, 1.0) * 255.0) as u8;

        // Opacity (Sigmoid)
        let opacity = (1.0 / (1.0 + (-p.opacity).exp())).clamp(0.0, 1.0);
        let a = (opacity * 255.0) as u8;

        // Scale (Exp)
        let s0 = p.scale_0.exp();
        let s1 = p.scale_1.exp();
        let s2 = p.scale_2.exp();

        // Rotation (Normalize -> Encode)
        let q_len =
            (p.rot_0 * p.rot_0 + p.rot_1 * p.rot_1 + p.rot_2 * p.rot_2 + p.rot_3 * p.rot_3).sqrt();
        let (r0, r1, r2, r3) = if q_len > 0.0 {
            (
                p.rot_0 / q_len,
                p.rot_1 / q_len,
                p.rot_2 / q_len,
                p.rot_3 / q_len,
            )
        } else {
            (1.0, 0.0, 0.0, 0.0)
        };

        let rot0 = (r0 * 128.0 + 128.0).clamp(0.0, 255.0) as u8;
        let rot1 = (r1 * 128.0 + 128.0).clamp(0.0, 255.0) as u8;
        let rot2 = (r2 * 128.0 + 128.0).clamp(0.0, 255.0) as u8;
        let rot3 = (r3 * 128.0 + 128.0).clamp(0.0, 255.0) as u8;

        let splat = SplatPoint {
            pos: [p.x, p.y, p.z],
            scale: [s0, s1, s2],
            color: [r, g, b, a],
            rot: [rot0, rot1, rot2, rot3],
        };

        // Calculate sort key: -volume * alpha
        // volume = exp(scale_sum)
        let volume = (p.scale_0 + p.scale_1 + p.scale_2).exp();
        let key = -(volume * opacity); // opacity is already calculated alpha

        (splat, key)
    }
}

/// Loads a PLY file and parses it into a vector of `PlyGaussian`.
///
/// This function uses `ply-rs` to parse the file. It specifically looks for the "vertex" element.
///
/// # Arguments
/// * `path` - Path to the .ply file.
///
/// # Returns
/// A `Result` containing the vector of parsed `PlyGaussian` structs or an error.
pub fn load_ply<P: AsRef<Path>>(path: P) -> Result<Vec<PlyGaussian>> {
    let f = File::open(path).context("Failed to open PLY file")?;
    let mut f = BufReader::with_capacity(10 * 1024 * 1024, f); // 10MB buffer
    let parser = Parser::<PlyGaussian>::new();
    let ply = parser
        .read_ply(&mut f)
        .context("Failed to parse PLY file")?;

    let vertices = ply
        .payload
        .get("vertex")
        .context("PLY file has no 'vertex' element")?;
    Ok(vertices.clone())
}

/// Converts a list of `PlyGaussian` structs into the optimized `SplatPoint` format.
///
/// This function performs the conversion in parallel using `rayon`.
/// When `sort` is true, it sorts the splats based on a calculated key (volume * opacity) to optimize rendering order.
///
/// # Arguments
/// * `ply_points` - A vector of raw `PlyGaussian` data.
/// * `sort` - Whether to sort the splats by importance (volume * opacity). Defaults to true for optimal rendering.
///
/// # Returns
/// A vector of `SplatPoint` structs ready for saving/rendering.
pub fn ply_to_splat(ply_points: Vec<PlyGaussian>, sort: bool) -> Vec<SplatPoint> {
    if sort {
        // Parallel convert to (SplatPoint, key)
        let mut data: Vec<(SplatPoint, f32)> = ply_points
            .into_par_iter()
            .map(|p| SplatPoint::from_ply(&p))
            .collect();

        // Parallel sort by key, tie-break by position (x, y, z)
        // This ensures deterministic output even across different platforms/architectures
        data.par_sort_by(|a, b| {
            a.1.total_cmp(&b.1)
                .then_with(|| a.0.pos[0].total_cmp(&b.0.pos[0]))
                .then_with(|| a.0.pos[1].total_cmp(&b.0.pos[1]))
                .then_with(|| a.0.pos[2].total_cmp(&b.0.pos[2]))
        });

        // Parallel strip key
        data.into_par_iter().map(|(s, _)| s).collect()
    } else {
        // Parallel convert without sorting - preserves original order
        ply_points
            .into_par_iter()
            .map(|p| SplatPoint::from_ply(&p).0)
            .collect()
    }
}

/// Saves a slice of `SplatPoint`s to a file in a raw binary format.
///
/// The output file is a direct dump of the `SplatPoint` structs (32 bytes per point).
/// This format is efficient for loading directly into GPU buffers.
///
/// # Arguments
/// * `path` - Destination path.
/// * `splats` - The data to write.
pub fn save_splat<P: AsRef<Path>>(path: P, splats: &[SplatPoint]) -> Result<()> {
    let mut f = File::create(path).context("Failed to create output file")?;

    // Zero-copy write: Cast the slice of structs directly to a slice of bytes.
    // SplatPoint is #[repr(C)] and Pod, so this is safe and extremely fast.
    let bytes: &[u8] = bytemuck::cast_slice(splats);
    f.write_all(bytes).context("Failed to write SPLAT data")?;

    f.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splat_conversion_logic() {
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

        let (splat, _) = SplatPoint::from_ply(&p);

        // Opacity 0.0 -> Sigmoid(0) = 0.5 -> 127 or 128
        assert!(splat.color[3] == 127 || splat.color[3] == 128);

        // Scale 0.0 -> Exp(0) = 1.0
        assert!((splat.scale[0] - 1.0).abs() < 1e-6);

        // Rotation (1, 0, 0, 0) -> (128+127, 128, 128, 128) approx
        // r0 = 1.0 -> 1.0 * 128 + 128 = 256 -> clamped to 255
        assert_eq!(splat.rot[0], 255);
        assert_eq!(splat.rot[1], 128);
        assert_eq!(splat.rot[2], 128);
        assert_eq!(splat.rot[3], 128);
    }

    #[test]
    fn test_opacity_extremes() {
        let mut p = PlyGaussian::default();

        // High opacity
        p.opacity = 100.0;
        let (splat, _) = SplatPoint::from_ply(&p);
        assert_eq!(splat.color[3], 255);

        // Low opacity
        p.opacity = -100.0;
        let (splat, _) = SplatPoint::from_ply(&p);
        assert_eq!(splat.color[3], 0);
    }

    #[test]
    fn test_ply_to_splat_no_sort_preserves_order() {
        // Create points with known positions
        let points = vec![
            PlyGaussian {
                x: 3.0,
                y: 0.0,
                z: 0.0,
                rot_0: 1.0,
                ..Default::default()
            },
            PlyGaussian {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                rot_0: 1.0,
                ..Default::default()
            },
            PlyGaussian {
                x: 2.0,
                y: 0.0,
                z: 0.0,
                rot_0: 1.0,
                ..Default::default()
            },
        ];

        // Without sorting, order should be preserved
        let splats = ply_to_splat(points.clone(), false);
        assert_eq!(splats[0].pos[0], 3.0);
        assert_eq!(splats[1].pos[0], 1.0);
        assert_eq!(splats[2].pos[0], 2.0);
    }

    #[test]
    fn test_ply_to_splat_with_sort() {
        // Create points with different importance values
        let points = vec![
            PlyGaussian {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                opacity: 10.0, // High opacity -> high importance
                scale_0: 1.0,
                scale_1: 1.0,
                scale_2: 1.0,
                rot_0: 1.0,
                ..Default::default()
            },
            PlyGaussian {
                x: 2.0,
                y: 0.0,
                z: 0.0,
                opacity: -10.0, // Low opacity -> low importance
                scale_0: 1.0,
                scale_1: 1.0,
                scale_2: 1.0,
                rot_0: 1.0,
                ..Default::default()
            },
        ];

        // With sorting, higher importance (more negative key) comes first
        let splats = ply_to_splat(points.clone(), true);
        // Point with x=1.0 has high opacity -> higher importance -> comes first
        assert_eq!(splats[0].pos[0], 1.0);
        assert_eq!(splats[1].pos[0], 2.0);
    }
}
