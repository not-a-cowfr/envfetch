use colored::Colorize;
use std::{env, fs, process};

use crate::models::*;
use crate::utils::*;

/// Print all environment variables
pub fn print_env() {
    // Print all environment variables
    for (key, value) in env::vars() {
        println!("{} = \"{}\"", key.blue(), value);
    }
}

/// Load variables from dotenv-style file
pub fn load(cli: &Cli, args: &LoadArgs) {
    // Try to read file
    match fs::read_to_string(&args.file) {
        Ok(content) => {
            // Try to parse file
            match dotenv_parser::parse_dotenv(&content) {
                Ok(variables) => {
                    for (key, value) in variables.into_iter() {
                        if args.global {
                            if let Err(err) = globalenv::set_var(&key, &value) {
                                error(
                                    &format!(
                                        "can't globally set variables: {} (do you have the required permissions?)",
                                        err
                                    ),
                                    cli.exit_on_error
                                );
                            }
                        } else {
                            unsafe { env::set_var(key, value) };
                        }
                    }
                    if let Some(process) = args.process.clone() {
                        run(process, cli.exit_on_error);
                    }
                }
                Err(err) => {
                    error(err.to_string().as_str(), cli.exit_on_error);
                    if let Some(process) = args.process.clone() {
                        run(process, cli.exit_on_error);
                    }
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            error(err.to_string().as_str(), cli.exit_on_error);
            if let Some(process) = args.process.clone() {
                run(process, cli.exit_on_error);
            }
            process::exit(1);
        }
    }
}

/// Get value of variable
pub fn get(cli: &Cli, args: &GetArgs) {
    // Check if variable with specified name exists
    match env::var(&args.key) {
        Ok(value) => println!("{:?}", &value),
        // If variable not found
        _ => {
            error(
                format!("can't find '{}'", &args.key).as_str(),
                cli.exit_on_error,
            );
            // Check if we need to search for similar environment variables
            if !args.no_similar_names {
                // Check for similar variables, if user made a mistake
                let similar_names = env::vars()
                    .map(|el| el.0)
                    .filter(|el| {
                        similar_string::compare_similarity(
                            args.key.to_lowercase(),
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

/// Set value to environment variable
pub fn set(cli: &Cli, args: &SetArgs) {
    if let Err(err) = validate_var_name(&args.key) {
        error(&err, cli.exit_on_error);
        process::exit(1);
    }

    if args.global {
        if let Err(err) = globalenv::set_var(&args.key, &args.value) {
            error(
                &format!(
                    "can't globally set variable: {} (do you have the required permissions?)",
                    err
                ),
                cli.exit_on_error,
            );
            process::exit(1);
        }
    } else {
        unsafe { env::set_var(&args.key, &args.value) };
    }
    if let Some(process) = args.process.clone() {
        run(process, cli.exit_on_error);
    }
}

/// Delete environment variable
pub fn delete(cli: &Cli, args: &DeleteArgs) {
    if let Err(err) = validate_var_name(&args.key) {
        error(&err, cli.exit_on_error);
        process::exit(1);
    }
    
    // Check if variable exists
    match env::var(&args.key) {
        Ok(_) if args.global => {
            if let Err(err) = globalenv::unset_var(&args.key) {
                error(
                    &format!(
                        "can't globally delete variable: {} (do you have the required permissions?)",
                        err
                    ),
                    cli.exit_on_error
                );
            }
        },
        Ok(_) => unsafe { env::remove_var(&args.key) }
        _ => warning("variable doesn't exists"),
    }
    if let Some(process) = args.process.clone() {
        run(process, cli.exit_on_error);
    }
}
