use assert_cmd::Command;
use ply2splat::SplatPoint;
use std::io::Write;

#[test]
fn test_cli_conversion() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ply2splat")?;

    // Create a dummy PLY file
    let mut ply_file = tempfile::NamedTempFile::new()?;
    writeln!(ply_file, "ply")?;
    writeln!(ply_file, "format ascii 1.0")?;
    writeln!(ply_file, "element vertex 2")?;
    writeln!(ply_file, "property float x")?;
    writeln!(ply_file, "property float y")?;
    writeln!(ply_file, "property float z")?;
    writeln!(ply_file, "property float f_dc_0")?;
    writeln!(ply_file, "property float f_dc_1")?;
    writeln!(ply_file, "property float f_dc_2")?;
    writeln!(ply_file, "property float opacity")?;
    writeln!(ply_file, "property float scale_0")?;
    writeln!(ply_file, "property float scale_1")?;
    writeln!(ply_file, "property float scale_2")?;
    writeln!(ply_file, "property float rot_0")?;
    writeln!(ply_file, "property float rot_1")?;
    writeln!(ply_file, "property float rot_2")?;
    writeln!(ply_file, "property float rot_3")?;
    writeln!(ply_file, "end_header")?;
    // Point 1
    writeln!(
        ply_file,
        "0.0 0.0 0.0 0.5 0.5 0.5 1.0 0.1 0.1 0.1 1.0 0.0 0.0 0.0"
    )?;
    // Point 2
    writeln!(
        ply_file,
        "1.0 1.0 1.0 0.1 0.1 0.1 0.5 0.2 0.2 0.2 0.0 1.0 0.0 0.0"
    )?;

    let output_path = ply_file.path().with_extension("splat");

    cmd.arg("--input")
        .arg(ply_file.path())
        .arg("--output")
        .arg(&output_path);

    cmd.assert().success();

    // Verify output exists and has correct size
    let content = std::fs::read(&output_path)?;
    // 2 points * 32 bytes = 64 bytes
    assert_eq!(content.len(), 64);

    Ok(())
}

#[test]
fn test_splat_struct_layout() {
    // Ensure the struct is exactly 32 bytes
    assert_eq!(std::mem::size_of::<SplatPoint>(), 32);
    assert_eq!(std::mem::align_of::<SplatPoint>(), 4);
}
