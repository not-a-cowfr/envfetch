//! Lightweight cross-platform CLI tool for working with environment variables
//!
//! Easily get value of environment variable, get list of all variables,
//! run processes with specific variable value, or delete specific variable
//! to run process without it

mod commands;
mod models;
mod utils;
mod variables;

use commands::{add, get, delete, load, print_env, set};
use models::Commands;

fn main() {
    let cli = commands::parse();

    match cli.command {
        // Get command handler
        Commands::Get(ref opt) => {
            get(opt);
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
        // Add command handler
        Commands::Add(ref opt) => {
            add(opt);
        }
        // Delete command handler
        Commands::Delete(ref opt) => {
            delete(opt, cli.exit_on_warning);
        }
    }
}
