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

use anyhow::{Context, Result, bail};
use bytemuck::{Pod, Zeroable};
use ply_rs::parser::Parser;
use ply_rs::ply::{Property, PropertyAccess};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
use std::path::Path;

#[cfg(feature = "cli")]
pub mod cli;

const SH_C0: f32 = 0.282_094_8;
pub const SPLAT_POINT_BYTES: usize = 32;

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

#[derive(Debug, Clone, Default)]
struct RawPlyGaussian {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    f_dc_0: Option<f32>,
    f_dc_1: Option<f32>,
    f_dc_2: Option<f32>,
    opacity: Option<f32>,
    scale_0: Option<f32>,
    scale_1: Option<f32>,
    scale_2: Option<f32>,
    rot_0: Option<f32>,
    rot_1: Option<f32>,
    rot_2: Option<f32>,
    rot_3: Option<f32>,
    errors: Vec<String>,
}

impl RawPlyGaussian {
    fn set_float(&mut self, key: &str, property: Property) -> Option<f32> {
        match property {
            Property::Float(v) => Some(v),
            Property::Double(v) => Some(v as f32),
            other => {
                self.errors.push(format!(
                    "`{key}` has type {}, expected float",
                    property_type_name(&other)
                ));
                None
            }
        }
    }

    fn into_validated(self, vertex_index: usize) -> Result<PlyGaussian> {
        if !self.errors.is_empty() {
            bail!(
                "vertex {vertex_index} has invalid PLY properties: {}",
                self.errors.join(", ")
            );
        }

        Ok(PlyGaussian {
            x: required_property(self.x, "x", vertex_index)?,
            y: required_property(self.y, "y", vertex_index)?,
            z: required_property(self.z, "z", vertex_index)?,
            f_dc_0: required_property(self.f_dc_0, "f_dc_0", vertex_index)?,
            f_dc_1: required_property(self.f_dc_1, "f_dc_1", vertex_index)?,
            f_dc_2: required_property(self.f_dc_2, "f_dc_2", vertex_index)?,
            opacity: required_property(self.opacity, "opacity", vertex_index)?,
            scale_0: required_property(self.scale_0, "scale_0", vertex_index)?,
            scale_1: required_property(self.scale_1, "scale_1", vertex_index)?,
            scale_2: required_property(self.scale_2, "scale_2", vertex_index)?,
            rot_0: required_property(self.rot_0, "rot_0", vertex_index)?,
            rot_1: required_property(self.rot_1, "rot_1", vertex_index)?,
            rot_2: required_property(self.rot_2, "rot_2", vertex_index)?,
            rot_3: required_property(self.rot_3, "rot_3", vertex_index)?,
        })
    }
}

impl PropertyAccess for RawPlyGaussian {
    fn new() -> Self {
        Self::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match key.as_str() {
            "x" => self.x = self.set_float("x", property),
            "y" => self.y = self.set_float("y", property),
            "z" => self.z = self.set_float("z", property),
            "f_dc_0" => self.f_dc_0 = self.set_float("f_dc_0", property),
            "f_dc_1" => self.f_dc_1 = self.set_float("f_dc_1", property),
            "f_dc_2" => self.f_dc_2 = self.set_float("f_dc_2", property),
            "opacity" => self.opacity = self.set_float("opacity", property),
            "scale_0" => self.scale_0 = self.set_float("scale_0", property),
            "scale_1" => self.scale_1 = self.set_float("scale_1", property),
            "scale_2" => self.scale_2 = self.set_float("scale_2", property),
            "rot_0" => self.rot_0 = self.set_float("rot_0", property),
            "rot_1" => self.rot_1 = self.set_float("rot_1", property),
            "rot_2" => self.rot_2 = self.set_float("rot_2", property),
            "rot_3" => self.rot_3 = self.set_float("rot_3", property),
            _ => {}
        }
    }
}

fn required_property(value: Option<f32>, field: &'static str, vertex_index: usize) -> Result<f32> {
    value.with_context(|| format!("vertex {vertex_index} is missing required property `{field}`"))
}

fn property_type_name(property: &Property) -> &'static str {
    match property {
        Property::Char(_) => "char",
        Property::UChar(_) => "uchar",
        Property::Short(_) => "short",
        Property::UShort(_) => "ushort",
        Property::Int(_) => "int",
        Property::UInt(_) => "uint",
        Property::Float(_) => "float",
        Property::Double(_) => "double",
        Property::ListChar(_) => "list char",
        Property::ListUChar(_) => "list uchar",
        Property::ListShort(_) => "list short",
        Property::ListUShort(_) => "list ushort",
        Property::ListInt(_) => "list int",
        Property::ListUInt(_) => "list uint",
        Property::ListFloat(_) => "list float",
        Property::ListDouble(_) => "list double",
    }
}

fn validate_vertices(vertices: Vec<RawPlyGaussian>) -> Result<Vec<PlyGaussian>> {
    vertices
        .into_iter()
        .enumerate()
        .map(|(index, vertex)| vertex.into_validated(index))
        .collect()
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
pub fn load_ply_from_bytes(data: &[u8]) -> Result<Vec<PlyGaussian>> {
    let mut cursor = Cursor::new(data);
    let parser = Parser::<RawPlyGaussian>::new();
    let mut ply = parser
        .read_ply(&mut cursor)
        .context("Failed to parse PLY data")?;

    // Take ownership instead of cloning; vertex payloads can be hundreds of MB.
    let vertices = ply
        .payload
        .remove("vertex")
        .context("PLY data has no 'vertex' element")?;
    validate_vertices(vertices)
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
    let parser = Parser::<RawPlyGaussian>::new();
    let mut ply = parser
        .read_ply(&mut f)
        .context("Failed to parse PLY file")?;

    // Take ownership instead of cloning; vertex payloads can be hundreds of MB.
    let vertices = ply
        .payload
        .remove("vertex")
        .context("PLY file has no 'vertex' element")?;
    validate_vertices(vertices)
}

fn compare_splat_entries(a: &(SplatPoint, f32), b: &(SplatPoint, f32)) -> Ordering {
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
pub fn ply_to_splat(ply_points: Vec<PlyGaussian>, sort: bool) -> Vec<SplatPoint> {
    // Parallel convert to (SplatPoint, key)
    let mut data: Vec<(SplatPoint, f32)> = ply_points
        .into_par_iter()
        .map(|p| SplatPoint::from_ply(&p))
        .collect();

    if sort {
        // Parallel sort by key, tie-break by position (x, y, z)
        // This ensures deterministic output even across different platforms/architectures
        data.par_sort_by(compare_splat_entries);
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
pub fn ply_to_splat(ply_points: Vec<PlyGaussian>, sort: bool) -> Vec<SplatPoint> {
    // Single-threaded convert to (SplatPoint, key)
    let mut data: Vec<(SplatPoint, f32)> = ply_points
        .into_iter()
        .map(|p| SplatPoint::from_ply(&p))
        .collect();

    if sort {
        // Single-threaded sort by key, tie-break by position (x, y, z)
        // This ensures deterministic output even across different platforms/architectures
        data.sort_by(compare_splat_entries);
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
pub fn save_splat<P: AsRef<Path>>(path: P, splats: &[SplatPoint]) -> Result<()> {
    let mut f = File::create(path).context("Failed to create output file")?;

    // Zero-copy write: Cast the slice of structs directly to a slice of bytes.
    // SplatPoint is #[repr(C)] and Pod, so this is safe and extremely fast.
    let bytes: &[u8] = bytemuck::cast_slice(splats);
    f.write_all(bytes).context("Failed to write SPLAT data")?;

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

/// Returns the number of splats encoded in a SPLAT byte buffer.
pub fn splat_count_from_bytes(splat_data: &[u8]) -> Result<usize> {
    if !splat_data.len().is_multiple_of(SPLAT_POINT_BYTES) {
        bail!(
            "Invalid SPLAT data: size {} is not a multiple of {SPLAT_POINT_BYTES} bytes",
            splat_data.len()
        );
    }
    Ok(splat_data.len() / SPLAT_POINT_BYTES)
}

/// Converts raw SPLAT bytes into `SplatPoint`s after validating the byte length.
pub fn splats_from_bytes(splat_data: &[u8]) -> Result<Vec<SplatPoint>> {
    splat_count_from_bytes(splat_data)?;
    Ok(splat_data
        .chunks_exact(SPLAT_POINT_BYTES)
        .map(bytemuck::pod_read_unaligned)
        .collect())
}

/// Loads a SPLAT file and returns the decoded splat points.
pub fn load_splat<P: AsRef<Path>>(path: P) -> Result<Vec<SplatPoint>> {
    let mut f = File::open(path).context("Failed to open SPLAT file")?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)
        .context("Failed to read SPLAT file")?;
    splats_from_bytes(&bytes)
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
pub fn convert(ply_data: &[u8], sort: bool) -> Result<(Vec<u8>, usize)> {
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
pub fn convert_file<P: AsRef<Path>>(input: P, output: P, sort: bool) -> Result<usize> {
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
    fn test_load_ply_from_bytes_rejects_missing_required_property() {
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
end_header
1.0 2.0 3.0 0.5 0.5 0.5 0.0 0.1 0.1 0.1 1.0 0.0 0.0
";

        let error = load_ply_from_bytes(ply_content).expect_err("missing rot_3 should fail");
        assert!(
            error
                .to_string()
                .contains("missing required property `rot_3`")
        );
    }

    #[test]
    fn test_load_ply_from_bytes_rejects_invalid_required_property_type() {
        let ply_content = b"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
property float f_dc_0
property float f_dc_1
property float f_dc_2
property uchar opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
end_header
1.0 2.0 3.0 0.5 0.5 0.5 1 0.1 0.1 0.1 1.0 0.0 0.0 0.0
";

        let error = load_ply_from_bytes(ply_content).expect_err("uchar opacity should fail");
        assert!(error.to_string().contains("`opacity` has type uchar"));
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

        assert_eq!(bytes.len(), SPLAT_POINT_BYTES);

        let recovered: &[SplatPoint] = bytemuck::cast_slice(&bytes);
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].pos[0], 1.0);
        assert_eq!(recovered[0].color[0], 255);
    }

    #[test]
    fn test_splats_from_bytes_rejects_invalid_size() {
        let error = splats_from_bytes(&[0; 31]).expect_err("invalid SPLAT byte length should fail");
        assert!(error.to_string().contains("not a multiple"));
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
        assert_eq!(bytes.len(), 2 * SPLAT_POINT_BYTES);
    }
}
