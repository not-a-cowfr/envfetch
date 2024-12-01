//! Lightweight cross-platform CLI tool for working with environment variables
//!
//! Easily get value of environment variable, get list of all variables,
//! run processes with specific variable value, or delete specific variable
//! to run process without it

mod utils;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use std::{env, fs, process};

use utils::{error, run, warning};

#[derive(Parser)]
#[command(
    author,
    version,
    after_help = "Get more info at project's repo: https://github.com/ankddev/envfetch",
    after_long_help = "Get more info at project's GitHub repo available at https://github.com/ankddev/envfetch",
    arg_required_else_help = true
)]
#[command(
    about = "envfetch - lightweight tool for working with environment variables",
    long_about = "envfetch is a lightweight cross-platform CLI tool for working with environment variables"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// All tool's commands
#[derive(Subcommand)]
enum Commands {
    /// Prints value of environment variable
    Get(GetArgs),
    /// Set environment variable and run given process.
    /// Note that the variable sets only for one run
    Set(SetArgs),
    /// Delete environment variable and run given process.
    /// Note that the variable deletes only for one run
    Delete(DeleteArgs),
    /// Load environment variables from dotenv file
    Load(LoadArgs),
    /// Prints all environment variables
    Print,
}

/// Args for get command
#[derive(Args, Debug)]
pub struct GetArgs {
    /// Environment variable name
    #[arg(required = true)]
    key: String,
    /// Disable showing similar variables' names if variable not found
    #[arg(long, short = 's', default_value = "false")]
    no_similar_names: bool,
}

#[derive(Args, Debug)]
pub struct LoadArgs {
    /// Process to start
    #[arg(required = true)]
    process: String,
    /// Relative or absolute path to file to read variables from.
    /// Note that it must in .env format
    #[arg(long, short, default_value = ".env")]
    file: String,
}

/// Args for set command
#[derive(Args, Debug)]
pub struct SetArgs {
    /// Environment variable name
    #[arg(required = true)]
    key: String,
    /// Value for environment variable
    #[arg(required = true)]
    value: String,
    /// Process to start
    #[arg(required = true)]
    process: String,
}

/// Args for delete command
#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Environment variable name
    #[arg(required = true)]
    key: String,
    /// Process to start
    #[arg(required = true)]
    process: String,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // Get command handler
        Commands::Get(opt) => {
            // Check if variable with specified name exists
            match env::var(&opt.key) {
                Ok(value) => println!("{:?}", &value),
                // If variable not found
                _ => {
                    error(format!("can't find '{}'", &opt.key).as_str());
                    // Check if we need to search for similar environment variables
                    if !opt.no_similar_names {
                        // Check for similar variables, if user made a mistake
                        let similar_names = env::vars()
                            .map(|el| el.0)
                            .filter(|el| {
                                similar_string::compare_similarity(
                                    opt.key.to_lowercase(),
                                    el.to_lowercase(),
                                ) > 0.6
                            })
                            .collect::<Vec<_>>();
                        if !similar_names.is_empty() {
                            eprintln!("Did you mean:");
                            for name in similar_names {
                                eprintln!("  {}", &name);
                            }
                        }
                    }
                    process::exit(1)
                }
            }
        }
        // Print command handler
        Commands::Print => {
            // Print all environment variables
            for (key, value) in env::vars() {
                println!("{} = {:?}", &key.blue(), &value);
            }
        }
        // Load command handler
        Commands::Load(opt) => {
            // Try to read file
            match fs::read_to_string(&opt.file) {
                Ok(content) => {
                    // Try to parse file
                    match dotenv_parser::parse_dotenv(&content) {
                        Ok(variables) => {
                            for (key, value) in variables.into_iter() {
                                unsafe { env::set_var(key, value) };
                            }
                            run(opt.process);
                        }
                        Err(err) => {
                            error(err.to_string().as_str());
                            run(opt.process);
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    error(err.to_string().as_str());
                    run(opt.process);
                    process::exit(1);
                }
            }
        }
        // Set command handler
        Commands::Set(opt) => {
            unsafe { env::set_var(opt.key, opt.value) };
            run(opt.process);
        }
        // Delete command handler
        Commands::Delete(opt) => {
            // Check if variable exists
            match env::var(&opt.key) {
                Ok(_) => unsafe { env::remove_var(&opt.key) },
                _ => warning("variable doesn't exists"),
            }
            run(opt.process);
        }
    }
}
