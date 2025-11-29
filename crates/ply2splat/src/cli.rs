use crate::{load_ply, ply_to_splat, save_splat};
use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Input PLY file
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output SPLAT file
    #[arg(short, long)]
    pub output: PathBuf,

    /// Disable sorting of splats
    #[arg(long)]
    pub no_sort: bool,
}

/// Runs the CLI logic with the given arguments.
pub fn run<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args = CliArgs::parse_from(args);
    let start_total = Instant::now();

    println!("Reading PLY file: {:?}", args.input);
    let start_read = Instant::now();
    let ply_data = load_ply(&args.input)?;
    let duration_read = start_read.elapsed();
    println!(
        "Loaded {} vertices in {:.2}s",
        ply_data.len(),
        duration_read.as_secs_f32()
    );

    if args.no_sort {
        println!("Processing (sorting disabled)...");
    } else {
        println!("Processing and sorting...");
    }
    let start_process = Instant::now();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("/|\\- ")
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.set_message("Converting...");

    let splats = ply_to_splat(ply_data, !args.no_sort);

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
