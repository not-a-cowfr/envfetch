//! Lightweight cross-platform CLI tool for working with environment variables
//!
//! Easily get value of environment variable, get list of all variables,
//! run processes with specific variable value, or delete specific variable
//! to run process without it

mod commands;
mod interactive;
mod models;
mod utils;
mod variables;

use clap::Parser;
use std::{io::Write, process::ExitCode};

use commands::run_command;
use models::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}: {}",
                record.level().to_string().to_lowercase(),
                record.args()
            )
        })
        .init();
    run_command(&cli.command)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn test_get_command_without_no_similar_names_flag() {
        let args = Cli::parse_from(["envfetch", "get", "PATH"]);
        assert_eq!(
            args.command,
            Commands::Get(GetArgs {
                key: "PATH".to_string(),
                no_similar_names: false
            })
        );
    }

    #[test]
    fn test_get_command_with_no_similar_names_flag() {
        let args = Cli::parse_from(["envfetch", "get", "PATH", "--no-similar-names"]);
        assert_eq!(
            args.command,
            Commands::Get(GetArgs {
                key: "PATH".to_string(),
                no_similar_names: true
            })
        );
    }

    #[test]
    fn test_print_command() {
        let args = Cli::parse_from(["envfetch", "print"]);
        assert_eq!(args.command, Commands::Print(PrintArgs { format: None }));
    }

    #[test]
    fn test_print_command_with_format() {
        let args = Cli::parse_from(["envfetch", "print", "--format", "{name}: \"{value}\""]);
        assert_eq!(args.command, Commands::Print(PrintArgs { format: Some("{name}: \"{value}\"".to_owned()) }));
    }

    #[test]
    fn test_set_command_simple() {
        let args = Cli::parse_from(["envfetch", "set", "VAR", "VALUE", "--", "npm run"]);
        assert_eq!(
            args.command,
            Commands::Set(SetArgs {
                global: false,
                key: "VAR".to_string(),
                value: "VALUE".to_string(),
                process: Some("npm run".to_string())
            })
        );
    }

    #[test]
    fn test_set_command_with_global_flag() {
        let args = Cli::parse_from(["envfetch", "set", "VAR", "VALUE", "--global"]);
        assert_eq!(
            args.command,
            Commands::Set(SetArgs {
                global: true,
                key: "VAR".to_string(),
                value: "VALUE".to_string(),
                process: None
            })
        );
    }

    #[test]
    fn test_set_command_with_global_flag_and_process() {
        let args = Cli::parse_from(["envfetch", "set", "VAR", "VALUE", "--global", "--", "npm run"]);
        assert_eq!(
            args.command,
            Commands::Set(SetArgs {
                global: true,
                key: "VAR".to_string(),
                value: "VALUE".to_string(),
                process: Some("npm run".to_string())
            })
        );
    }

    #[test]
    fn test_add_command_simple() {
        let args = Cli::parse_from(["envfetch", "add", "PATH", "./executable", "--", "npm run"]);
        assert_eq!(
            args.command,
            Commands::Add(AddArgs {
                global: false,
                key: "PATH".to_string(),
                value: "./executable".to_string(),
                process: Some("npm run".to_string())
            })
        );
    }

    #[test]
    fn test_add_command_with_global_flag() {
        let args = Cli::parse_from(["envfetch", "add", "PATH", "./executable", "--global"]);
        assert_eq!(
            args.command,
            Commands::Add(AddArgs {
                global: true,
                key: "PATH".to_string(),
                value: "./executable".to_string(),
                process: None
            })
        );
    }

    #[test]
    fn test_add_command_with_global_flag_and_process() {
        let args = Cli::parse_from([
            "envfetch",
            "add",
            "PATH",
            "./executable",
            "--global",
            "--",
            "npm run",
        ]);
        assert_eq!(
            args.command,
            Commands::Add(AddArgs {
                global: true,
                key: "PATH".to_string(),
                value: "./executable".to_string(),
                process: Some("npm run".to_string())
            })
        );
    }

    #[test]
    fn test_delete_command_simple() {
        let args = Cli::parse_from(["envfetch", "delete", "VAR", "--", "npm run"]);
        assert_eq!(
            args.command,
            Commands::Delete(DeleteArgs {
                key: "VAR".to_string(),
                global: false,
                process: Some("npm run".to_string())
            })
        );
    }

    #[test]
    fn test_delete_command_with_global_flag() {
        let args = Cli::parse_from(["envfetch", "delete", "VAR", "--global"]);
        assert_eq!(
            args.command,
            Commands::Delete(DeleteArgs {
                key: "VAR".to_string(),
                global: true,
                process: None
            })
        );
    }

    #[test]
    fn test_delete_command_with_global_flag_and_process() {
        let args = Cli::parse_from(["envfetch", "delete", "VAR", "--global", "--", "npm run"]);
        assert_eq!(
            args.command,
            Commands::Delete(DeleteArgs {
                key: "VAR".to_string(),
                global: true,
                process: Some("npm run".to_string())
            })
        );
    }
}
