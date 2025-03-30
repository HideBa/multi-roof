use clap::{Parser, Subcommand};
use lodconv::{convert_lod, Result};
use std::path::PathBuf;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about = "Convert LoD2.2 building models to LoD1.2")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Convert a LoD2.2 model to LoD1.2
    Convert {
        /// Input OBJ file path (LoD2.2)
        #[arg(short, long)]
        input: PathBuf,

        /// Output OBJ file path (LoD1.2)
        #[arg(short, long)]
        output: PathBuf,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Convert {
            input,
            output,
            verbose,
        } => {
            if verbose {
                println!("Converting {} to {}", input.display(), output.display());
            }

            convert_lod(&input, &output)?;

            if verbose {
                println!("Conversion completed successfully!");
            }
        }
    }

    Ok(())
}
