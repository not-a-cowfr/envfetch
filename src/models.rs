use clap::{Args, Parser, Subcommand};

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
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Exit on any error
    #[arg(long, short = 'e', global = true)]
    pub exit_on_error: bool,
}

/// All tool's commands
#[derive(Subcommand)]
pub enum Commands {
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
    pub key: String,
    /// Disable showing similar variables' names if variable not found
    #[arg(long, short = 's', default_value = "false")]
    pub no_similar_names: bool,
}

/// Args for load command
#[derive(Args, Debug)]
pub struct LoadArgs {
    /// Globally set variable
    #[arg(required = false, long, short)]
    pub global: bool,
    /// Process to start, not required if --global flag is set
    #[arg(required_unless_present = "global")]
    pub process: Option<String>,
    /// Relative or absolute path to file to read variables from.
    /// Note that it must in .env format
    #[arg(long, short, default_value = ".env")]
    pub file: String,
}

/// Args for set command
#[derive(Args, Debug)]
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

/// Args for delete command
#[derive(Args, Debug)]
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
