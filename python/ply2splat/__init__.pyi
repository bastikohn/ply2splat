"""
ply2splat - A high-performance library for converting Gaussian Splatting PLY files to SPLAT format.

This module provides Python bindings for the ply2splat Rust library, enabling fast
conversion of Gaussian Splatting PLY files to the compact SPLAT binary format.

Example:
    >>> import ply2splat
    >>> count = ply2splat.convert("input.ply", "output.splat")
    >>> print(f"Converted {count} splats")
"""

from typing import Tuple

def convert(input_path: str, output_path: str, sort: bool = True) -> int:
    """
    Convert a Gaussian Splatting PLY file to the compact SPLAT binary format.

    Args:
        input_path: Path to the input PLY file
        output_path: Path for the output SPLAT file
        sort: Whether to sort splats by importance (volume * opacity).
              Defaults to True. Sorting improves rendering quality but takes longer.

    Returns:
        The number of splats converted

    Raises:
        IOError: If the input file cannot be read or output file cannot be written

    Example:
        >>> count = ply2splat.convert("scene.ply", "scene.splat")
        >>> print(f"Converted {count} splats")
    """
    ...


def load_and_convert(input_path: str, sort: bool = True) -> Tuple[bytes, int]:
    """
    Load a PLY file and return splat data as bytes.

    This function loads a PLY file, converts it to SPLAT format, and returns
    the raw bytes. This is useful for further processing in Python without
    writing to disk.

    Args:
        input_path: Path to the input PLY file
        sort: Whether to sort splats by importance (volume * opacity).
              Defaults to True. Sorting improves rendering quality but takes longer.

    Returns:
        A tuple of (bytes, count) where bytes is the raw SPLAT data (32 bytes per splat)
        and count is the number of splats

    Raises:
        IOError: If the input file cannot be read

    Example:
        >>> data, count = ply2splat.load_and_convert("scene.ply")
        >>> print(f"Loaded {count} splats, {len(data)} bytes")
    """
    ...
