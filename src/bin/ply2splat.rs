use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use ply2splat::{load_ply, ply_to_splat, save_splat};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input PLY file
    #[arg(short, long)]
    input: PathBuf,

    /// Output SPLAT file
    #[arg(short, long)]
    output: PathBuf,

    /// Disable sorting of splats by importance (volume * opacity)
    #[arg(long)]
    no_sort: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let start_total = Instant::now();
    let sort = !args.no_sort;

    println!("Reading PLY file: {:?}", args.input);
    let start_read = Instant::now();
    let ply_data = load_ply(&args.input)?;
    let duration_read = start_read.elapsed();
    println!(
        "Loaded {} vertices in {:.2}s",
        ply_data.len(),
        duration_read.as_secs_f32()
    );

    if sort {
        println!("Processing and sorting...");
    } else {
        println!("Processing (sorting disabled)...");
    }
    let start_process = Instant::now();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("/|\\- ")
            .template("{spinner} {msg}")?,
    );
    pb.set_message("Converting...");

    let splats = ply_to_splat(ply_data, sort);

    pb.finish_with_message("Conversion complete");
    let duration_process = start_process.elapsed();
    println!("Processed in {:.2}s", duration_process.as_secs_f32());

    println!("Writing SPLAT file: {:?}", args.output);
    let start_write = Instant::now();
    save_splat(&args.output, &splats)?;
    let duration_write = start_write.elapsed();
    println!(
        "Written to {:?} in {:.2}s",
        args.output,
        duration_write.as_secs_f32()
    );

    println!("Total time: {:.2}s", start_total.elapsed().as_secs_f32());

    Ok(())
}
