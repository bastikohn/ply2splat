//! Python bindings for ply2splat.
//!
//! This module exposes the core functionality of the ply2splat library to Python
//! via PyO3, allowing Python users to convert PLY files to SPLAT format.

use ply2splat_core::{load_ply, ply_to_splat, save_splat, SplatPoint};
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
