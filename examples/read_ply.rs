use ply2splat::load_ply;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <input.ply>", args[0]);
        return Ok(());
    }

    let ply_data = load_ply(&args[1])?;
    println!("Loaded {} gaussians", ply_data.len());

    if let Some(first) = ply_data.first() {
        println!(
            "First gaussian pos: ({}, {}, {})",
            first.x, first.y, first.z
        );
    }

    Ok(())
}
