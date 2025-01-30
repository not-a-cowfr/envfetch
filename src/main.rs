//! Lightweight cross-platform CLI tool for working with environment variables
//!
//! Easily get value of environment variable, get list of all variables,
//! run processes with specific variable value, or delete specific variable
//! to run process without it

mod commands;
mod models;
mod utils;
mod variables;

use clap::Parser;

use commands::*;
use models::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // Get command handler
        Commands::Get(ref opt) => {
            get(opt, cli.exit_on_warning);
        }
        // Print command handler
        Commands::Print => {
            print_env();
        }
        // Load command handler
        Commands::Load(ref opt) => {
            load(opt);
        }
        // Set command handler
        Commands::Set(ref opt) => {
            set(opt);
        }
        // Delete command handler
        Commands::Delete(ref opt) => {
            delete(opt, cli.exit_on_warning);
        }
    }
}
