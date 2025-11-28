use anyhow::Result;
use ply2splat::cli;

fn main() -> Result<()> {
    cli::run(std::env::args())
}