use std::fmt::Display;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    after_help = "Get more info at project's repo: https://github.com/ankddev/envfetch",
    after_long_help = "Get more info at project's GitHub repo available at https://github.com/ankddev/envfetch",
    arg_required_else_help = true,
    name = "envfetch"
)]
#[command(
    about = "envfetch - lightweight tool for working with environment variables",
    long_about = "envfetch is a lightweight cross-platform CLI tool for working with environment variables"
)]
pub struct Cli {
    /// Tool commands
    #[command(subcommand)]
    pub command: Commands,
}

/// All tool's commands
#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    /// Print value of environment variable.
    Get(GetArgs),
    /// Set environment variable and optionally run given process.
    Set(SetArgs),
    /// Add value to the end of environment variable and optionally run given process.
    Add(AddArgs),
    /// Delete environment variable and optionally run given process.
    Delete(DeleteArgs),
    /// Load environment variables from dotenv file and optionally run given process.
    Load(LoadArgs),
    /// Print all environment variables.
    Print,
}

/// Args for get command
#[derive(Args, Debug, PartialEq, Eq)]
pub struct GetArgs {
    /// Environment variable name
    #[arg(required = true)]
    pub key: String,
    /// Disable showing similar variables' names if variable not found
    #[arg(long, short = 's', default_value = "false")]
    pub no_similar_names: bool,
}

/// Args for load command
#[derive(Args, Debug, PartialEq, Eq)]
pub struct LoadArgs {
    /// Globally set variable
    #[arg(required = false, long, short)]
    pub global: bool,
    /// Process to start, not required if --global flag is set
    #[arg(required_unless_present = "global")]
    pub process: Option<String>,
    /// Relative or absolute path to file to read variables from.
    /// Note that it must be in .env format
    #[arg(long, short, default_value = ".env")]
    pub file: String,
}

/// Args for set command
#[derive(Args, Debug, PartialEq, Eq)]
pub struct SetArgs {
    /// Environment variable name
    #[arg(required = true)]
    pub key: String,
    /// Value for environment variable
    #[arg(required = true)]
    pub value: String,
    /// Globally set variable
    #[arg(required = false, long, short)]
    pub global: bool,
    /// Process to start, not required if --global flag is set
    #[arg(required_unless_present = "global")]
    pub process: Option<String>,
}

/// Args for add command
#[derive(Args, Debug, PartialEq, Eq)]
pub struct AddArgs {
    /// Environment variable name
    #[arg(required = true)]
    pub key: String,
    /// Value for add to the end of environment variable
    #[arg(required = true)]
    pub value: String,
    /// Globally set variable
    #[arg(required = false, long, short)]
    pub global: bool,
    /// Process to start, not required if --global flag is set
    #[arg(required_unless_present = "global")]
    pub process: Option<String>,
}

/// Args for delete command
#[derive(Args, Debug, PartialEq, Eq)]
pub struct DeleteArgs {
    /// Environment variable name
    #[arg(required = true)]
    pub key: String,
    /// Globally set variable
    #[arg(required = false, long, short)]
    pub global: bool,
    /// Process to start, not required if --global flag is set
    #[arg(required_unless_present = "global")]
    pub process: Option<String>,
}

pub enum ErrorKind {
    StartingProcessError,
    ProcessFailed,
    CannotSetVariableGlobally(String),
    CannotDeleteVariableGlobally(String),
    ParsingError(String),
    FileError(String),
    CannotFindVariable(String, bool),
    NameValidationError(String),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::StartingProcessError => write!(f, "Can't start process"),
            ErrorKind::ProcessFailed => write!(f, "Process failed"),
            ErrorKind::CannotSetVariableGlobally(err) => {
                write!(f, "Can't set variable globally (try running with sudo or administrative privileges): {}", err)
            }
            ErrorKind::CannotDeleteVariableGlobally(err) => {
                write!(f, "Can't delete variable globally (try running with sudo or administrative privileges): {}", err)
            }
            ErrorKind::ParsingError(err) => write!(f, "Parsing error: {}", err),
            ErrorKind::FileError(err) => write!(f, "File error: {}", err),
            ErrorKind::CannotFindVariable(name, _) => write!(f, "Can't find variable: {}", name),
            ErrorKind::NameValidationError(err) => write!(f, "Name validation error: {}", err),
        }
    }
}
