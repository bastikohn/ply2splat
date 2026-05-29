//! Native Node.js bindings for ply2splat via NAPI-RS.
//!
//! This crate provides native Node.js bindings with full multi-threading support,
//! offering better performance than the WASM version for large files.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ply2splat::{convert as convert_bytes, splat_count_from_bytes};

/// Convert PLY data to SPLAT format.
///
/// @param plyData - PLY file contents as a Buffer
/// @param sort - Whether to sort splats by importance (default: true)
/// @returns Object containing the SPLAT data buffer and count
#[napi]
pub fn convert(ply_data: Buffer, sort: Option<bool>) -> Result<ConversionResult> {
    let sort = sort.unwrap_or(true);
    let (data, count) = convert_bytes(&ply_data, sort)
        .map_err(|e| Error::from_reason(format!("Failed to convert PLY data: {}", e)))?;

    Ok(ConversionResult {
        data: Buffer::from(data),
        count: count as u32,
    })
}

/// Result of a PLY to SPLAT conversion.
#[napi(object)]
pub struct ConversionResult {
    /// The converted SPLAT data
    pub data: Buffer,
    /// Number of splats in the result
    pub count: u32,
}

/// Get the number of splats in a SPLAT data buffer.
///
/// @param splatData - SPLAT file contents as a Buffer
/// @returns Number of splats in the data
#[napi]
pub fn get_splat_count(splat_data: Buffer) -> Result<u32> {
    splat_count_from_bytes(&splat_data)
        .map(|count| count as u32)
        .map_err(|e| Error::from_reason(e.to_string()))
}

/// Run the ply2splat CLI directly.
///
/// @param args - Array of command-line arguments (e.g. ["--input", "file.ply", "--output", "file.splat"])
#[napi]
pub fn cli(args: Vec<String>) -> Result<()> {
    // Prepend a dummy program name because clap expects the first arg to be the binary path
    let mut full_args = vec!["ply2splat".to_string()];
    full_args.extend(args);

    ply2splat::cli::run(full_args)
        .map_err(|e| Error::from_reason(format!("CLI execution failed: {}", e)))
}
