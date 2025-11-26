"""
ply2splat - A high-performance library for converting Gaussian Splatting PLY files to SPLAT format.

This module provides Python bindings for the ply2splat Rust library, enabling fast
conversion of Gaussian Splatting PLY files to the compact SPLAT binary format.

Example:
    >>> import ply2splat
    >>> count = ply2splat.convert("input.ply", "output.splat")
    >>> print(f"Converted {count} splats")
    
    >>> # Load and access individual splats
    >>> data = ply2splat.load_ply_file("input.ply")
    >>> for splat in data:
    ...     print(splat.position, splat.color)
"""

from typing import Tuple, List, Iterator


class Splat:
    """A single Gaussian Splat with position, scale, color, and rotation.
    
    Attributes:
        position: A tuple (x, y, z) of float values representing the splat position.
        scale: A tuple (x, y, z) of float values representing the splat scale.
        color: A tuple (R, G, B, A) of integers 0-255 representing the splat color.
        rotation: A tuple (r0, r1, r2, r3) of integers 0-255 representing the encoded quaternion.
    """
    
    @property
    def position(self) -> Tuple[float, float, float]:
        """The position (x, y, z) of the splat."""
        ...
    
    @property
    def scale(self) -> Tuple[float, float, float]:
        """The scale (x, y, z) of the splat."""
        ...
    
    @property
    def color(self) -> Tuple[int, int, int, int]:
        """The color (R, G, B, A) of the splat, values 0-255."""
        ...
    
    @property
    def rotation(self) -> Tuple[int, int, int, int]:
        """The rotation quaternion (r0, r1, r2, r3) encoded as 0-255."""
        ...


class SplatData:
    """A collection of Gaussian Splats loaded from a file.
    
    This class provides list-like access to individual splats and supports
    iteration, indexing, and length queries.
    
    Example:
        >>> data = ply2splat.load_ply_file("scene.ply")
        >>> print(f"Loaded {len(data)} splats")
        >>> first_splat = data[0]
        >>> for splat in data:
        ...     print(splat.position)
    """
    
    def __len__(self) -> int:
        """Return the number of splats."""
        ...
    
    def __getitem__(self, index: int) -> Splat:
        """Get a splat by index. Supports negative indexing."""
        ...
    
    def __iter__(self) -> Iterator[Splat]:
        """Iterate over all splats."""
        ...
    
    def to_list(self) -> List[Splat]:
        """Get all splats as a list."""
        ...
    
    def to_bytes(self) -> bytes:
        """Get the raw bytes representation of all splats (32 bytes per splat)."""
        ...


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


def load_ply_file(input_path: str, sort: bool = True) -> SplatData:
    """
    Load a PLY file and return structured splat data.

    This function loads a PLY file, converts it to SPLAT format, and returns
    a SplatData object that provides access to individual splats.

    Args:
        input_path: Path to the input PLY file
        sort: Whether to sort splats by importance (volume * opacity).
              Defaults to True. Sorting improves rendering quality but takes longer.

    Returns:
        A SplatData object containing all splats

    Raises:
        IOError: If the input file cannot be read

    Example:
        >>> data = ply2splat.load_ply_file("scene.ply")
        >>> print(f"Loaded {len(data)} splats")
        >>> for splat in data:
        ...     print(splat.position, splat.color)
    """
    ...


def load_splat_file(input_path: str) -> SplatData:
    """
    Load a SPLAT file and return structured splat data.

    This function loads a binary SPLAT file and returns a SplatData object
    that provides access to individual splats.

    Args:
        input_path: Path to the input SPLAT file

    Returns:
        A SplatData object containing all splats

    Raises:
        IOError: If the input file cannot be read or has invalid format

    Example:
        >>> data = ply2splat.load_splat_file("scene.splat")
        >>> print(f"Loaded {len(data)} splats")
        >>> first_splat = data[0]
        >>> print(first_splat.position)
    """
    ...
