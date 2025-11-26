//! # ply2splat
//!
//! `ply2splat` is a high-performance library and CLI tool for converting Gaussian Splatting PLY files
//! into a compact, optimized binary format suitable for real-time rendering.
//!
//! ## Features
//!
//! - **Fast Parsing**: Uses `ply-rs` for robust PLY parsing.
//! - **Parallel Processing**: Leverages `rayon` for multi-threaded conversion and sorting (native only).
//! - **Optimized Output**: Produces a dense, memory-efficient binary format (32 bytes per splat).
//! - **Sorting**: Automatically sorts splats by importance (volume * opacity) and spatial position for deterministic rendering order.
//! - **WASM Support**: Can be compiled to WebAssembly for use in browsers and Node.js.

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use ply_rs::parser::Parser;
use ply_rs::ply::{Property, PropertyAccess};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
use std::io::Cursor;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufReader, Write};
#[cfg(not(target_arch = "wasm32"))]
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
    let parser = Parser::<PlyGaussian>::new();
    let ply = parser
        .read_ply(&mut cursor)
        .context("Failed to parse PLY data")?;

    let vertices = ply
        .payload
        .get("vertex")
        .context("PLY data has no 'vertex' element")?;
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
#[cfg(not(target_arch = "wasm32"))]
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
/// This function performs the conversion in parallel using `rayon` (on native targets).
/// It optionally sorts the splats based on a calculated key (volume * opacity) to optimize rendering order.
///
/// # Arguments
/// * `ply_points` - A vector of raw `PlyGaussian` data.
/// * `sort` - If true, sorts the splats by importance (volume * opacity).
///
/// # Returns
/// A vector of `SplatPoint` structs ready for saving/rendering.
#[cfg(not(target_arch = "wasm32"))]
pub fn ply_to_splat(ply_points: Vec<PlyGaussian>, sort: bool) -> Vec<SplatPoint> {
    // Parallel convert to (SplatPoint, key)
    let mut data: Vec<(SplatPoint, f32)> = ply_points
        .into_par_iter()
        .map(|p| SplatPoint::from_ply(&p))
        .collect();

    if sort {
        // Parallel sort by key, tie-break by position (x, y, z)
        // This ensures deterministic output even across different platforms/architectures
        data.par_sort_by(|a, b| {
            a.1.total_cmp(&b.1)
                .then_with(|| a.0.pos[0].total_cmp(&b.0.pos[0]))
                .then_with(|| a.0.pos[1].total_cmp(&b.0.pos[1]))
                .then_with(|| a.0.pos[2].total_cmp(&b.0.pos[2]))
        });
    }

    // Parallel strip key
    data.into_par_iter().map(|(s, _)| s).collect()
}

/// Converts a list of `PlyGaussian` structs into the optimized `SplatPoint` format.
///
/// This is a single-threaded version for WASM targets where rayon is not available.
/// It optionally sorts the splats based on a calculated key (volume * opacity) to optimize rendering order.
///
/// # Arguments
/// * `ply_points` - A vector of raw `PlyGaussian` data.
/// * `sort` - If true, sorts the splats by importance (volume * opacity).
///
/// # Returns
/// A vector of `SplatPoint` structs ready for saving/rendering.
#[cfg(target_arch = "wasm32")]
pub fn ply_to_splat(ply_points: Vec<PlyGaussian>, sort: bool) -> Vec<SplatPoint> {
    // Single-threaded convert to (SplatPoint, key)
    let mut data: Vec<(SplatPoint, f32)> = ply_points
        .into_iter()
        .map(|p| SplatPoint::from_ply(&p))
        .collect();

    if sort {
        // Single-threaded sort by key, tie-break by position (x, y, z)
        // This ensures deterministic output even across different platforms/architectures
        data.sort_by(|a, b| {
            a.1.total_cmp(&b.1)
                .then_with(|| a.0.pos[0].total_cmp(&b.0.pos[0]))
                .then_with(|| a.0.pos[1].total_cmp(&b.0.pos[1]))
                .then_with(|| a.0.pos[2].total_cmp(&b.0.pos[2]))
        });
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
#[cfg(not(target_arch = "wasm32"))]
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
        }; // Should be first if sorted (low opacity/volume)
        let p2 = PlyGaussian {
            x: 0.0,
            opacity: 100.0,
            ..Default::default()
        }; // Should be last if sorted (high opacity/volume) -> Larger key (negative)

        let input = vec![p1.clone(), p2.clone()];

        // With sorting: p2 should come before p1 because:
        // Key = -(volume * opacity)
        // p1: volume=1, opacity=0.5 -> key = -0.5
        // p2: volume=1, opacity=1.0 -> key = -1.0
        // Sorted: -1.0 (p2) < -0.5 (p1) -> p2 then p1
        let sorted = ply_to_splat(input.clone(), true);
        assert_eq!(sorted[0].pos[0], 0.0); // p2
        assert_eq!(sorted[1].pos[0], 1.0); // p1

        // Without sorting: Order preserved (p1 then p2)
        let unsorted = ply_to_splat(input.clone(), false);
        assert_eq!(unsorted[0].pos[0], 1.0); // p1
        assert_eq!(unsorted[1].pos[0], 0.0); // p2
    }

    #[test]
    fn test_load_ply_from_bytes() {
        // Create a minimal PLY file in memory
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

        // Each splat is 32 bytes
        assert_eq!(bytes.len(), 32);

        // Verify round-trip
        let recovered: &[SplatPoint] = bytemuck::cast_slice(&bytes);
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].pos[0], 1.0);
        assert_eq!(recovered[0].color[0], 255);
    }

    #[test]
    fn test_full_conversion_from_bytes() {
        // Create a PLY file and convert it entirely from bytes
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

        let gaussians = load_ply_from_bytes(ply_content).expect("Failed to parse PLY");
        assert_eq!(gaussians.len(), 2);

        let splats = ply_to_splat(gaussians, true);
        assert_eq!(splats.len(), 2);

        let bytes = splats_to_bytes(&splats);
        assert_eq!(bytes.len(), 64); // 2 splats * 32 bytes
    }
}

/// Python bindings module for ply2splat.
///
/// This module exposes the core functionality of the ply2splat library to Python
/// via PyO3, allowing Python users to convert PLY files to SPLAT format.
#[cfg(feature = "python")]
mod python {
    use super::*;
    use pyo3::exceptions::PyIOError;
    use pyo3::prelude::*;
    use std::fs::File;
    use std::io::{BufReader, Read};

    /// A single Gaussian Splat with position, scale, color, and rotation.
    ///
    /// This class provides access to the individual properties of a splat
    /// in the compact 32-byte SPLAT format.
    #[pyclass]
    #[derive(Clone)]
    pub struct Splat {
        /// Position (x, y, z)
        #[pyo3(get)]
        pub position: (f32, f32, f32),
        /// Scale (x, y, z)
        #[pyo3(get)]
        pub scale: (f32, f32, f32),
        /// Color (R, G, B, A) as values 0-255
        #[pyo3(get)]
        pub color: (u8, u8, u8, u8),
        /// Rotation quaternion encoded as (r0, r1, r2, r3), values 0-255
        #[pyo3(get)]
        pub rotation: (u8, u8, u8, u8),
    }

    #[pymethods]
    impl Splat {
        fn __repr__(&self) -> String {
            format!(
                "Splat(position={:?}, scale={:?}, color={:?}, rotation={:?})",
                self.position, self.scale, self.color, self.rotation
            )
        }
    }

    impl From<&SplatPoint> for Splat {
        fn from(sp: &SplatPoint) -> Self {
            Splat {
                position: (sp.pos[0], sp.pos[1], sp.pos[2]),
                scale: (sp.scale[0], sp.scale[1], sp.scale[2]),
                color: (sp.color[0], sp.color[1], sp.color[2], sp.color[3]),
                rotation: (sp.rot[0], sp.rot[1], sp.rot[2], sp.rot[3]),
            }
        }
    }

    /// A collection of Gaussian Splats loaded from a file.
    ///
    /// This class provides list-like access to individual splats and supports
    /// iteration, indexing, and length queries.
    #[pyclass]
    pub struct SplatData {
        splats: Vec<SplatPoint>,
    }

    #[pymethods]
    impl SplatData {
        /// Get the number of splats.
        fn __len__(&self) -> usize {
            self.splats.len()
        }

        /// Get a splat by index.
        fn __getitem__(&self, index: isize) -> PyResult<Splat> {
            let len = self.splats.len() as isize;
            let idx = if index < 0 { len + index } else { index };
            if idx < 0 || idx >= len {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "index out of range",
                ));
            }
            Ok(Splat::from(&self.splats[idx as usize]))
        }

        /// Iterate over all splats.
        fn __iter__(slf: PyRef<'_, Self>) -> SplatIterator {
            SplatIterator {
                data: slf.into(),
                index: 0,
            }
        }

        /// Get all splats as a list.
        fn to_list(&self) -> Vec<Splat> {
            self.splats.iter().map(Splat::from).collect()
        }

        /// Get the raw bytes representation of all splats.
        fn to_bytes(&self) -> Vec<u8> {
            bytemuck::cast_slice(&self.splats).to_vec()
        }

        fn __repr__(&self) -> String {
            format!("SplatData({} splats)", self.splats.len())
        }
    }

    /// Iterator for SplatData.
    #[pyclass]
    pub struct SplatIterator {
        data: Py<SplatData>,
        index: usize,
    }

    #[pymethods]
    impl SplatIterator {
        fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Splat> {
            let py = slf.py();
            let data = slf.data.borrow(py);
            let current_index = slf.index;
            if current_index < data.splats.len() {
                let splat = Splat::from(&data.splats[current_index]);
                drop(data); // Release the borrow before mutating
                slf.index += 1;
                Some(splat)
            } else {
                None
            }
        }
    }

    /// Convert a Gaussian Splatting PLY file to the compact SPLAT binary format.
    ///
    /// Args:
    ///     input_path: Path to the input PLY file
    ///     output_path: Path for the output SPLAT file
    ///     sort: Whether to sort splats by importance (default: True)
    ///
    /// Returns:
    ///     The number of splats converted
    ///
    /// Raises:
    ///     IOError: If the input file cannot be read or output file cannot be written
    #[pyfunction]
    #[pyo3(signature = (input_path, output_path, sort=true))]
    fn convert(input_path: &str, output_path: &str, sort: bool) -> PyResult<usize> {
        let ply_data = load_ply(input_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
        let count = ply_data.len();
        let splats = ply_to_splat(ply_data, sort);
        save_splat(output_path, &splats).map_err(|e| PyIOError::new_err(e.to_string()))?;
        Ok(count)
    }

    /// Load a PLY file and return splat data as bytes.
    ///
    /// This function loads a PLY file, converts it to SPLAT format, and returns
    /// the raw bytes. This is useful for further processing in Python without
    /// writing to disk.
    ///
    /// Args:
    ///     input_path: Path to the input PLY file
    ///     sort: Whether to sort splats by importance (default: True)
    ///
    /// Returns:
    ///     A tuple of (bytes, count) where bytes is the raw SPLAT data and count
    ///     is the number of splats
    ///
    /// Raises:
    ///     IOError: If the input file cannot be read
    #[pyfunction]
    #[pyo3(signature = (input_path, sort=true))]
    fn load_and_convert(input_path: &str, sort: bool) -> PyResult<(Vec<u8>, usize)> {
        let ply_data = load_ply(input_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
        let count = ply_data.len();
        let splats = ply_to_splat(ply_data, sort);
        let bytes: Vec<u8> = bytemuck::cast_slice(&splats).to_vec();
        Ok((bytes, count))
    }

    /// Load a PLY file and return structured splat data.
    ///
    /// This function loads a PLY file, converts it to SPLAT format, and returns
    /// a SplatData object that provides access to individual splats.
    ///
    /// Args:
    ///     input_path: Path to the input PLY file
    ///     sort: Whether to sort splats by importance (default: True)
    ///
    /// Returns:
    ///     A SplatData object containing all splats
    ///
    /// Raises:
    ///     IOError: If the input file cannot be read
    #[pyfunction]
    #[pyo3(signature = (input_path, sort=true))]
    fn load_ply_file(input_path: &str, sort: bool) -> PyResult<SplatData> {
        let ply_data = load_ply(input_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
        let splats = ply_to_splat(ply_data, sort);
        Ok(SplatData { splats })
    }

    /// Load a SPLAT file and return structured splat data.
    ///
    /// This function loads a binary SPLAT file and returns a SplatData object
    /// that provides access to individual splats.
    ///
    /// Args:
    ///     input_path: Path to the input SPLAT file
    ///
    /// Returns:
    ///     A SplatData object containing all splats
    ///
    /// Raises:
    ///     IOError: If the input file cannot be read or has invalid format
    #[pyfunction]
    fn load_splat_file(input_path: &str) -> PyResult<SplatData> {
        let file = File::open(input_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        if bytes.len() % 32 != 0 {
            return Err(PyIOError::new_err(format!(
                "Invalid SPLAT file: size {} is not a multiple of 32 bytes",
                bytes.len()
            )));
        }

        let splats: Vec<SplatPoint> = bytemuck::cast_slice(&bytes).to_vec();
        Ok(SplatData { splats })
    }

    /// A ply2splat module for converting Gaussian Splatting PLY files to SPLAT format.
    #[pymodule]
    fn ply2splat(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<Splat>()?;
        m.add_class::<SplatData>()?;
        m.add_function(wrap_pyfunction!(convert, m)?)?;
        m.add_function(wrap_pyfunction!(load_and_convert, m)?)?;
        m.add_function(wrap_pyfunction!(load_ply_file, m)?)?;
        m.add_function(wrap_pyfunction!(load_splat_file, m)?)?;
        Ok(())
    }
}

/// WebAssembly bindings module for ply2splat.
///
/// This module exposes the core functionality of the ply2splat library to JavaScript
/// via wasm-bindgen, allowing browser and Node.js users to convert PLY data to SPLAT format.
#[cfg(feature = "wasm")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    /// Result of a PLY to SPLAT conversion.
    ///
    /// Contains the converted SPLAT data as raw bytes and the number of splats.
    #[wasm_bindgen]
    pub struct ConversionResult {
        data: Vec<u8>,
        count: usize,
    }

    #[wasm_bindgen]
    impl ConversionResult {
        /// Get the converted SPLAT data as a byte array.
        ///
        /// Each splat is exactly 32 bytes in the format:
        /// - Position: 3 x f32 (12 bytes)
        /// - Scale: 3 x f32 (12 bytes)
        /// - Color: 4 x u8 RGBA (4 bytes)
        /// - Rotation: 4 x u8 quaternion (4 bytes)
        #[wasm_bindgen(getter)]
        pub fn data(&self) -> Vec<u8> {
            self.data.clone()
        }

        /// Get the number of splats in the result.
        #[wasm_bindgen(getter)]
        pub fn count(&self) -> usize {
            self.count
        }
    }

    /// Convert PLY data (as bytes) to SPLAT format.
    ///
    /// This is the main entry point for WASM usage. Pass PLY file contents as a Uint8Array
    /// and receive the converted SPLAT data back.
    ///
    /// @param ply_data - PLY file contents as a Uint8Array
    /// @param sort - Whether to sort splats by importance (volume * opacity). Default: true
    /// @returns ConversionResult containing the SPLAT data and count
    /// @throws Error if the PLY data is invalid or cannot be parsed
    #[wasm_bindgen(js_name = convert)]
    pub fn wasm_convert(ply_data: &[u8], sort: Option<bool>) -> Result<ConversionResult, JsValue> {
        let sort = sort.unwrap_or(true);

        let ply_points = load_ply_from_bytes(ply_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse PLY data: {}", e)))?;

        let count = ply_points.len();
        let splats = ply_to_splat(ply_points, sort);
        let data = splats_to_bytes(&splats);

        Ok(ConversionResult { data, count })
    }

    /// Parse a SPLAT file from bytes and return individual splat data.
    ///
    /// This function takes raw SPLAT binary data and returns an array of splat objects
    /// that can be easily accessed from JavaScript.
    ///
    /// @param splat_data - SPLAT file contents as a Uint8Array (must be multiple of 32 bytes)
    /// @returns Array of Splat objects
    /// @throws Error if the data length is not a multiple of 32 bytes
    #[wasm_bindgen(js_name = parseSplatData)]
    pub fn parse_splat_data(splat_data: &[u8]) -> Result<js_sys::Array, JsValue> {
        if splat_data.len() % 32 != 0 {
            return Err(JsValue::from_str(&format!(
                "Invalid SPLAT data: size {} is not a multiple of 32 bytes",
                splat_data.len()
            )));
        }

        let splats: &[SplatPoint] = bytemuck::cast_slice(splat_data);
        let array = js_sys::Array::new_with_length(splats.len() as u32);

        for (i, splat) in splats.iter().enumerate() {
            let obj = js_sys::Object::new();

            // Position array
            let pos = js_sys::Array::new();
            pos.push(&JsValue::from_f64(splat.pos[0] as f64));
            pos.push(&JsValue::from_f64(splat.pos[1] as f64));
            pos.push(&JsValue::from_f64(splat.pos[2] as f64));
            js_sys::Reflect::set(&obj, &JsValue::from_str("position"), &pos)?;

            // Scale array
            let scale = js_sys::Array::new();
            scale.push(&JsValue::from_f64(splat.scale[0] as f64));
            scale.push(&JsValue::from_f64(splat.scale[1] as f64));
            scale.push(&JsValue::from_f64(splat.scale[2] as f64));
            js_sys::Reflect::set(&obj, &JsValue::from_str("scale"), &scale)?;

            // Color array (RGBA)
            let color = js_sys::Array::new();
            color.push(&JsValue::from_f64(splat.color[0] as f64));
            color.push(&JsValue::from_f64(splat.color[1] as f64));
            color.push(&JsValue::from_f64(splat.color[2] as f64));
            color.push(&JsValue::from_f64(splat.color[3] as f64));
            js_sys::Reflect::set(&obj, &JsValue::from_str("color"), &color)?;

            // Rotation array (quaternion)
            let rot = js_sys::Array::new();
            rot.push(&JsValue::from_f64(splat.rot[0] as f64));
            rot.push(&JsValue::from_f64(splat.rot[1] as f64));
            rot.push(&JsValue::from_f64(splat.rot[2] as f64));
            rot.push(&JsValue::from_f64(splat.rot[3] as f64));
            js_sys::Reflect::set(&obj, &JsValue::from_str("rotation"), &rot)?;

            array.set(i as u32, obj.into());
        }

        Ok(array)
    }

    /// Get the number of splats in a SPLAT data buffer.
    ///
    /// @param splat_data - SPLAT file contents as a Uint8Array
    /// @returns Number of splats in the data
    /// @throws Error if the data length is not a multiple of 32 bytes
    #[wasm_bindgen(js_name = getSplatCount)]
    pub fn get_splat_count(splat_data: &[u8]) -> Result<usize, JsValue> {
        if splat_data.len() % 32 != 0 {
            return Err(JsValue::from_str(&format!(
                "Invalid SPLAT data: size {} is not a multiple of 32 bytes",
                splat_data.len()
            )));
        }
        Ok(splat_data.len() / 32)
    }
}
