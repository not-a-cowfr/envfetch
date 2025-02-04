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
use log::error;
use std::{env, io::Write, process};
use utils::find_similar_string;

use commands::{add, delete, get, load, print_env, set};
use models::{Cli, Commands, ErrorKind};

fn main() {
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
    run_command(&cli.command);
}

fn run_command(command: &Commands) {
    match command {
        Commands::Get(ref opt) => {
            if let Err(error) = get(opt) {
                error!("{}", error);
                if let ErrorKind::CannotFindVariable(key, no_similar_names) = error {
                    if !no_similar_names {
                        let similar_names = find_similar_string(
                            key.clone(),
                            env::vars().map(|(key, _)| key).collect(),
                            0.6,
                        );
                        if !similar_names.is_empty() {
                            eprintln!("Did you mean:");
                            for name in similar_names {
                                eprintln!("  {}", &name);
                            }
                        }
                    }
                }
                process::exit(1);
            }
        }
        Commands::Print => print_env(),
        Commands::Load(ref opt) => {
            if let Err(error) = load(opt) {
                error!("{}", error);
                process::exit(1);
            }
        }
        Commands::Set(ref opt) => {
            if let Err(error) = set(opt) {
                error!("{}", error);
                process::exit(1);
            }
        }
        Commands::Add(ref opt) => {
            if let Err(error) = add(opt) {
                error!("{}", error);
                process::exit(1);
            }
        }
        Commands::Delete(ref opt) => {
            if let Err(error) = delete(opt) {
                error!("{}", error);
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use std::io;

    // Override stdout/stderr during tests
    fn with_captured_output<F: FnOnce()>(test: F) {
        let stdout = io::stdout();
        let stderr = io::stderr();
        let _lock_out = stdout.lock();
        let _lock_err = stderr.lock();
        test();
    }

    #[test]
    fn test_run_command_get_success() {
        env::set_var("TEST_RUN_VAR", "test_value");
        with_captured_output(|| {
            run_command(&Commands::Get(GetArgs {
                key: "TEST_RUN_VAR".to_string(),
                no_similar_names: false,
            }));
        });
        env::remove_var("TEST_RUN_VAR");
    }

    #[test]
    fn test_run_command_set() {
        with_captured_output(|| {
            run_command(&Commands::Set(SetArgs {
                key: "TEST_SET_RUN".to_string(),
                value: "test_value".to_string(),
                global: false,
                process: None,
            }));
        });

        assert_eq!(env::var("TEST_SET_RUN").unwrap(), "test_value");
        env::remove_var("TEST_SET_RUN");
    }

    #[test]
    fn test_run_command_add() {
        env::set_var("TEST_ADD_RUN", "initial_");

        with_captured_output(|| {
            run_command(&Commands::Add(AddArgs {
                key: "TEST_ADD_RUN".to_string(),
                value: "value".to_string(),
                global: false,
                process: None,
            }));
        });

        assert_eq!(env::var("TEST_ADD_RUN").unwrap(), "initial_value");
        env::remove_var("TEST_ADD_RUN");
    }

    #[test]
    fn test_run_command_print() {
        env::set_var("TEST_PRINT_RUN", "test_value");
        with_captured_output(|| {
            run_command(&Commands::Print);
        });
        env::remove_var("TEST_PRINT_RUN");
    }

    #[test]
    fn test_run_command_delete() {
        env::set_var("TEST_DELETE_RUN", "test_value");

        with_captured_output(|| {
            run_command(&Commands::Delete(DeleteArgs {
                key: "TEST_DELETE_RUN".to_string(),
                global: false,
                process: None,
            }));
        });

        assert!(env::var("TEST_DELETE_RUN").is_err());
    }

    #[test]
    fn test_run_command_load() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "TEST_LOAD_RUN=test_value").unwrap();

        with_captured_output(|| {
            run_command(&Commands::Load(LoadArgs {
                file: temp_file.path().to_string_lossy().to_string(),
                global: false,
                process: None,
            }));
        });

        assert_eq!(env::var("TEST_LOAD_RUN").unwrap(), "test_value");
        env::remove_var("TEST_LOAD_RUN");
    }

    #[test]
    fn test_get_command_without_no_similar_names_flag() {
        let args = Cli::parse_from(&["envfetch", "get", "PATH"]);
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
        let args = Cli::parse_from(&["envfetch", "get", "PATH", "--no-similar-names"]);
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
        let args = Cli::parse_from(&["envfetch", "print"]);
        assert_eq!(args.command, Commands::Print);
    }

    #[test]
    fn test_set_command_simple() {
        let args = Cli::parse_from(&["envfetch", "set", "VAR", "VALUE", "npm run"]);
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
        let args = Cli::parse_from(&["envfetch", "set", "VAR", "VALUE", "--global"]);
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
        let args = Cli::parse_from(&["envfetch", "set", "VAR", "VALUE", "npm run", "--global"]);
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
        let args = Cli::parse_from(&["envfetch", "add", "PATH", "./executable", "npm run"]);
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
        let args = Cli::parse_from(&["envfetch", "add", "PATH", "./executable", "--global"]);
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
        let args = Cli::parse_from(&[
            "envfetch",
            "add",
            "PATH",
            "./executable",
            "npm run",
            "--global",
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
        let args = Cli::parse_from(&["envfetch", "delete", "VAR", "npm run"]);
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
        let args = Cli::parse_from(&["envfetch", "delete", "VAR", "--global"]);
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
        let args = Cli::parse_from(&["envfetch", "delete", "VAR", "npm run", "--global"]);
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
