//! Error types for ply2splat.
//!
//! This module provides a strictly typed error type for library consumers,
//! replacing the generic `anyhow::Error` with specific error variants.

use thiserror::Error;

/// Error type for ply2splat operations.
///
/// This provides specific error variants for different failure modes,
/// making it easier for consumers to handle errors programmatically.
#[derive(Error, Debug)]
pub enum PlyError {
    /// An I/O error occurred while reading or writing files.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse PLY file data.
    #[error("Failed to parse PLY file: {0}")]
    Parse(String),

    /// A required element is missing from the PLY file.
    #[error("Missing required element '{0}' in PLY file")]
    MissingElement(String),
}

/// A specialized `Result` type for ply2splat operations.
pub type PlyResult<T> = Result<T, PlyError>;
