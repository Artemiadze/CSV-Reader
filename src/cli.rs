use std::path::PathBuf;
use clap::{Parser, Subcommand};

// Defining the command-line interface (clap).
// The subcommands:
//  - Head: show the first N lines
//  - Tail: show the last N lines
//  - Stats: calculate statistics by numeric columns
//  - Shape: show the number of rows/columns and headers
#[derive(Parser)]
#[command(name = "csvview")]
#[command(about = "Быстрый просмотр больших CSV-файлов в терминале")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}  

#[derive(Subcommand)]
pub enum Commands {
    Head { 
        file: PathBuf, 
        #[arg(short, long, default_value_t = 5)] 
        n: usize 
    },
    
    Tail { 
        file: PathBuf, 
        #[arg(short, long, default_value_t = 5)] 
        n: usize 
    },
    
    Stats { file: PathBuf },

    Shape { file: PathBuf },
}