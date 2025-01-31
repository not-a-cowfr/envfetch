use clap::Parser;
use rayon::prelude::*;
use std::ffi::OsString;
use std::io::{stderr, stdout};
use std::{env, fs, process};

use crate::models::*;
use crate::utils::*;
use crate::variables;

pub fn parse() -> Cli {
    parse_from(env::args_os())
}

fn parse_from(iter: impl IntoIterator<Item = OsString>) -> Cli {
    Cli::parse_from(iter)
}

/// Print all environment variables
pub fn print_env() {
    // Print all environment variables
    variables::print_env(&mut stdout());
}

/// Load variables from dotenv-style file
pub fn load(args: &LoadArgs) {
    // Try to read file
    match fs::read_to_string(&args.file) {
        Ok(content) => {
            // Try to parse file
            match dotenv_parser::parse_dotenv(&content) {
                Ok(variables) => {
                    variables.into_par_iter().for_each(|(key, value)| {
                        if let Err(err) =
                            variables::set_variable(&key, &value, args.global, args.process.clone())
                        {
                            error(&err, &mut stderr());
                            process::exit(1);
                        }
                    });
                }
                Err(err) => {
                    error(err.to_string().as_str(), &mut stderr());
                    if let Some(process) = args.process.clone() {
                        run(process);
                    }
                    process::exit(1)
                }
            }
        }
        Err(err) => {
            error(err.to_string().as_str(), &mut stderr());
            if let Some(process) = args.process.clone() {
                run(process);
            }
            process::exit(1)
        }
    }
}

/// Get value of variable
pub fn get(args: &GetArgs) {
    // Check if variable with specified name exists
    match env::var(&args.key) {
        Ok(value) => println!("{:?}", &value),
        // If variable not found
        _ => {
            error(
                format!("can't find '{}'", &args.key).as_str(),
                &mut stderr(),
            );
            // Check if we need to search for similar environment variables
            if !args.no_similar_names {
                // Check for similar variables, if user made a mistake
                let similar_names = find_similar_string(
                    args.key.clone(),
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
            process::exit(1)
        }
    }
}

/// Set value to environment variable
pub fn set(args: &SetArgs) {
    if let Err(err) = validate_var_name(&args.key) {
        error(&err, &mut stderr());
        process::exit(1);
    }

    if let Err(err) =
        variables::set_variable(&args.key, &args.value, args.global, args.process.clone())
    {
        error(&err, &mut stderr());
        process::exit(1);
    }
}

/// Add value to environment variable
pub fn add(args: &AddArgs) {
    if let Err(err) = validate_var_name(&args.key) {
        error(&err, &mut stderr());
        process::exit(1);
    }

    let current_value = if let Ok(value) = env::var(&args.key) {
        value
    } else {
        "".to_string()
    };

    if let Err(err) = variables::set_variable(
        &args.key,
        &format!("{}{}", current_value, args.value),
        args.global,
        args.process.clone(),
    ) {
        error(&err, &mut stderr());
        process::exit(1);
    }
}

/// Delete environment variable
pub fn delete(args: &DeleteArgs, exit_on_warning: bool) {
    if let Err(err) = validate_var_name(&args.key) {
        error(&err, &mut stderr());
        process::exit(1);
    }

    // Check if variable exists
    match env::var(&args.key) {
        Ok(_) => {
            if let Err(err) = variables::delete_variable(args.key.clone(), args.global) {
                error(&err, &mut stderr());
                process::exit(1);
            }
        }
        _ => warning("variable doesn't exists", exit_on_warning, &mut stderr()),
    }
    if let Some(process) = args.process.clone() {
        run(process);
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{GetArgs, SetArgs};

    use super::*;

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
