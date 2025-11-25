use assert_cmd::Command;
use ply2splat::SplatPoint;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[test]
#[allow(deprecated)]
fn test_cli_conversion() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ply2splat"));

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

#[test]
#[allow(deprecated)]
fn test_cli_no_sort_flag() -> Result<(), Box<dyn std::error::Error>> {
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
    // Point 1 (lower importance)
    writeln!(
        ply_file,
        "0.0 0.0 0.0 0.5 0.5 0.5 -10.0 0.1 0.1 0.1 1.0 0.0 0.0 0.0"
    )?;
    // Point 2 (higher importance)
    writeln!(
        ply_file,
        "1.0 1.0 1.0 0.1 0.1 0.1 10.0 0.2 0.2 0.2 1.0 0.0 0.0 0.0"
    )?;

    let output_path_sorted = ply_file.path().with_extension("sorted.splat");
    let output_path_unsorted = ply_file.path().with_extension("unsorted.splat");

    // Run with sorting (default)
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ply2splat"));
    cmd.arg("--input")
        .arg(ply_file.path())
        .arg("--output")
        .arg(&output_path_sorted);
    cmd.assert().success();

    // Run without sorting
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ply2splat"));
    cmd.arg("--input")
        .arg(ply_file.path())
        .arg("--output")
        .arg(&output_path_unsorted)
        .arg("--no-sort");
    cmd.assert().success();

    // Verify outputs have correct size
    let sorted_content = std::fs::read(&output_path_sorted)?;
    let unsorted_content = std::fs::read(&output_path_unsorted)?;
    assert_eq!(sorted_content.len(), 64);
    assert_eq!(unsorted_content.len(), 64);

    // Outputs should differ since sorting changes order
    assert_ne!(sorted_content, unsorted_content);

    Ok(())
}

fn get_cache_dir() -> PathBuf {
    let cache_dir = PathBuf::from("test_cache");
    if !cache_dir.exists() {
        fs::create_dir(&cache_dir).expect("Failed to create cache dir");
    }
    cache_dir
}

fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[allow(deprecated)]
fn run_dataset_test(
    url: &str,
    expected_input_hash: &str,
    expected_output_hash: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir();
    let file_name = url.split('/').last().unwrap().split('?').next().unwrap();
    let cached_file_path = cache_dir.join(file_name);

    // 1. Check Cache
    let content = if cached_file_path.exists() {
        println!("Found cached file: {:?}", cached_file_path);
        let data = fs::read(&cached_file_path)?;
        let hash = calculate_hash(&data);
        if hash == expected_input_hash {
            data
        } else {
            println!(
                "Cached file hash mismatch (expected {}, got {}). Redownloading...",
                expected_input_hash, hash
            );
            download_and_cache(url, &cached_file_path)?
        }
    } else {
        println!("File not in cache. Downloading...");
        download_and_cache(url, &cached_file_path)?
    };

    // 2. Verify Input Hash (Sanity Check)
    let input_hash = calculate_hash(&content);
    assert_eq!(input_hash, expected_input_hash, "Input file hash mismatch");

    // 3. Run Conversion
    // Use a temp file for the actual test input to avoid modifying cache by accident
    let mut temp_ply = tempfile::NamedTempFile::new()?;
    temp_ply.write_all(&content)?;
    let output_path = temp_ply.path().with_extension("splat");

    println!("Running conversion...");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ply2splat"));
    cmd.arg("--input")
        .arg(temp_ply.path())
        .arg("--output")
        .arg(&output_path);

    cmd.assert().success();

    // 4. Verify Output Hash
    println!("Verifying output hash...");
    let output_content = std::fs::read(&output_path)?;
    let output_hash = calculate_hash(&output_content);
    println!("Output hash: {}", output_hash);

    assert_eq!(
        output_hash, expected_output_hash,
        "Output file hash mismatch"
    );

    Ok(())
}

fn download_and_cache(
    url: &str,
    cache_path: &PathBuf,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!("Downloading from: {}", url);
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download file: {}", response.status()).into());
    }
    let content = response.bytes()?.to_vec();
    fs::write(cache_path, &content)?;
    println!("Downloaded and cached to {:?}", cache_path);
    Ok(content)
}

#[test]
#[ignore = "reason: takes too long to download and hash the file"]
fn test_dataset_drjohnson() -> Result<(), Box<dyn std::error::Error>> {
    run_dataset_test(
        "https://huggingface.co/datasets/Voxel51/gaussian_splatting/resolve/main/FO_dataset/drjohnson/point_cloud/iteration_30000/point_cloud.ply?download=true",
        "92f4898839ec4ad7f197cf6c74b89918b35ea712b4e41435593ccb152d22b7f5",
        "1fa57e61226e54c0461de1535b77cd0c5264ec8c586e9ca2ff1ff6a5ab8fd2c2"
    )
}
