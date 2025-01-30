use colored::Colorize;
use rayon::prelude::*;
use similar_string;
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
                    let vars_vec: Vec<_> = variables.into_iter().collect();
                    vars_vec.par_iter().for_each(|(key, value)| {
                        if let Err(err) = variables::set_variable(key, value, args.global, args.process.clone()) {
                            error(err.as_str());
                        }
                    });
                }
                Err(err) => {
                    error(err.to_string().as_str());
                    if let Some(process) = args.process.clone() {
                        run(process);
                    }
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            error(err.to_string().as_str());
            if let Some(process) = args.process.clone() {
                run(process);
            }
            process::exit(1);
        }
    }
}

/// Get value of variable
pub fn get(args: &GetArgs, exit_on_warning: bool) {
    // Check if variable with specified name exists
    match env::var(&args.key) {
        Ok(value) => println!("{:?}", &value),
        // If variable not found
        _ => {
            // FIXME: use error here and print before exitting
            warning(
                format!("can't find '{}'", &args.key).as_str(),
                exit_on_warning,
            );
            // Check if we need to search for similar environment variables
            if !args.no_similar_names {
                // Check for similar variables, if user made a mistake
                let similar_names = env::vars()
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .map(|(name, _)| name)
                    .filter(|name| {
                        similar_string::compare_similarity(
                            args.key.to_lowercase(),
                            name.to_lowercase(),
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
pub fn set(args: &SetArgs) {
    if let Err(err) = validate_var_name(&args.key) {
        error(&err);
        process::exit(1);
    }

    if args.global {
        if let Err(err) = globalenv::set_var(&args.key, &args.value) {
            error(&format!(
                "can't globally set variable: {} (do you have the required permissions?)",
                err
            ));
            process::exit(1);
        }
    } else {
        unsafe { env::set_var(&args.key, &args.value) };
    }
    if let Some(process) = args.process.clone() {
        run(process);
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
        Ok(_) if args.global => {
            if let Err(err) = globalenv::unset_var(&args.key) {
                error(&format!(
                    "can't globally delete variable: {} (do you have the required permissions?)",
                    err
                ));
            }
        }
        Ok(_) => unsafe { env::remove_var(&args.key) },
        _ => warning("variable doesn't exists", exit_on_warning),
    }
    if let Some(process) = args.process.clone() {
        run(process);
    }
}
