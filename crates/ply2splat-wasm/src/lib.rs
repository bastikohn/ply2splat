//! WebAssembly bindings for ply2splat.
//!
//! This crate exposes the core functionality of the ply2splat library to JavaScript
//! via wasm-bindgen, allowing browser and Node.js users to convert PLY data to SPLAT format.

use ply2splat::{SplatPoint, load_ply_from_bytes, ply_to_splat, splats_to_bytes};
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
    ///
    /// Note: This clones the data on each access. This is intentional for WASM interop
    /// since wasm-bindgen doesn't support returning references to internal data.
    /// For large files, consider caching the result on the JavaScript side.
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
    if !splat_data.len().is_multiple_of(32) {
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
    if !splat_data.len().is_multiple_of(32) {
        return Err(JsValue::from_str(&format!(
            "Invalid SPLAT data: size {} is not a multiple of 32 bytes",
            splat_data.len()
        )));
    }
    Ok(splat_data.len() / 32)
}
