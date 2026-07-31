use anyhow::Result;
use ply2splat::cli;

fn main() -> Result<()> {
    if let Err(err) = cli::run(std::env::args()) {
        // Let clap render its own usage errors: colored output, single
        // "error:" prefix, and the conventional usage exit code 2.
        match err.downcast::<clap::Error>() {
            Ok(clap_err) => clap_err.exit(),
            Err(other) => return Err(other),
        }
    }
    Ok(())
}
