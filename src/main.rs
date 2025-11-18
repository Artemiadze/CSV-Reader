mod cli;
mod command;

use clap::Parser;
use std::error::Error;

use cli::{Cli, Commands};
use command::{head, tail, shape, stats};


// The application's entry point.
// Parse the CLI using clap and dispatch to the appropriate function for the selected subcommand.
// Return the Result to handle possible errors (IO, CSV parsing, etc.) correctly.
fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Head { file, n } => head(&file, n)?,
        Commands::Tail { file, n } => tail(&file, n)?,
        Commands::Stats { file } => stats(&file)?,
        Commands::Shape { file } => shape(&file)?,
    }

    Ok(())
}