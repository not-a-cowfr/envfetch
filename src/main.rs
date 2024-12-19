//! Lightweight cross-platform CLI tool for working with environment variables
//!
//! Easily get value of environment variable, get list of all variables,
//! run processes with specific variable value, or delete specific variable
//! to run process without it

mod utils;
mod models;
mod commands;

use clap::Parser;

use models::{Cli, Commands};
use commands::*;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // Get command handler
        Commands::Get(ref opt) => {
            get(&cli, opt);
        }
        // Print command handler
        Commands::Print => {
            print_env();
        }
        // Load command handler
        Commands::Load(ref opt) => {
            load(&cli, opt);
        }
        // Set command handler
        Commands::Set(ref opt) => {
            set(&cli, opt);
        }
        // Delete command handler
        Commands::Delete(ref opt) => {
            delete(&cli, opt)
        }
    }
}
