use rayon::prelude::*;
use std::{env, fs, process};

use crate::models::*;
use crate::utils::*;
use crate::variables;

/// Print all environment variables
pub fn print_env() {
    // Print all environment variables
    variables::print_env();
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
                        if let Err(err) = variables::set_variable(&key, &value, args.global, args.process.clone()) {
                            error(&err);
                            process::exit(1);
                        }
                    });
                }
                Err(err) => {
                    error(err.to_string().as_str());
                    if let Some(process) = args.process.clone() {
                        run(process);
                    }
                    process::exit(1)
                }
            }
        }
        Err(err) => {
            error(err.to_string().as_str());
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
                format!("can't find '{}'", &args.key).as_str());
            // Check if we need to search for similar environment variables
            if !args.no_similar_names {
                // Check for similar variables, if user made a mistake
                let similar_names = find_similar_string(args.key.clone(), env::vars().map(|(key, _)| key).collect(), 0.6);
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
        error(&err);
        process::exit(1);
    }

    if let Err(err) = variables::set_variable(&args.key, &args.value, args.global, args.process.clone()) {
        error(&err);
        process::exit(1);
    }
}

/// Delete environment variable
pub fn delete(args: &DeleteArgs, exit_on_warning: bool) {
    if let Err(err) = validate_var_name(&args.key) {
        error(&err);
        process::exit(1);
    }

    // Check if variable exists
    match env::var(&args.key) {
        Ok(_) => {
            if let Err(err) = variables::delete_variable(args.key.clone(), args.global) {
                error(&err);
                process::exit(1);
            }
        }
        _ => warning("variable doesn't exists", exit_on_warning),
    }
    if let Some(process) = args.process.clone() {
        run(process);
    }
}
