//! # ply2splat
//!
//! `ply2splat` is a high-performance library and CLI tool for converting Gaussian Splatting PLY files
//! into a compact, optimized binary format suitable for real-time rendering.
//!
//! ## Features
//!
//! - **Fast Parsing**: Uses `ply-rs` for robust PLY parsing.
//! - **Parallel Processing**: Leverages `rayon` for multi-threaded conversion and sorting
//!   (when the `parallel` feature is enabled).
//! - **Optimized Output**: Produces a dense, memory-efficient binary format (32 bytes per splat).
//! - **Sorting**: Automatically sorts splats by importance (volume * opacity) and spatial position
//!   for deterministic rendering order.

pub mod error;
pub use error::{PlyError, PlyResult};

use bytemuck::{Pod, Zeroable};
use ply_rs::parser::Parser;
use ply_rs::ply::{Property, PropertyAccess};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufReader, Cursor, Write};
use std::path::Path;

#[cfg(feature = "cli")]
pub mod cli;

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

/// Loads PLY data from a byte slice and parses it into a vector of `PlyGaussian`.
///
/// This function is useful for WASM environments where file I/O is not available,
/// or when working with PLY data already in memory.
///
/// # Arguments
/// * `data` - A byte slice containing PLY file data.
///
/// # Returns
/// A `Result` containing the vector of parsed `PlyGaussian` structs or an error.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(data), fields(data_len = data.len())))]
pub fn load_ply_from_bytes(data: &[u8]) -> PlyResult<Vec<PlyGaussian>> {
    let mut cursor = Cursor::new(data);
    let parser = Parser::<PlyGaussian>::new();
    let ply = parser
        .read_ply(&mut cursor)
        .map_err(|e| PlyError::Parse(e.to_string()))?;

    let vertices = ply
        .payload
        .get("vertex")
        .ok_or_else(|| PlyError::MissingElement("vertex".to_string()))?;
    Ok(vertices.clone())
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
#[cfg_attr(feature = "tracing", tracing::instrument(fields(path = %path.as_ref().display())))]
pub fn load_ply<P: AsRef<Path>>(path: P) -> PlyResult<Vec<PlyGaussian>> {
    let f = File::open(path)?;
    let mut f = BufReader::with_capacity(10 * 1024 * 1024, f); // 10MB buffer
    let parser = Parser::<PlyGaussian>::new();
    let ply = parser
        .read_ply(&mut f)
        .map_err(|e| PlyError::Parse(e.to_string()))?;

    let vertices = ply
        .payload
        .get("vertex")
        .ok_or_else(|| PlyError::MissingElement("vertex".to_string()))?;
    Ok(vertices.clone())
}

/// Standard comparator for splats to ensure deterministic sorting.
///
/// Sorts by: sort key (ascending) -> Position X -> Position Y -> Position Z.
/// This ensures identical output across parallel and serial implementations,
/// and across different platforms/architectures.
#[inline]
pub fn compare_splats(a: &(SplatPoint, f32), b: &(SplatPoint, f32)) -> std::cmp::Ordering {
    a.1.total_cmp(&b.1)
        .then_with(|| a.0.pos[0].total_cmp(&b.0.pos[0]))
        .then_with(|| a.0.pos[1].total_cmp(&b.0.pos[1]))
        .then_with(|| a.0.pos[2].total_cmp(&b.0.pos[2]))
}

/// Converts a list of `PlyGaussian` structs into the optimized `SplatPoint` format.
///
/// This function performs the conversion in parallel using `rayon` (when the `parallel` feature is enabled).
/// It optionally sorts the splats based on a calculated key (volume * opacity) to optimize rendering order.
///
/// # Arguments
/// * `ply_points` - A vector of raw `PlyGaussian` data.
/// * `sort` - If true, sorts the splats by importance (volume * opacity).
///
/// # Returns
/// A vector of `SplatPoint` structs ready for saving/rendering.
#[cfg(feature = "parallel")]
#[cfg_attr(feature = "tracing", tracing::instrument(skip(ply_points), fields(vertex_count = ply_points.len(), sort, parallel = true)))]
pub fn ply_to_splat(ply_points: Vec<PlyGaussian>, sort: bool) -> Vec<SplatPoint> {
    // Parallel convert to (SplatPoint, key)
    let mut data: Vec<(SplatPoint, f32)> = ply_points
        .into_par_iter()
        .map(|p| SplatPoint::from_ply(&p))
        .collect();

    if sort {
        // Use shared comparator for deterministic sorting
        data.par_sort_by(compare_splats);
    }

    // Parallel strip key
    data.into_par_iter().map(|(s, _)| s).collect()
}

/// Converts a list of `PlyGaussian` structs into the optimized `SplatPoint` format.
///
/// This is a single-threaded version for environments where rayon is not available.
/// It optionally sorts the splats based on a calculated key (volume * opacity) to optimize rendering order.
///
/// # Arguments
/// * `ply_points` - A vector of raw `PlyGaussian` data.
/// * `sort` - If true, sorts the splats by importance (volume * opacity).
///
/// # Returns
/// A vector of `SplatPoint` structs ready for saving/rendering.
#[cfg(not(feature = "parallel"))]
#[cfg_attr(feature = "tracing", tracing::instrument(skip(ply_points), fields(vertex_count = ply_points.len(), sort, parallel = false)))]
pub fn ply_to_splat(ply_points: Vec<PlyGaussian>, sort: bool) -> Vec<SplatPoint> {
    // Single-threaded convert to (SplatPoint, key)
    let mut data: Vec<(SplatPoint, f32)> = ply_points
        .into_iter()
        .map(|p| SplatPoint::from_ply(&p))
        .collect();

    if sort {
        // Use shared comparator for deterministic sorting
        data.sort_by(compare_splats);
    }

    // Strip key
    data.into_iter().map(|(s, _)| s).collect()
}

/// Saves a slice of `SplatPoint`s to a file in a raw binary format.
///
/// The output file is a direct dump of the `SplatPoint` structs (32 bytes per point).
/// This format is efficient for loading directly into GPU buffers.
///
/// # Arguments
/// * `path` - Destination path.
/// * `splats` - The data to write.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(splats), fields(path = %path.as_ref().display(), splat_count = splats.len(), bytes = splats.len() * 32)))]
pub fn save_splat<P: AsRef<Path>>(path: P, splats: &[SplatPoint]) -> PlyResult<()> {
    let mut f = File::create(path)?;

    // Zero-copy write: Cast the slice of structs directly to a slice of bytes.
    // SplatPoint is #[repr(C)] and Pod, so this is safe and extremely fast.
    let bytes: &[u8] = bytemuck::cast_slice(splats);
    f.write_all(bytes)?;

    f.flush()?;
    Ok(())
}

/// Converts a slice of `SplatPoint`s to raw bytes.
///
/// This function returns a Vec<u8> containing the binary representation of the splats.
/// Each splat is exactly 32 bytes. This is useful for WASM environments where you
/// want to return the data to JavaScript.
///
/// # Arguments
/// * `splats` - The splat data to convert.
///
/// # Returns
/// A `Vec<u8>` containing the raw splat data.
pub fn splats_to_bytes(splats: &[SplatPoint]) -> Vec<u8> {
    bytemuck::cast_slice(splats).to_vec()
}

/// Converts PLY data bytes to SPLAT format bytes.
///
/// This is a convenience function that combines `load_ply_from_bytes`, `ply_to_splat`,
/// and `splats_to_bytes` into a single call.
///
/// # Arguments
/// * `ply_data` - A byte slice containing PLY file data.
/// * `sort` - If true, sorts the splats by importance (volume * opacity).
///
/// # Returns
/// A `Result` containing a tuple of (splat bytes, splat count) or an error.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(ply_data), fields(input_bytes = ply_data.len(), sort)))]
pub fn convert(ply_data: &[u8], sort: bool) -> PlyResult<(Vec<u8>, usize)> {
    let ply_points = load_ply_from_bytes(ply_data)?;
    let count = ply_points.len();
    let splats = ply_to_splat(ply_points, sort);
    let bytes = splats_to_bytes(&splats);
    Ok((bytes, count))
}

/// Converts a PLY file to a SPLAT file.
///
/// This is a convenience function that combines file loading, conversion, and saving.
///
/// # Arguments
/// * `input` - Path to the input PLY file.
/// * `output` - Path for the output SPLAT file.
/// * `sort` - If true, sorts the splats by importance (volume * opacity).
///
/// # Returns
/// A `Result` containing the number of splats converted or an error.
#[cfg_attr(feature = "tracing", tracing::instrument(fields(input = %input.as_ref().display(), output = %output.as_ref().display(), sort)))]
pub fn convert_file<P: AsRef<Path>>(input: P, output: P, sort: bool) -> PlyResult<usize> {
    let ply_data = load_ply(input)?;
    let count = ply_data.len();
    let splats = ply_to_splat(ply_data, sort);
    save_splat(output, &splats)?;
    Ok(count)
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

        // Sorting disabled for this logic test
        let splats = ply_to_splat(vec![p.clone()], false);
        let splat = splats[0];

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
        let splats = ply_to_splat(vec![p.clone()], false);
        assert_eq!(splats[0].color[3], 255);

        // Low opacity
        p.opacity = -100.0;
        let splats = ply_to_splat(vec![p.clone()], false);
        assert_eq!(splats[0].color[3], 0);
    }

    #[test]
    fn test_sorting_flag() {
        let p1 = PlyGaussian {
            x: 1.0,
            opacity: 0.0,
            ..Default::default()
        };
        let p2 = PlyGaussian {
            x: 0.0,
            opacity: 100.0,
            ..Default::default()
        };

        let input = vec![p1.clone(), p2.clone()];

        let sorted = ply_to_splat(input.clone(), true);
        assert_eq!(sorted[0].pos[0], 0.0); // p2
        assert_eq!(sorted[1].pos[0], 1.0); // p1

        let unsorted = ply_to_splat(input.clone(), false);
        assert_eq!(unsorted[0].pos[0], 1.0); // p1
        assert_eq!(unsorted[1].pos[0], 0.0); // p2
    }

    #[test]
    fn test_load_ply_from_bytes() {
        let ply_content = b"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property float f_dc_0
property float f_dc_1
property float f_dc_2
property float opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
end_header
1.0 2.0 3.0 0.5 0.5 0.5 0.0 0.1 0.1 0.1 1.0 0.0 0.0 0.0
";

        let result = load_ply_from_bytes(ply_content);
        assert!(result.is_ok());

        let gaussians = result.unwrap();
        assert_eq!(gaussians.len(), 1);
        assert_eq!(gaussians[0].x, 1.0);
        assert_eq!(gaussians[0].y, 2.0);
        assert_eq!(gaussians[0].z, 3.0);
    }

    #[test]
    fn test_splats_to_bytes() {
        let splat = SplatPoint {
            pos: [1.0, 2.0, 3.0],
            scale: [0.1, 0.2, 0.3],
            color: [255, 128, 64, 200],
            rot: [255, 128, 128, 128],
        };

        let bytes = splats_to_bytes(&[splat]);

        assert_eq!(bytes.len(), 32);

        let recovered: &[SplatPoint] = bytemuck::cast_slice(&bytes);
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].pos[0], 1.0);
        assert_eq!(recovered[0].color[0], 255);
    }

    #[test]
    fn test_convert() {
        let ply_content = b"ply
format ascii 1.0
element vertex 2
property float x
property float y
property float z
property float f_dc_0
property float f_dc_1
property float f_dc_2
property float opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
end_header
0.0 0.0 0.0 0.5 0.5 0.5 1.0 0.1 0.1 0.1 1.0 0.0 0.0 0.0
1.0 1.0 1.0 0.1 0.1 0.1 0.5 0.2 0.2 0.2 0.0 1.0 0.0 0.0
";

        let (bytes, count) = convert(ply_content, true).expect("Failed to convert");
        assert_eq!(count, 2);
        assert_eq!(bytes.len(), 64); // 2 splats * 32 bytes
    }

    #[test]
    fn test_compare_splats_ordering() {
        // Test that compare_splats orders by key first
        let splat1 = SplatPoint {
            pos: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [0, 0, 0, 0],
            rot: [128, 128, 128, 128],
        };
        let splat2 = SplatPoint {
            pos: [1.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [0, 0, 0, 0],
            rot: [128, 128, 128, 128],
        };

        // splat1 with lower key should come first
        let a = (splat1, -1.0_f32);
        let b = (splat2, 0.0_f32);
        assert_eq!(compare_splats(&a, &b), std::cmp::Ordering::Less);

        // splat2 with lower key should come first
        let a = (splat1, 1.0_f32);
        let b = (splat2, 0.0_f32);
        assert_eq!(compare_splats(&a, &b), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_compare_splats_tiebreak() {
        // Test that compare_splats breaks ties by position
        let splat1 = SplatPoint {
            pos: [0.0, 1.0, 2.0],
            scale: [1.0, 1.0, 1.0],
            color: [0, 0, 0, 0],
            rot: [128, 128, 128, 128],
        };
        let splat2 = SplatPoint {
            pos: [0.0, 1.0, 3.0],
            scale: [1.0, 1.0, 1.0],
            color: [0, 0, 0, 0],
            rot: [128, 128, 128, 128],
        };

        // Same key, splat1 has smaller z
        let a = (splat1, 0.0_f32);
        let b = (splat2, 0.0_f32);
        assert_eq!(compare_splats(&a, &b), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_ply_error_display() {
        // Test that error messages are human-readable
        let io_err = PlyError::Io(std::io::Error::other("test error"));
        assert!(io_err.to_string().contains("IO error"));

        let parse_err = PlyError::Parse("invalid format".to_string());
        assert!(parse_err.to_string().contains("Failed to parse PLY file"));

        let missing_err = PlyError::MissingElement("vertex".to_string());
        assert!(missing_err.to_string().contains("Missing required element"));
        assert!(missing_err.to_string().contains("vertex"));
    }

    #[test]
    fn test_load_ply_from_bytes_missing_vertex() {
        // PLY file without vertex element
        let ply_content = b"ply
format ascii 1.0
element face 1
property list uchar int vertex_indices
end_header
3 0 1 2
";
        let result = load_ply_from_bytes(ply_content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, PlyError::MissingElement(_)));
    }
}
